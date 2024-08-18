use rustc_hash::FxHashMap;
use strum::VariantArray;

use crate::{
    candidates::Candidates,
    pathfinding::BfsScratch,
    pentonimo::{self, Pentonimo, PentonimoKind},
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

pub fn find_best(shape: (u16, u16)) {
    let mut scratch = BfsScratch::new(shape);

    let mut states: FxHashMap<StateKey, u16> = FxHashMap::default();

    let mut state = StateKey {
        map: TileMap::new(shape),
        available: Candidates::new([1; 12]),
    };

    //
}
