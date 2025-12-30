use std::collections::{HashMap, VecDeque};

use uuid::Uuid;

use crate::{ecs::resources::Position, game::behavior::GameError};

#[derive(Debug, Clone)]
pub struct BfsMap {
    width: u8,
    height: u8,
    dist: Vec<Option<u32>>,
    parent: Vec<Option<Position>>,
}

impl BfsMap {
    fn idx(&self, pos: Position) -> Option<usize> {
        if pos.x < 0 || pos.y < 0 || pos.x >= self.width as i32 || pos.y >= self.height as i32 {
            return None;
        }
        Some((pos.y as usize) * (self.width as usize) + (pos.x as usize))
    }

    pub fn distance_to(&self, pos: Position) -> Option<u32> {
        let idx = self.idx(pos)?;
        self.dist[idx]
    }

    pub fn reconstruct_path_to(&self, dest: Position) -> Option<Vec<Position>> {
        let dest_idx = self.idx(dest)?;
        if self.dist[dest_idx].is_none() {
            return None;
        }

        let mut current = dest;
        let mut out = Vec::new();
        loop {
            out.push(current);
            let idx = self.idx(current)?;
            let Some(prev) = self.parent[idx] else {
                break;
            };
            current = prev;
        }
        out.reverse();
        Some(out)
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeField {
    pub width: u8,
    pub height: u8,
    reserved_by_unit: HashMap<Uuid, Position>,
    reserved_by_pos: HashMap<Position, Uuid>,
}

impl RuntimeField {
    pub fn new(width: u8, height: u8) -> Self {
        Self {
            width,
            height,
            reserved_by_unit: HashMap::new(),
            reserved_by_pos: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.reserved_by_unit.clear();
        self.reserved_by_pos.clear();
    }

    pub fn in_bounds(&self, pos: Position) -> bool {
        pos.x >= 0 && pos.y >= 0 && pos.x < self.width as i32 && pos.y < self.height as i32
    }

    fn idx(&self, pos: Position) -> Result<usize, GameError> {
        if !self.in_bounds(pos) {
            return Err(GameError::OutOfBounds);
        }
        Ok((pos.y as usize) * (self.width as usize) + (pos.x as usize))
    }

    pub fn reserved_destination_of(&self, unit: Uuid) -> Option<Position> {
        self.reserved_by_unit.get(&unit).copied()
    }

    pub fn reserved_unit_at(&self, pos: Position) -> Option<Uuid> {
        self.reserved_by_pos.get(&pos).copied()
    }

    pub fn reserve(&mut self, unit: Uuid, pos: Position) -> Result<(), GameError> {
        self.cancel_reservation(unit);

        let _ = self.idx(pos)?;
        if self.reserved_by_pos.contains_key(&pos) {
            return Err(GameError::PositionOccupied);
        }

        self.reserved_by_unit.insert(unit, pos);
        self.reserved_by_pos.insert(pos, unit);
        Ok(())
    }

    pub fn cancel_reservation(&mut self, unit: Uuid) {
        let Some(pos) = self.reserved_by_unit.remove(&unit) else {
            return;
        };
        self.reserved_by_pos.remove(&pos);
    }

    pub fn remove_unit(&mut self, unit: Uuid) {
        self.cancel_reservation(unit);
    }

    pub fn bfs_neighbors(pos: Position) -> [Position; 8] {
        // NW, N, NE, W, E, SW, S, SE
        [
            Position::new(pos.x - 1, pos.y - 1),
            Position::new(pos.x, pos.y - 1),
            Position::new(pos.x + 1, pos.y - 1),
            Position::new(pos.x - 1, pos.y),
            Position::new(pos.x + 1, pos.y),
            Position::new(pos.x - 1, pos.y + 1),
            Position::new(pos.x, pos.y + 1),
            Position::new(pos.x + 1, pos.y + 1),
        ]
    }

    pub fn bfs_map<P>(&self, start: Position, mut is_passable: P) -> Result<BfsMap, GameError>
    where
        P: FnMut(Position) -> bool,
    {
        let start_idx = self.idx(start)?;
        let len = (self.width as usize) * (self.height as usize);

        let mut dist: Vec<Option<u32>> = vec![None; len];
        let mut parent: Vec<Option<Position>> = vec![None; len];
        let mut queue: VecDeque<Position> = VecDeque::new();

        dist[start_idx] = Some(0);
        parent[start_idx] = None;
        queue.push_back(start);

        while let Some(current) = queue.pop_front() {
            let current_idx = match self.idx(current) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let Some(current_dist) = dist[current_idx] else {
                continue;
            };

            for next in Self::bfs_neighbors(current) {
                if !self.in_bounds(next) {
                    continue;
                }

                let next_idx = match self.idx(next) {
                    Ok(v) => v,
                    Err(_) => continue,
                };

                if dist[next_idx].is_some() {
                    continue;
                }

                if !is_passable(next) {
                    continue;
                }

                dist[next_idx] = Some(current_dist + 1);
                parent[next_idx] = Some(current);
                queue.push_back(next);
            }
        }

        Ok(BfsMap {
            width: self.width,
            height: self.height,
            dist,
            parent,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bfs_allows_corner_cutting() {
        let field = RuntimeField::new(4, 4);
        let bfs = field.bfs_map(Position::new(1, 1), |_| true).unwrap();

        assert_eq!(bfs.distance_to(Position::new(0, 0)), Some(1));
    }

    #[test]
    fn bfs_neighbor_order_is_deterministic_for_parents() {
        let field = RuntimeField::new(4, 4);
        let bfs = field.bfs_map(Position::new(1, 1), |_| true).unwrap();

        // For (2,0), the unique shortest is via NE from start (1,1)->(2,0).
        let path = bfs.reconstruct_path_to(Position::new(2, 0)).unwrap();
        assert_eq!(path, vec![Position::new(1, 1), Position::new(2, 0)]);
    }

    #[test]
    fn bfs_can_model_reserved_as_block_via_closure() {
        let mut field = RuntimeField::new(4, 4);
        let unit = Uuid::from_u128(1);
        field.reserve(unit, Position::new(2, 1)).unwrap();

        let bfs = field
            .bfs_map(Position::new(1, 1), |pos| {
                field.reserved_unit_at(pos).is_none()
            })
            .unwrap();

        assert_eq!(bfs.distance_to(Position::new(2, 1)), None);
    }
}
