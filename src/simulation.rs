use std::collections::HashSet;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::base::Base;
use crate::map::GameMap;
use crate::messages::Message;
use crate::robots::{Collector, Scout};

const NUM_SCOUTS: usize = 3;
const NUM_COLLECTORS: usize = 3;
const TICK_MS: u64 = 80;

#[derive(Clone)]
pub struct RobotPos {
    pub pos: (usize, usize),
    pub is_collector: bool,
}

pub struct SimState {
    pub map: GameMap,
    pub base: Base,
    pub robot_positions: Vec<RobotPos>,
    pub claimed_resources: HashSet<(usize, usize)>,
    pub event_log: Vec<String>,
    pub event_log_scroll: usize,
    pub tick: u64,
    pub running: bool,
}

pub type SharedState = Arc<Mutex<SimState>>;

pub struct Simulation {
    pub state: SharedState,
    robot_handles: Vec<JoinHandle<()>>,
    message_handle: Option<JoinHandle<()>>,
}

impl Simulation {
    pub fn new(width: usize, height: usize) -> Self {
        let map = GameMap::generate(width, height);
        let base = Base::new(map.base_pos);

        let mut robot_positions = Vec::new();
        for _ in 0..NUM_SCOUTS {
            robot_positions.push(RobotPos {
                pos: map.base_pos,
                is_collector: false,
            });
        }
        for _ in 0..NUM_COLLECTORS {
            robot_positions.push(RobotPos {
                pos: map.base_pos,
                is_collector: true,
            });
        }

        let state = Arc::new(Mutex::new(SimState {
            map,
            base,
            robot_positions,
            claimed_resources: HashSet::new(),
            event_log: Vec::new(),
            event_log_scroll: 0,
            tick: 0,
            running: true,
        }));

        Simulation {
            state,
            robot_handles: Vec::new(),
            message_handle: None,
        }
    }

    pub fn start(&mut self) {
        let (tx, rx): (Sender<Message>, Receiver<Message>) = mpsc::channel();

        for i in 0..NUM_SCOUTS {
            let state_clone = Arc::clone(&self.state);
            let tx_clone = tx.clone();
            let handle = thread::spawn(move || {
                let (w, h, pos) = {
                    let s = state_clone.lock().unwrap();
                    (s.map.width, s.map.height, s.map.base_pos)
                };
                let mut scout = Scout::new(i, pos, w, h);
                while is_running(&state_clone) {
                    scout.tick(&state_clone, &tx_clone);
                    {
                        let mut s = state_clone.lock().unwrap();
                        s.robot_positions[i].pos = scout.pos;
                    }
                    thread::sleep(Duration::from_millis(TICK_MS));
                }
            });
            self.robot_handles.push(handle);
        }

        for i in 0..NUM_COLLECTORS {
            let state_clone = Arc::clone(&self.state);
            let tx_clone = tx.clone();
            let handle = thread::spawn(move || {
                let (w, h, pos) = {
                    let s = state_clone.lock().unwrap();
                    (s.map.width, s.map.height, s.map.base_pos)
                };
                let mut collector = Collector::new(i, NUM_SCOUTS + i, pos, w, h);
                while is_running(&state_clone) {
                    collector.tick(&state_clone, &tx_clone);
                    {
                        let mut s = state_clone.lock().unwrap();
                        s.robot_positions[NUM_SCOUTS + i].pos = collector.pos;
                    }
                    thread::sleep(Duration::from_millis(TICK_MS));
                }
            });
            self.robot_handles.push(handle);
        }

        let state_clone = Arc::clone(&self.state);
        self.message_handle = Some(thread::spawn(move || {
            for msg in rx {
                let log_entry = describe_message(&msg);
                if let Message::ResourceDepleted { pos } = msg {
                    let mut s = state_clone.lock().unwrap();
                    s.claimed_resources.remove(&pos);
                    s.base.process_message(Message::ResourceDepleted { pos });
                    if let Some(entry) = log_entry {
                        push_log_entry(&mut s, entry);
                    }
                    s.tick += 1;
                    continue;
                }

                let mut s = state_clone.lock().unwrap();
                s.base.process_message(msg);
                if let Some(entry) = log_entry {
                    push_log_entry(&mut s, entry);
                }
                s.tick += 1;
            }
        }));
    }

    pub fn stop(&mut self) {
        {
            let mut s = self.state.lock().unwrap();
            s.running = false;
        }

        for handle in self.robot_handles.drain(..) {
            let _ = handle.join();
        }

        if let Some(handle) = self.message_handle.take() {
            let _ = handle.join();
        }
    }
}

fn is_running(state: &SharedState) -> bool {
    state.lock().unwrap().running
}

fn push_log_entry(state: &mut SimState, entry: String) {
    state.event_log.push(entry);
    if state.event_log.len() > 100 {
        state.event_log.remove(0);
        if state.event_log_scroll > 0 {
            state.event_log_scroll = state.event_log_scroll.saturating_sub(1);
        }
    }
}

fn describe_message(msg: &Message) -> Option<String> {
    match msg {
        Message::ResourceDiscovered {
            pos,
            kind,
            quantity,
        } => Some(format!(
            "Nouveau gisement {:?} decouvert en ({}, {}) - {} unite(s)",
            kind, pos.0, pos.1, quantity
        )),
        Message::ObstacleDiscovered { pos } => {
            Some(format!("Obstacle detecte en ({}, {})", pos.0, pos.1))
        }
        Message::ResourceCollected { pos, kind } => Some(format!(
            "1 unite de {:?} recoltee en ({}, {})",
            kind, pos.0, pos.1
        )),
        Message::ResourceDeposited { kind, amount } => Some(format!(
            "{} unite(s) de {:?} deposee(s) a la base",
            amount, kind
        )),
        Message::ResourceDepleted { pos } => {
            Some(format!("Gisement epuise en ({}, {})", pos.0, pos.1))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simulation_initializes_all_robots_at_base() {
        let sim = Simulation::new(20, 10);
        let state = sim.state.lock().unwrap();

        assert_eq!(state.robot_positions.len(), NUM_SCOUTS + NUM_COLLECTORS);
        assert!(state
            .robot_positions
            .iter()
            .all(|robot| robot.pos == state.map.base_pos));
    }

    #[test]
    fn message_descriptions_are_human_readable_for_log() {
        let msg = Message::ResourceDepleted { pos: (4, 7) };

        assert_eq!(
            describe_message(&msg),
            Some("Gisement epuise en (4, 7)".to_string())
        );
    }

    #[test]
    fn simulation_can_start_and_stop_threads() {
        let mut sim = Simulation::new(12, 8);

        sim.start();
        sim.stop();

        assert!(!sim.state.lock().unwrap().running);
    }
}
