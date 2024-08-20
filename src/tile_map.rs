use std::{
    fmt::Display,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign},
};

use smallvec::SmallVec;

use crate::{pentonimo::PositionedPentonimo, tile::Tile};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct TileMap {
    pub tiles: SmallVec<[Tile; 1]>,
    pub shape: (u16, u16),
}

impl Display for TileMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tile_shape = self.tile_shape();

        let (x_max, y_max) = if f.alternate() {
            (8 * tile_shape.1 as u16, 8 * tile_shape.0 as u16)
        } else {
            (self.shape.0, self.shape.1)
        };

        for y in 0..y_max {
            for x in 0..x_max {
                let tile_x = (x / 8) as usize;
                let tile_y = (y / 8) as usize;

                let tile = self.tiles[tile_y * tile_shape.0 + tile_x];

                let isset = tile.get((x % 8) as u8, (y % 8) as u8);

                if f.alternate() {
                    let inbounds = x < self.shape.0 && y < self.shape.1;
                    let c = if inbounds { "" } else { "30" };
                    write!(f, "\x1b[{c}m{}", if isset { 'x' } else { '.' })?;
                } else {
                    write!(f, "{}", if isset { 'x' } else { '.' })?;
                }
            }
            writeln!(f, "\x1b[m")?;
        }
        Ok(())
    }
}

#[inline]
fn div_ceil(a: u16, b: u16) -> u16 {
    (a + b - 1) / b
}

impl TileMap {
    pub fn tile_shape(&self) -> (usize, usize) {
        (
            div_ceil(self.shape.0, 8) as usize,
            div_ceil(self.shape.1, 8) as usize,
        )
    }
    pub fn new(shape: (u16, u16)) -> Self {
        let tile_shape = (div_ceil(shape.0, 8) as usize, div_ceil(shape.1, 8) as usize);

        let mut tiles = vec![Tile(0); tile_shape.0 * tile_shape.1];

        // fill the remainder of cells with ones
        let remainder = (shape.0 % 8, shape.1 % 8);

        if remainder.1 > 0 {
            // iterate over bottom-most row and fill leftover cells
            let block_bottom = Tile::fill_bottom(8 - remainder.1 as u8);
            for x in 0..tile_shape.0 {
                let index = (tile_shape.1 - 1) * tile_shape.0 as usize + x;
                tiles[index] |= block_bottom;
            }
        }

        if remainder.0 > 0 {
            // iterate over right-most column and fill leftover cells
            let block_right = Tile::fill_right(8 - remainder.0 as u8);
            for y in 0..tile_shape.1 {
                let index = y * tile_shape.0 as usize + tile_shape.0 - 1;
                tiles[index] |= block_right;
            }
        }

        Self {
            tiles: tiles.into(),
            shape,
        }
    }

    #[inline]
    pub fn can_place(&self, rhs: PositionedPentonimo) -> bool {
        let (x, y) = rhs.position();
        let parts = rhs.pentonimo().shift_split((x % 8) as i8, (y % 8) as i8);

        let (tx, ty) = (x / 8, y / 8);

        for (i, part) in parts.iter().enumerate() {
            if !part.is_empty() {
                let dx = i % 2;
                let dy = i / 2;

                if !(self.get_tile(tx + dx as u16, ty + dy as u16) & *part).is_empty() {
                    return false;
                }
            }
        }

        true
    }

    #[inline]
    pub fn get_tile(&self, tx: u16, ty: u16) -> Tile {
        let tile_index = tx as usize + (ty as usize) * self.tile_shape().0;
        self.tiles[tile_index]
    }

    #[inline]
    pub fn get(&self, x: u16, y: u16) -> bool {
        let (tx, ty) = (x / 8, y / 8);
        let (x, y) = (x % 8, y % 8);

        self.get_tile(tx, ty).get(x as u8, y as u8)
    }
}

impl BitOrAssign<PositionedPentonimo> for TileMap {
    #[inline]
    fn bitor_assign(&mut self, rhs: PositionedPentonimo) {
        let (x, y) = rhs.position();
        let parts = rhs.pentonimo().shift_split((x % 8) as i8, (y % 8) as i8);

        let (tx, ty) = (x / 8, y / 8);

        for (i, part) in parts.iter().enumerate() {
            if !part.is_empty() {
                let dx = i % 2;
                let dy = i / 2;
                let tile_index = tx as usize + dx + (ty as usize + dy) * self.tile_shape().0;

                self.tiles[tile_index] |= *part;
            }
        }
    }
}

impl BitAndAssign<PositionedPentonimo> for TileMap {
    #[inline]
    fn bitand_assign(&mut self, rhs: PositionedPentonimo) {
        let (x, y) = rhs.position();
        let parts = rhs.pentonimo().shift_split((x % 8) as i8, (y % 8) as i8);

        let (tx, ty) = (x / 8, y / 8);

        for (i, part) in parts.iter().enumerate() {
            if !part.is_empty() {
                let dx = i % 2;
                let dy = i / 2;
                let tile_index = tx as usize + dx + (ty as usize + dy) * self.tile_shape().0;

                self.tiles[tile_index] &= *part;
            }
        }
    }
}

impl BitAnd<PositionedPentonimo> for TileMap {
    type Output = TileMap;

    #[inline]
    fn bitand(mut self, rhs: PositionedPentonimo) -> Self::Output {
        self &= rhs;
        self
    }
}

impl BitOr<PositionedPentonimo> for TileMap {
    type Output = TileMap;

    #[inline]
    fn bitor(mut self, rhs: PositionedPentonimo) -> Self::Output {
        self |= rhs;
        self
    }
}
