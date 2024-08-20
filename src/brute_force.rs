use rustc_hash::FxHashMap;
use strum::VariantArray;

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
    states: FxHashMap<StateKey, u16>,
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

    let states: FxHashMap<StateKey, u16> = FxHashMap::default();

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
                available.push(variant.position(x, y));
            }
        }
    }
    available.shrink_to_fit();

    let mut state = DfsState {
        scratch,
        states,
        buffer_capacity: available.len(),
        buffer_pool: Vec::new(),
    };

    let (max, placed) = state.dfs(key, diameter, &available);

    assert_eq!(max, *state.states.values().max().unwrap());

    (max, placed)
}
