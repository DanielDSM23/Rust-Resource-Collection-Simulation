use rand::seq::SliceRandom;
use std::sync::mpsc::Sender;

use crate::map::{ResourceKind, Tile};
use crate::messages::Message;
use crate::simulation::SharedState;

#[derive(Debug, Clone, PartialEq)]
enum CollectorState {
    Idle,
    GoingToResource { target: (usize, usize) },
    Collecting { target: (usize, usize) },
    ReturningToBase,
}

pub struct Collector {
    #[allow(dead_code)]
    pub id: usize,
    robot_index: usize,
    pub pos: (usize, usize),
    pub known: Vec<Vec<Option<Tile>>>,
    pub carrying: Option<(ResourceKind, u32)>,
    state: CollectorState,
}

impl Collector {
    pub fn new(
        id: usize,
        robot_index: usize,
        pos: (usize, usize),
        width: usize,
        height: usize,
    ) -> Self {
        Collector {
            id,
            robot_index,
            pos,
            known: vec![vec![None; height]; width],
            carrying: None,
            state: CollectorState::Idle,
        }
    }

    pub fn tick(&mut self, state: &SharedState, tx: &Sender<Message>) {
        let (width, height, base_pos) = {
            let s = state.lock().unwrap();
            (s.map.width, s.map.height, s.map.base_pos)
        };

        self.absorb_knowledge(state);

        match self.state.clone() {
            CollectorState::Idle => {
                if let Some(target) = self.find_resource_target(state) {
                    self.state = CollectorState::GoingToResource { target };
                }
            }

            CollectorState::GoingToResource { target } => {
                if self.pos == target {
                    self.state = CollectorState::Collecting { target };
                } else {
                    let next = crate::robots::pathfinding::next_step_toward(
                        &self.known,
                        self.pos,
                        target,
                        width,
                        height,
                    );
                    if let Some(step) = next {
                        if let Some(new_pos) = self.move_or_detour(step, state, tx) {
                            self.pos = new_pos;
                        }
                    } else {
                        release_claim(state, target);
                        self.state = CollectorState::Idle;
                    }
                }
            }

            CollectorState::Collecting { target } => {
                let mut s = state.lock().unwrap();
                match s.map.get_mut(target.0, target.1) {
                    Tile::Resource { kind, quantity } => {
                        if *quantity == 0 {
                            drop(s);
                            release_claim(state, target);
                            self.known[target.0][target.1] = Some(Tile::Empty);
                            self.state = CollectorState::Idle;
                            return;
                        }

                        let k = *kind;
                        *quantity -= 1;
                        let remaining = *quantity;
                        drop(s);

                        let _ = tx.send(Message::ResourceCollected {
                            pos: target,
                            kind: k,
                        });
                        self.carrying = Some((k, 1));

                        if remaining == 0 {
                            let _ = tx.send(Message::ResourceDepleted { pos: target });
                            // Update local knowledge
                            self.known[target.0][target.1] = Some(Tile::Empty);
                        }
                        release_claim(state, target);
                        self.state = CollectorState::ReturningToBase;
                    }
                    _ => {
                        // Resource gone
                        drop(s);
                        release_claim(state, target);
                        self.known[target.0][target.1] = Some(Tile::Empty);
                        self.state = CollectorState::Idle;
                    }
                }
            }

            CollectorState::ReturningToBase => {
                if self.pos == base_pos {
                    // Deposit resources
                    if let Some((kind, amount)) = self.carrying.take() {
                        let _ = tx.send(Message::ResourceDeposited { kind, amount });
                    }
                    self.state = CollectorState::Idle;
                } else {
                    let next = crate::robots::pathfinding::next_step_toward(
                        &self.known,
                        self.pos,
                        base_pos,
                        width,
                        height,
                    );
                    if let Some(step) = next {
                        if let Some(new_pos) = self.move_or_detour(step, state, tx) {
                            self.pos = new_pos;
                        }
                    }
                }
            }
        }
    }

