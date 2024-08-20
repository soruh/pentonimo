use std::fmt::{Debug, Display};

use rustc_hash::{FxBuildHasher, FxHashSet};

use crate::tile_map::TileMap;

pub struct BfsScratch {
    shape: (u16, u16),
    visited: Vec<bool>,
    candidates_1: Vec<Point>,
    candidates_2: Vec<Point>,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct Point(u16, u16);

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct Path(Point, Point);

impl Debug for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}:{:?}", self.0, self.1)
    }
}

impl Debug for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} -> {:?}", self.0, self.1)
    }
}

impl BfsScratch {
    pub fn new(shape: (u16, u16)) -> Self {
        let max_width = shape.0.max(shape.1);
        // 1 4 8 12 16
        let max_candidates = (max_width as usize / 2 * 4).max(1);
        Self {
            shape,
            visited: vec![false; shape.0 as usize * shape.1 as usize],
            candidates_1: Vec::with_capacity(max_candidates),
            candidates_2: Vec::with_capacity(max_candidates),
        }
    }

    // bfs search to find eccentricity
    pub fn eccentricity(&mut self, tile_map: &TileMap, x: u16, y: u16) -> (u16, Point) {
        assert_eq!(self.shape, tile_map.shape);

        let start = Point(x, y);

        if tile_map.get(x, y) {
            return (0, start);
        }

        self.candidates_1.clear();
        self.candidates_1.push(start);

        #[inline]
        fn index_for_point(shape: (u16, u16), p: Point) -> usize {
            p.0 as usize + p.1 as usize * shape.0 as usize
        }

        self.visited.fill(false);
        self.visited[index_for_point(self.shape, start)] = true;

        let mut prev = start;

        for i in 0u16.. {
            if self.candidates_1.is_empty() {
                return (i, prev);
            }

            self.candidates_2.clear();

            for &candidate in &self.candidates_1 {
                self.visited[index_for_point(self.shape, candidate)] = true;

                let dxs = (-1..=1).step_by(2).map(|d| (d, 0));
                let dys = (-1..=1).step_by(2).map(|d| (0, d));

                for (dx, dy) in dxs.chain(dys) {
                    let x = candidate.0 as i32 + dx;
                    let y = candidate.1 as i32 + dy;

                    if x < 0 || x >= tile_map.shape.0 as i32 {
                        continue;
                    }
                    if y < 0 || y >= tile_map.shape.1 as i32 {
                        continue;
                    }

                    let p = Point(x as u16, y as u16);

                    if !tile_map.get(p.0, p.1) && !self.visited[index_for_point(self.shape, p)] {
                        self.candidates_2.push(p);
                        prev = p;
                    }
                }
            }

            std::mem::swap(&mut self.candidates_1, &mut self.candidates_2);
        }

        unreachable!("eccentricity > u16::MAX")
    }

    pub fn graph_diameter(&mut self, tile_map: &TileMap) -> (u16, Path) {
        assert_eq!(self.shape, tile_map.shape);

        let mut maximum = None;
        let mut max_coords = None;

        for y in 0..tile_map.shape.0 {
            for x in 0..tile_map.shape.1 {
                let (e, end) = self.eccentricity(tile_map, x, y);

                if maximum.is_none() || e > maximum.unwrap() {
                    maximum = Some(e);
                    max_coords = Some(Path(Point(x, y), end));
                }
            }
        }

        (maximum.unwrap(), max_coords.unwrap())
    }
}
