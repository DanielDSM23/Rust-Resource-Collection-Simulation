use rand::{seq::SliceRandom, Rng};
use std::sync::mpsc::Sender;

use crate::map::Tile;
use crate::messages::Message;
use crate::simulation::SharedState;

#[derive(Debug, Clone, Copy, PartialEq)]
enum ScoutState {
    Exploring,
    ReturningToBase,
}

#[derive(Debug, Clone)]
struct Discovery {
    pos: (usize, usize),
    tile: Tile,
}

pub struct Scout {
    #[allow(dead_code)]
    pub id: usize,
    robot_index: usize,
    pub pos: (usize, usize),
    pub known: Vec<Vec<Option<Tile>>>,
    target: Option<(usize, usize)>,
    state: ScoutState,
    pending_discoveries: Vec<Discovery>,
}

impl Scout {
    pub fn new(id: usize, pos: (usize, usize), width: usize, height: usize) -> Self {
        Scout {
            id,
            robot_index: id,
            pos,
            known: vec![vec![None; height]; width],
            target: None,
            state: ScoutState::Exploring,
            pending_discoveries: Vec::new(),
        }
    }

    /// Advance the scout by one simulation tick.
    pub fn tick(&mut self, state: &SharedState, tx: &Sender<Message>) {
        let (width, height, base_pos) = {
            let s = state.lock().unwrap();
            (s.map.width, s.map.height, s.map.base_pos)
        };
        self.absorb_knowledge(state);
        self.scan_visible_area(state);

        match self.state {
            ScoutState::Exploring => {
                if self.target.is_none() {
                    let mut rng = rand::thread_rng();
                    let tx_coord = rng.gen_range(0..width);
                    let ty_coord = rng.gen_range(0..height);
                    self.target = Some((tx_coord, ty_coord));
                }

                if let Some(goal) = self.target {
                    if self.pos == goal {
                        self.target = None;
                        self.scan_current_tile(state);
                    } else {
                        let next = crate::robots::pathfinding::next_step_toward(
                            &self.known,
                            self.pos,
                            goal,
                            width,
                            height,
                        );
                        if let Some(step) = next {
                            if let Some(new_pos) = self.move_or_detour(step, state) {
                                self.pos = new_pos;
                                self.scan_visible_area(state);
                            } else {
                                self.target = None;
                            }
                        } else {
                            // Path blocked, pick new target
                            self.target = None;
                        }
                    }
                }

                if !self.pending_discoveries.is_empty() {
                    self.state = ScoutState::ReturningToBase;
                    self.target = Some(base_pos);
                }
            }
            ScoutState::ReturningToBase => {
                if self.pos == base_pos {
                    self.share_discoveries(tx);
                    self.state = ScoutState::Exploring;
                    self.target = None;
                } else {
                    let next = crate::robots::pathfinding::next_step_toward(
                        &self.known,
                        self.pos,
                        base_pos,
                        width,
                        height,
                    );
                    if let Some(step) = next {
                        if let Some(new_pos) = self.move_or_detour(step, state) {
                            self.pos = new_pos;
                            self.scan_visible_area(state);
                        }
                    }
                }
            }
        }
    }

    fn scan_current_tile(&mut self, state: &SharedState) {
        let s = state.lock().unwrap();
        let (x, y) = self.pos;

        if self.known[x][y].is_some() {
            return;
        }

        let tile = s.map.get(x, y).clone();
        self.known[x][y] = Some(tile.clone());

        match tile {
            Tile::Resource { .. } | Tile::Obstacle => {
                self.pending_discoveries
                    .push(Discovery { pos: (x, y), tile });
            }
            _ => {}
        }
    }

    fn scan_visible_area(&mut self, state: &SharedState) {
        let (width, height) = {
            let s = state.lock().unwrap();
            (s.map.width, s.map.height)
        };

        let (x, y) = self.pos;
        let mut positions = vec![(x, y)];
        if x > 0 {
            positions.push((x - 1, y));
        }
        if x + 1 < width {
            positions.push((x + 1, y));
        }
        if y > 0 {
            positions.push((x, y - 1));
        }
        if y + 1 < height {
            positions.push((x, y + 1));
        }

        let s = state.lock().unwrap();
        for (px, py) in positions {
            if self.known[px][py].is_some() {
                continue;
            }

            let tile = s.map.get(px, py).clone();
            self.known[px][py] = Some(tile.clone());

            match tile {
                Tile::Resource { .. } | Tile::Obstacle => {
                    self.pending_discoveries.push(Discovery {
                        pos: (px, py),
                        tile,
                    });
                }
                _ => {}
            }
        }
    }

    fn share_discoveries(&mut self, tx: &Sender<Message>) {
        for discovery in self.pending_discoveries.drain(..) {
            match discovery.tile {
                Tile::Resource { kind, quantity } => {
                    let _ = tx.send(Message::ResourceDiscovered {
                        pos: discovery.pos,
                        kind,
                        quantity,
                    });
                }
                Tile::Obstacle => {
                    let _ = tx.send(Message::ObstacleDiscovered { pos: discovery.pos });
                }
                _ => {}
            }
        }
    }

    fn absorb_knowledge(&mut self, state: &SharedState) {
        let s = state.lock().unwrap();
        for (pos, tile) in &s.base.known_resources {
            self.known[pos.0][pos.1] = Some(tile.clone());
        }
        for pos in &s.base.known_obstacles {
            self.known[pos.0][pos.1] = Some(Tile::Obstacle);
        }
    }

    fn try_move(&mut self, step: (usize, usize), state: &SharedState) -> bool {
        let s = state.lock().unwrap();
        if matches!(s.map.get(step.0, step.1), Tile::Obstacle) {
            self.known[step.0][step.1] = Some(Tile::Obstacle);
            self.pending_discoveries.push(Discovery {
                pos: step,
                tile: Tile::Obstacle,
            });
            return false;
        }

        let base_pos = s.map.base_pos;
        let occupied = step != base_pos
            && s.robot_positions
                .iter()
                .enumerate()
                .any(|(index, robot)| index != self.robot_index && robot.pos == step);

        !occupied
    }

    fn move_or_detour(
        &mut self,
        preferred: (usize, usize),
        state: &SharedState,
    ) -> Option<(usize, usize)> {
        if self.try_move(preferred, state) {
            return Some(preferred);
        }

        let mut candidates = self.neighbor_positions(state);
        candidates.shuffle(&mut rand::thread_rng());
        candidates
            .into_iter()
            .find(|candidate| self.try_move(*candidate, state))
    }

    fn neighbor_positions(&self, state: &SharedState) -> Vec<(usize, usize)> {
        let (width, height) = {
            let s = state.lock().unwrap();
            (s.map.width, s.map.height)
        };

        let (x, y) = self.pos;
        let mut positions = Vec::with_capacity(4);
        if x > 0 {
            positions.push((x - 1, y));
        }
        if x + 1 < width {
            positions.push((x + 1, y));
        }
        if y > 0 {
            positions.push((x, y - 1));
        }
        if y + 1 < height {
            positions.push((x, y + 1));
        }
        positions
    }
}
