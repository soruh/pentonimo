use std::{
    fmt::Display,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not},
};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Tile(pub u64);

impl Tile {
    pub fn empty() -> Self {
        Self(0)
    }
    pub fn full() -> Self {
        Self(u64::MAX)
    }
    pub fn get(&self, x: u8, y: u8) -> bool {
        let index = 8 * y + x;
        (self.0 >> index) & 1 == 1
    }
    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    #[must_use]
    pub fn shift_x(self, d: i8) -> Self {
        let mut bytes = self.0.to_le_bytes();

        if d > 0 {
            let d = d as u32;
            for byte in &mut bytes {
                *byte <<= d;
            }
        } else {
            let d = (-d) as u32;
            for byte in &mut bytes {
                *byte >>= d;
            }
        }

        Self(u64::from_le_bytes(bytes))
    }

    #[must_use]
    pub fn shift_y(self, d: i8) -> Self {
        if d > 0 {
            let d = d as u32 * 8;
            Self(self.0 << d)
        } else {
            let d = (-d) as u32 * 8;
            Self(self.0 >> d)
        }
    }

    pub fn flip_xy(self) -> Self {
        Self(self.0.reverse_bits())
    }

    pub fn flip_x(self) -> Self {
        let mut bytes = self.0.to_le_bytes();
        for byte in &mut bytes {
            *byte = byte.reverse_bits();
        }
        Self(u64::from_le_bytes(bytes))
    }

    pub fn flip_y(self) -> Self {
        let mut bytes = self.0.to_le_bytes();
        bytes.reverse();
        Self(u64::from_le_bytes(bytes))
    }

    pub fn transpose(self) -> Self {
        let mut grid = self.0;

        // Swap 1-bit groups (2x2 block swap)
        let t = (grid ^ (grid >> 7)) & 0x00AA00AA00AA00AA;
        grid = grid ^ t ^ (t << 7);

        // Swap 2-bit groups (4x4 block swap)
        let t = (grid ^ (grid >> 14)) & 0x0000CCCC0000CCCC;
        grid = grid ^ t ^ (t << 14);

        // Swap 4-bit groups (8x8 block swap)
        let t = (grid ^ (grid >> 28)) & 0x00000000F0F0F0F0;
        grid = grid ^ t ^ (t << 28);

        Self(grid)
    }

    pub fn rotate(self, rotate: Rotate) -> Self {
        match rotate {
            Rotate::Left => self.transpose().flip_y(),
            Rotate::Right => self.transpose().flip_x(),
            Rotate::Full => self.flip_xy(),
        }
    }

    pub fn fill_bottom(n_rows: u8) -> Self {
        const FIRST_ROW: u64 = 0xff000000_00000000;

        let mut res = 0;
        for i in 0..n_rows {
            res |= FIRST_ROW >> (i * 8);
        }
        Self(res)
    }

    pub fn fill_right(n_cols: u8) -> Self {
        const FIRST_COLUMN: u64 = 0x80808080_80808080;

        let mut res = 0;
        for i in 0..n_cols {
            res |= FIRST_COLUMN >> i;
        }
        Self(res)
    }
}

impl BitAnd for Tile {
    type Output = Tile;

    fn bitand(self, rhs: Tile) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for Tile {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitOr for Tile {
    type Output = Tile;

    fn bitor(self, rhs: Tile) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for Tile {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl Not for Tile {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

#[derive(Clone, Copy)]
pub enum Rotate {
    Left,  // -90
    Right, // +90
    Full,  // 180
}

impl Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..8 {
            for x in 0..8 {
                let is_set = self.get(x, y);
                write!(f, "{}", if is_set { 'x' } else { '.' })?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {

    use super::Tile;

    #[test]
    fn shift() {
        let full = Tile::full();

        for i in 0..8 {
            assert_ne!(full.shift_x(i).0, 0);
            assert_ne!(full.shift_x(-i).0, 0);
            assert_ne!(full.shift_y(i).0, 0);
            assert_ne!(full.shift_y(-i).0, 0);
        }
    }
}
