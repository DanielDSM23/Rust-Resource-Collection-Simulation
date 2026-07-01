use noise::{NoiseFn, Perlin};
use rand::{Rng};

use super::tile::{ResourceKind, Tile};

pub struct GameMap {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Vec<Tile>>,
    pub base_pos: (usize, usize),
}

impl GameMap {
    /// Generate a new map using Perlin noise for obstacles and random resource placement.
    pub fn generate(width: usize, height: usize) -> Self {
        let mut rng = rand::thread_rng();
        let perlin = Perlin::new(rng.r#gen());

        let base_pos = (width / 2, height / 2);

        let mut tiles = vec![vec![Tile::Empty; height]; width];

        // Generate obstacles with Perlin noise
        for (x, column) in tiles.iter_mut().enumerate() {
            for (y, tile) in column.iter_mut().enumerate() {
                let nx = x as f64 / width as f64 * 4.0;
                let ny = y as f64 / height as f64 * 4.0;
                let val = perlin.get([nx, ny]);
                if val > 0.35 {
                    *tile = Tile::Obstacle;
                }
            }
        }

        // Place the base and clear a small area around it
        let (bx, by) = base_pos;
        for dx in 0..=2_usize {
            for dy in 0..=2_usize {
                let cx = bx.saturating_sub(1) + dx;
                let cy = by.saturating_sub(1) + dy;
                if cx < width && cy < height {
                    tiles[cx][cy] = Tile::Empty;
                }
            }
        }
        tiles[bx][by] = Tile::Base;

        // Place energy and crystal resources
        let resource_count = (width * height) / 40;
        let mut placed = 0;
        let mut attempts = 0;
        while placed < resource_count && attempts < resource_count * 20 {
            attempts += 1;
            let x = rng.gen_range(0..width);
            let y = rng.gen_range(0..height);
            if tiles[x][y] == Tile::Empty && (x, y) != base_pos {
                let kind = if rng.gen_bool(0.5) {
                    ResourceKind::Energy
                } else {
                    ResourceKind::Crystal
                };
                let quantity = rng.gen_range(50..=200);
                tiles[x][y] = Tile::Resource { kind, quantity };
                placed += 1;
            }
        }

        GameMap {
            width,
            height,
            tiles,
            base_pos,
        }
    }

    pub fn get(&self, x: usize, y: usize) -> &Tile {
        &self.tiles[x][y]
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> &mut Tile {
        &mut self.tiles[x][y]
    }
}
