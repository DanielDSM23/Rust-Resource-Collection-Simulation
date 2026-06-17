use noise::{NoiseFn, Perlin};
use rand::Rng;

use super::tile::{ResourceKind, Tile};

pub struct GameMap {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Vec<Tile>>,
    pub base_pos: (usize, usize),
}