    fn find_resource_target(&self, state: &SharedState) -> Option<(usize, usize)> {
        let mut s = state.lock().unwrap();
        let target = s
            .base
            .known_resources
            .iter()
            .filter(|(pos, tile)| {
                !s.claimed_resources.contains(pos)
                    && matches!(tile, Tile::Resource { quantity, .. } if *quantity > 0)
            })
            .min_by_key(|(pos, _)| {
                let dx = pos.0 as i32 - self.pos.0 as i32;
                let dy = pos.1 as i32 - self.pos.1 as i32;
                dx * dx + dy * dy
            })
            .map(|(pos, _)| *pos);

        if let Some(pos) = target {
            s.claimed_resources.insert(pos);
        }

        target
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

    fn try_move(
        &mut self,
        step: (usize, usize),
        state: &SharedState,
        tx: &Sender<Message>,
    ) -> bool {
        let mut s = state.lock().unwrap();
        if matches!(s.map.get(step.0, step.1), Tile::Obstacle) {
            self.known[step.0][step.1] = Some(Tile::Obstacle);
            s.base.known_obstacles.insert(step);
            let _ = tx.send(Message::ObstacleDiscovered { pos: step });
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
        tx: &Sender<Message>,
    ) -> Option<(usize, usize)> {
        if self.try_move(preferred, state, tx) {
            return Some(preferred);
        }

        let mut candidates = self.neighbor_positions(state);
        candidates.shuffle(&mut rand::thread_rng());
        candidates
            .into_iter()
            .find(|candidate| self.try_move(*candidate, state, tx))
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

fn release_claim(state: &SharedState, target: (usize, usize)) {
    state.lock().unwrap().claimed_resources.remove(&target);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};
    use std::sync::{Arc, Mutex};

    use crate::base::Base;
    use crate::map::GameMap;
    use crate::simulation::{RobotPos, SimState};

    fn test_state(width: usize, height: usize) -> SharedState {
        let base_pos = (0, 0);
        let map = GameMap {
            width,
            height,
            tiles: vec![vec![Tile::Empty; height]; width],
            base_pos,
        };

        Arc::new(Mutex::new(SimState {
            map,
            base: Base::new(base_pos),
            robot_positions: vec![
                RobotPos {
                    pos: base_pos,
                    is_collector: true,
                },
                RobotPos {
                    pos: (1, 0),
                    is_collector: true,
                },
            ],
            claimed_resources: HashSet::new(),
            event_log: Vec::new(),
            event_log_scroll: 0,
            tick: 0,
            running: true,
        }))
    }

    #[test]
    fn collector_claims_known_resource_before_targeting_it() {
        let state = test_state(4, 4);
        {
            let mut s = state.lock().unwrap();
            s.base.known_resources = HashMap::from([(
                (2, 2),
                Tile::Resource {
                    kind: ResourceKind::Energy,
                    quantity: 5,
                },
            )]);
        }

        let collector = Collector::new(0, 0, (0, 0), 4, 4);

        assert_eq!(collector.find_resource_target(&state), Some((2, 2)));
        assert!(state.lock().unwrap().claimed_resources.contains(&(2, 2)));
    }

    #[test]
    fn collector_does_not_target_already_claimed_resource() {
        let state = test_state(4, 4);
        {
            let mut s = state.lock().unwrap();
            s.base.known_resources = HashMap::from([(
                (2, 2),
                Tile::Resource {
                    kind: ResourceKind::Crystal,
                    quantity: 5,
                },
            )]);
            s.claimed_resources.insert((2, 2));
        }

        let collector = Collector::new(0, 0, (0, 0), 4, 4);

        assert_eq!(collector.find_resource_target(&state), None);
    }

    #[test]
    fn collector_refuses_occupied_non_base_cell() {
        let state = test_state(4, 4);
        let (tx, _rx) = std::sync::mpsc::channel();
        let mut collector = Collector::new(0, 0, (0, 0), 4, 4);

        assert!(!collector.try_move((1, 0), &state, &tx));
    }

    #[test]
    fn collector_records_obstacle_before_entering_it() {
        let state = test_state(4, 4);
        {
            let mut s = state.lock().unwrap();
            s.map.tiles[1][1] = Tile::Obstacle;
        }
        let (tx, rx) = std::sync::mpsc::channel();
        let mut collector = Collector::new(0, 0, (0, 0), 4, 4);

        assert!(!collector.try_move((1, 1), &state, &tx));
        assert_eq!(collector.known[1][1], Some(Tile::Obstacle));
        assert!(matches!(
            rx.try_recv(),
            Ok(Message::ObstacleDiscovered { pos: (1, 1) })
        ));
    }
}
