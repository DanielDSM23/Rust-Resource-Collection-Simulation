use std::collections::{HashMap, VecDeque};

use crate::map::Tile;


pub fn next_step_toward(
    known: &[Vec<Option<Tile>>],
    from: (usize, usize),
    goal: (usize, usize),
    width: usize,
    height: usize,
) -> Option<(usize, usize)> {
    if from == goal {
        return None;
    }

    let mut queue = VecDeque::new();
    let mut came_from: HashMap<(usize, usize), (usize, usize)> = HashMap::new();

    queue.push_back(from);
    came_from.insert(from, from);

    while let Some(current) = queue.pop_front() {
        if current == goal {
            let mut step = goal;
            while came_from[&step] != from {
                step = came_from[&step];
            }
            return Some(step);
        }

        for neighbor in neighbors(current, width, height) {
            if came_from.contains_key(&neighbor) {
                continue;
            }

            let passable = if let Some(tile) = known[neighbor.0][neighbor.1].as_ref() {
                tile.is_walkable()
            } else {
                true
            };

            if passable {
                came_from.insert(neighbor, current);
                queue.push_back(neighbor);
            }
        }
    }
    None
}

fn neighbors(pos: (usize, usize), width: usize, height: usize) -> Vec<(usize, usize)> {
    let (x, y) = pos;
    let mut result = Vec::with_capacity(4);
    if x > 0 {
        result.push((x - 1, y));
    }
    if x + 1 < width {
        result.push((x + 1, y));
    }
    if y > 0 {
        result.push((x, y - 1));
    }
    if y + 1 < height {
        result.push((x, y + 1));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_first_step_toward_reachable_goal() {
        let known = vec![vec![Some(Tile::Empty); 3]; 3];

        let step = next_step_toward(&known, (0, 0), (2, 0), 3, 3);

        assert_eq!(step, Some((1, 0)));
    }

    #[test]
    fn avoids_known_obstacles() {
        let mut known = vec![vec![Some(Tile::Empty); 3]; 3];
        known[1][0] = Some(Tile::Obstacle);

        let step = next_step_toward(&known, (0, 0), (2, 0), 3, 3);

        assert_eq!(step, Some((0, 1)));
    }

    #[test]
    fn returns_none_when_goal_is_unreachable() {
        let mut known = vec![vec![Some(Tile::Empty); 3]; 3];
        known[1][0] = Some(Tile::Obstacle);
        known[0][1] = Some(Tile::Obstacle);

        let step = next_step_toward(&known, (0, 0), (2, 2), 3, 3);

        assert_eq!(step, None);
    }
}
