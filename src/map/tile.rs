/// Kind of collectible resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceKind {
    Energy,
    Crystal,
}

/// Every cell on the map is one of these tiles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tile {
    Empty,
    Obstacle,
    Base,
    Resource { kind: ResourceKind, quantity: u32 },
}

impl Tile {
    pub fn is_walkable(&self) -> bool {
        !matches!(self, Tile::Obstacle)
    }
}
