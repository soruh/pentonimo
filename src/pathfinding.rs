use std::{collections::BinaryHeap, fmt::Debug};

use rustc_hash::FxHashMap;

use crate::tile_map::TileMap;

pub struct BfsScratch {
    shape: (u16, u16),
    visited: Vec<bool>,
    candidates_1: Vec<Point>,
    candidates_2: Vec<Point>,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Point(pub u16, pub u16);

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Path(pub Point, pub Point);

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
        debug_assert_eq!(self.shape, tile_map.shape);

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

                for (dx, dy) in OffsetIterator::default() {
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
        debug_assert_eq!(self.shape, tile_map.shape);

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

#[derive(Default)]
struct OffsetIterator(u8);

impl Iterator for OffsetIterator {
    type Item = (i32, i32);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // let res = match self.0 {
        //     0 => (-1, 0),
        //     1 => (1, 0),
        //     2 => (0, -1),
        //     3 => (0, 1),
        //     _ => {
        //         return None;
        //     }
        // };
        // self.0 += 1;
        // Some(res)

        if self.0 < 4 {
            let i = self.0 as usize;
            self.0 += 1;
            Some([(-1, 0), (1, 0), (0, -1), (0, 1)][i])
        } else {
            None
        }
    }
}

#[derive(Copy, Clone)]
struct Vertex {
    cost: u64,
    position: Point,
}
impl PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
    }
}
impl Eq for Vertex {}
impl PartialOrd for Vertex {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cost.cmp(&other.cost))
    }
}
impl Ord for Vertex {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.cost
            .cmp(&other.cost)
            .then_with(|| self.position.cmp(&other.position))
    }
}

// https://doc.rust-lang.org/std/collections/binary_heap/index.html
pub fn dijkstra(map: &TileMap, Path(start, goal): Path) -> Vec<Point> {
    let mut queue = BinaryHeap::new();
    let mut dist = FxHashMap::<Point, u64>::default();
    let mut prev = FxHashMap::<Point, Point>::default();

    dist.insert(start, 0);
    queue.push(Vertex {
        cost: 0,
        position: start,
    });

    while let Some(Vertex { cost, position }) = queue.pop() {
        // We have found a shortest path
        if position == goal {
            break;
        }

        // Important as we may have already found a better way
        if cost > dist[&position] {
            continue;
        }

        // For each node we can reach, see if we can find a way with
        // a lower cost going through this node
        for (dx, dy) in OffsetIterator::default() {
            let vx = position.0 as i32 + dx;
            let vy = position.1 as i32 + dy;

            if vx >= 0
                && vx < map.shape.0 as i32
                && vy >= 0
                && vy < map.shape.1 as i32
                && !map.get(vx as u16, vy as u16)
            {
                let next = Vertex {
                    cost: cost + 1,
                    position: Point(vx as u16, vy as u16),
                };

                // If so, add it to the frontier and continue
                if next.cost < dist.get(&next.position).copied().unwrap_or(u64::MAX) {
                    queue.push(next);
                    // Relaxation, we have now found a better way
                    dist.insert(next.position, next.cost);
                    prev.insert(next.position, position);
                }
            }
        }
    }

    let mut path = Vec::with_capacity(dist[&goal] as usize + 1);

    let mut p = goal;

    while p != start {
        path.push(p);
        p = prev[&p];
    }

    path.push(start);
    path.reverse();

    path
}
