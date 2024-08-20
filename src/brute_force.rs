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
}

impl DfsState {
    fn bfs(&mut self, key: StateKey, prev_diameter: u16) -> (u16, Vec<PositionedPentonimo>) {
        let (diameter, _) = self.scratch.graph_diameter(&key.map);

        if diameter < prev_diameter {
            // diameter decreased. Discard this branch
            return (diameter, vec![]);
        }

        let (mx, my) = key.map.shape;

        let mut current_max = diameter;
        let mut placed = vec![];

        for variant in key.avaiable_variants() {
            let (dx, dy) = variant.shape();

            let Some(px) = mx.checked_sub(dx as u16) else {
                continue;
            };
            let Some(py) = my.checked_sub(dy as u16) else {
                continue;
            };

            for x in 0..px {
                for y in 0..py {
                    let positioned = variant.position(x, y);

                    if key.map.can_place(positioned) {
                        let mut map = key.map.clone();
                        map |= positioned;
                        let mut available = key.available;
                        available.decrement(positioned.pentonimo().kind() as u8);

                        // if mx == my also check for key.mirror_xy()
                        let key = StateKey { map, available };
                        if !self.states.contains_key(&key) {
                            let (max_diameter, mut new_placed) = self.bfs(key, diameter);

                            if max_diameter > current_max {
                                current_max = max_diameter;
                                new_placed.push(positioned);
                                placed = new_placed;
                            }
                        }
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

    let mut state = DfsState { scratch, states };

    let key = StateKey {
        map,
        available: Candidates::new([1; 12]),
    };
    let (max, placed) = state.bfs(key, diameter);

    assert_eq!(max, *state.states.values().max().unwrap());

    (max, placed)
}
