use crate::map::ResourceKind;

/// Messages exchanged between robots and the base via mpsc channels.
#[derive(Debug, Clone)]
pub enum Message {
    ResourceDiscovered {
        pos: (usize, usize),
        kind: ResourceKind,
        quantity: u32,
    },
    ObstacleDiscovered {
        pos: (usize, usize),
    },
    ResourceCollected {
        pos: (usize, usize),
        kind: ResourceKind,
    },
    ResourceDeposited {
        kind: ResourceKind,
        amount: u32,
    },
    ResourceDepleted {
        pos: (usize, usize),
    },
}
