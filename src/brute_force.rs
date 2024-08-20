use std::{
    ops::Deref,
    sync::{Arc, Mutex},
};

use rustc_hash::FxBuildHasher;
use strum::VariantArray;
use threadpool::ThreadPool;

use crate::{
    candidates::Candidates,
    pathfinding::BfsScratch,
    pentonimo::{Pentonimo, PentonimoKind, PositionedPentonimo},
    tile_map::TileMap,
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct StateKey {
    map: TileMap,
    available: Candidates,
}

impl StateKey {
    fn avaiable_variants(&self) -> impl Iterator<Item = Pentonimo> + '_ {
        PentonimoKind::VARIANTS
            .iter()
            .filter(|x| self.available.get(**x as u8) > 0)
            .flat_map(|kind| Pentonimo::new(*kind).variants())
    }
}

struct DfsState {
    scratch: BfsScratch,
    states: Arc<dashmap::DashMap<StateKey, u16, FxBuildHasher>>,
    buffer_capacity: usize,
    buffer_pool: Vec<Vec<PositionedPentonimo>>,
}

impl DfsState {
    fn get_buffer(&mut self) -> Vec<PositionedPentonimo> {
        if let Some(mut buffer) = self.buffer_pool.pop() {
            buffer.clear();
            buffer
        } else {
            Vec::with_capacity(self.buffer_capacity)
        }
    }
    fn return_buffer(&mut self, buffer: Vec<PositionedPentonimo>) {
        self.buffer_pool.push(buffer)
    }
    fn dfs(
        &mut self,
        key: StateKey,
        prev_diameter: u16,
        available: &[PositionedPentonimo],
    ) -> (u16, Vec<PositionedPentonimo>) {
        let (diameter, _) = self.scratch.graph_diameter(&key.map);

        if diameter < prev_diameter {
            // diameter decreased. Discard this branch
            return (diameter, vec![]);
        }

        let mut current_max = diameter;
        let mut placed = vec![];

        for &positioned in available {
            if key.map.can_place(positioned) {
                let mut map = key.map.clone();
                map |= positioned;
                let mut available_pieces = key.available;
                available_pieces.decrement(positioned.pentonimo().kind() as u8);

                // if mx == my also check for key.mirror_xy()
                let key = StateKey {
                    map,
                    available: available_pieces,
                };
                if !self.states.contains_key(&key) {
                    let mut new_available = self.get_buffer();
                    for &positioned in available {
                        if available_pieces.get(positioned.pentonimo().kind() as u8) > 0
                            && key.map.can_place(positioned)
                        {
                            new_available.push(positioned);
                        }
                    }

                    let (max_diameter, mut new_placed) = self.dfs(key, diameter, &new_available);

                    self.return_buffer(new_available);

                    if max_diameter > current_max {
                        current_max = max_diameter;
                        new_placed.push(positioned);
                        placed = new_placed;
                    }
                }
            }
        }

        self.states.insert(key, diameter);

        (current_max, placed)
    }
}

pub fn find_best(shape: (u16, u16)) -> (u16, Vec<PositionedPentonimo>) {
    let mut scratch = BfsScratch::new(shape);

    let map = TileMap::new(shape);
    let (diameter, _) = scratch.graph_diameter(&map);

    let mut available = Vec::new();
    let key = StateKey {
        map,
        available: Candidates::new([1; 12]),
    };
    for variant in key.avaiable_variants() {
        let (dx, dy) = variant.shape();

        let Some(px) = shape.0.checked_sub(dx as u16) else {
            continue;
        };
        let Some(py) = shape.1.checked_sub(dy as u16) else {
            continue;
        };

        for x in 0..px {
            for y in 0..py {
                let positioned = variant.position(x, y);
                debug_assert!(key.map.can_place(positioned));
                available.push(variant.position(x, y));
            }
        }
    }
    available.shrink_to_fit();

    let num_threads = (|| std::env::var("PENTONIMO_NUM_THREADS").ok()?.parse().ok())()
        .unwrap_or_else(num_cpus::get);
    let pool = ThreadPool::new(num_threads);

    let available = Arc::new(available);
    // dashmap ?
    let states = Arc::new(dashmap::DashMap::with_hasher_and_shard_amount(
        FxBuildHasher,
        num_threads.max(2),
    ));
    let results = Arc::new(Mutex::new(Vec::<(u16, Vec<PositionedPentonimo>)>::new()));

    available.clone().deref().iter().for_each(|&positioned| {
        let available = available.clone();
        let states = states.clone();
        let results = results.clone();
        pool.execute(move || {
            let mut state = DfsState {
                scratch: BfsScratch::new(shape),
                states,
                buffer_capacity: available.len(),
                buffer_pool: Vec::new(),
            };

            let mut map = TileMap::new(shape);

            map |= positioned;

            let mut key = StateKey {
                map,
                available: Candidates::new([1; 12]),
            };

            key.available.decrement(positioned.pentonimo().kind() as u8);

            let mut new_available = state.get_buffer();

            for &positioned in &*available {
                if key.available.get(positioned.pentonimo().kind() as u8) > 0
                    && key.map.can_place(positioned)
                {
                    new_available.push(positioned);
                }
            }

            let (max, mut placed) = state.dfs(key, diameter, &new_available);

            placed.push(positioned);

            // let mut dest = states.lock().unwrap();

            // for (key, value) in state.states {
            //     let old_value = *dest.entry(key).or_insert(value);
            //     debug_assert_eq!(old_value, value);
            // }

            results.lock().unwrap().push((max, placed));
        });
    });

    pool.join();

    Arc::into_inner(results)
        .unwrap()
        .into_inner()
        .unwrap()
        .into_iter()
        .max_by_key(|x| x.0)
        .unwrap_or((diameter, Vec::new()))
}
