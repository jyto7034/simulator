use std::collections::VecDeque;

use crate::{
    game::resources::Position,
    game::{
        battle::battlefield::{Battlefield, Tile},
        behavior::GameError,
        enums::Side,
    },
};

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
        self.dist[dest_idx]?;

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

impl Battlefield {
    fn is_bfs_passable<P>(&self, pos: Position, is_passable: &mut P) -> bool
    where
        P: FnMut(Position, &Tile) -> bool,
    {
        if !self.in_bounds(pos) {
            return false;
        }
        let Ok(idx) = self.idx(pos) else {
            return false;
        };
        is_passable(pos, &self.tiles[idx])
    }

    fn bfs_neighbor_idx<P>(
        &self,
        current: Position,
        next: Position,
        is_passable: &mut P,
    ) -> Option<usize>
    where
        P: FnMut(Position, &Tile) -> bool,
    {
        if !self.is_bfs_passable(next, is_passable) {
            return None;
        }

        let dx = next.x - current.x;
        let dy = next.y - current.y;
        if dx.abs() == 1 && dy.abs() == 1 {
            let horizontal = Position::new(next.x, current.y);
            let vertical = Position::new(current.x, next.y);
            if !self.is_bfs_passable(horizontal, is_passable)
                || !self.is_bfs_passable(vertical, is_passable)
            {
                return None;
            }
        }

        self.idx(next).ok()
    }

    fn neighbors_8_for_side(pos: Position, side: Side) -> [Position; 8] {
        match side {
            Side::Player => [
                Position::new(pos.x, pos.y - 1),
                Position::new(pos.x, pos.y + 1),
                Position::new(pos.x - 1, pos.y),
                Position::new(pos.x + 1, pos.y),
                Position::new(pos.x - 1, pos.y - 1),
                Position::new(pos.x + 1, pos.y - 1),
                Position::new(pos.x - 1, pos.y + 1),
                Position::new(pos.x + 1, pos.y + 1),
            ],
            Side::Opponent => [
                Position::new(pos.x, pos.y + 1),
                Position::new(pos.x, pos.y - 1),
                Position::new(pos.x - 1, pos.y),
                Position::new(pos.x + 1, pos.y),
                Position::new(pos.x - 1, pos.y + 1),
                Position::new(pos.x + 1, pos.y + 1),
                Position::new(pos.x - 1, pos.y - 1),
                Position::new(pos.x + 1, pos.y - 1),
            ],
        }
    }

    pub fn bfs_map_8<P>(&self, start: Position, mut is_passable: P) -> Result<BfsMap, GameError>
    where
        P: FnMut(Position, &Tile) -> bool,
    {
        let start_idx = self.idx(start)?;
        let len = self.tiles.len();

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

            for next in Self::neighbors_8(current) {
                let Some(next_idx) = self.bfs_neighbor_idx(current, next, &mut is_passable) else {
                    continue;
                };
                if dist[next_idx].is_some() {
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

    pub fn bfs_map_8_for_side<P>(
        &self,
        start: Position,
        side: Side,
        mut is_passable: P,
    ) -> Result<BfsMap, GameError>
    where
        P: FnMut(Position, &Tile) -> bool,
    {
        let start_idx = self.idx(start)?;
        let len = self.tiles.len();

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

            for next in Self::neighbors_8_for_side(current, side) {
                let Some(next_idx) = self.bfs_neighbor_idx(current, next, &mut is_passable) else {
                    continue;
                };
                if dist[next_idx].is_some() {
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
