use std::collections::{HashMap, HashSet};

use crate::map::{ResourceKind, Tile};
use crate::messages::Message;

/// The central base: aggregates all knowledge and stores collected resources.
pub struct Base {
    pub pos: (usize, usize),
    /// Globally known resource positions from scout reports.
    pub known_resources: HashMap<(usize, usize), Tile>,
    /// Globally known obstacle positions.
    pub known_obstacles: HashSet<(usize, usize)>,
    /// Total energy stored at the base.
    pub total_energy: u32,
    /// Total crystals stored at the base.
    pub total_crystals: u32,
}

impl Base {
    pub fn new(pos: (usize, usize)) -> Self {
        Base {
            pos,
            known_resources: HashMap::new(),
            known_obstacles: HashSet::new(),
            total_energy: 0,
            total_crystals: 0,
        }
    }

    /// Process an incoming message and update global state.
    pub fn process_message(&mut self, msg: Message) {
        match msg {
            Message::ResourceDiscovered {
                pos,
                kind,
                quantity,
            } => {
                self.known_resources
                    .entry(pos)
                    .or_insert(Tile::Resource { kind, quantity });
            }
            Message::ObstacleDiscovered { pos } => {
                self.known_obstacles.insert(pos);
            }
            Message::ResourceCollected { pos, kind } => {
                let _ = kind;
                // Update quantity in known map
                if let Some(Tile::Resource { quantity, .. }) = self.known_resources.get_mut(&pos) {
                    if *quantity > 0 {
                        *quantity -= 1;
                    }
                }
            }
            Message::ResourceDepleted { pos } => {
                self.known_resources.remove(&pos);
            }
            Message::ResourceDeposited { kind, amount } => match kind {
                ResourceKind::Energy => self.total_energy += amount,
                ResourceKind::Crystal => self.total_crystals += amount,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_discoveries_are_aggregated_once() {
        let mut base = Base::new((1, 1));

        base.process_message(Message::ResourceDiscovered {
            pos: (2, 3),
            kind: ResourceKind::Energy,
            quantity: 80,
        });
        base.process_message(Message::ResourceDiscovered {
            pos: (2, 3),
            kind: ResourceKind::Crystal,
            quantity: 120,
        });

        assert_eq!(base.known_resources.len(), 1);
        assert_eq!(
            base.known_resources.get(&(2, 3)),
            Some(&Tile::Resource {
                kind: ResourceKind::Energy,
                quantity: 80
            })
        );
    }

    #[test]
    fn deposits_update_resource_totals() {
        let mut base = Base::new((0, 0));

        base.process_message(Message::ResourceDeposited {
            kind: ResourceKind::Energy,
            amount: 3,
        });
        base.process_message(Message::ResourceDeposited {
            kind: ResourceKind::Crystal,
            amount: 5,
        });

        assert_eq!(base.total_energy, 3);
        assert_eq!(base.total_crystals, 5);
    }

    #[test]
    fn depletion_removes_known_resource() {
        let mut base = Base::new((0, 0));

        base.process_message(Message::ResourceDiscovered {
            pos: (4, 4),
            kind: ResourceKind::Crystal,
            quantity: 1,
        });
        base.process_message(Message::ResourceDepleted { pos: (4, 4) });

        assert!(!base.known_resources.contains_key(&(4, 4)));
    }
}
