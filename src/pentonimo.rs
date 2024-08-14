use std::{fmt::Display, ops::Range, str::RSplitTerminator};

use crate::tile::{Rotate, Tile};

#[derive(Debug, Clone, Copy, strum::VariantArray)]
pub enum PentonimoKind {
    F,
    L,
    N,
    P,
    T,
    U,
    V,
    W,
    I,
    X,
    Y,
    Z,
}

#[derive(Clone, Copy)]
pub struct Pentonimo {
    kind: PentonimoKind,
    tile: Tile,
    bounds: PentonimoBounds,
}

#[derive(Clone, Copy)]
struct PentonimoBounds {
    start_x: u8,
    end_x: u8,
    start_y: u8,
    end_y: u8,
}

impl PentonimoBounds {
    pub const fn new(x: Range<u8>, y: Range<u8>) -> Self {
        Self {
            start_x: x.start,
            end_x: x.end,
            start_y: y.start,
            end_y: y.end,
        }
    }
    fn range_x(&self) -> Range<u8> {
        self.start_x..self.end_x
    }
    fn range_y(&self) -> Range<u8> {
        self.start_y..self.end_y
    }
    pub fn contains(&self, x: u8, y: u8) -> bool {
        self.range_x().contains(&x) && self.range_y().contains(&y)
    }
    fn checked_add(coordinate: u8, d: i8) -> Option<u8> {
        let res = coordinate as i16 + d as i16;
        (0..8).contains(&res).then_some(res as u8)
    }
    fn rotate_point(x: u8, y: u8, rotate: Rotate) -> (u8, u8) {
        debug_assert!(x < 8);
        debug_assert!(y < 8);

        let x = x as i8 - 4;
        let y = y as i8 - 4;

        let (x, y) = match rotate {
            Rotate::Left => (y, -x),
            Rotate::Right => (-y, x),
            Rotate::Full => (-x, -y),
        };

        ((x + 4) as u8, (y + 4) as u8)
    }

    fn shift_x(self, d: i8) -> Option<Self> {
        Some(Self {
            start_x: Self::checked_add(self.start_x, d)?,
            end_x: Self::checked_add(self.end_x, d)?,
            start_y: self.start_y,
            end_y: self.end_y,
        })
    }
    fn shift_y(self, d: i8) -> Option<Self> {
        Some(Self {
            start_x: self.start_x,
            end_x: self.end_x,
            start_y: Self::checked_add(self.start_y, d)?,
            end_y: Self::checked_add(self.end_y, d)?,
        })
    }

    fn flip_x(self) -> Self {
        Self {
            start_x: 8 - self.end_x,
            end_x: 8 - self.start_x,
            start_y: self.start_y,
            end_y: self.end_y,
        }
    }
    fn flip_y(self) -> Self {
        Self {
            start_x: self.start_x,
            end_x: self.end_x,
            start_y: 8 - self.end_y,
            end_y: 8 - self.start_y,
        }
    }
    fn flip_xy(self) -> Self {
        self.flip_x().flip_y()
    }

    fn rotate(self, rotate: Rotate) -> Self {
        let (x1, y1) = Self::rotate_point(self.start_x, self.start_y, rotate);
        let (x2, y2) = Self::rotate_point(self.end_x, self.end_y, rotate);

        Self {
            start_x: x1.min(x2),
            end_x: x1.max(x2),
            start_y: y1.min(y2),
            end_y: y1.max(y2),
        }
    }
    fn normalize(self) -> Self {
        Self {
            start_x: 0,
            end_x: self.end_x - self.start_x,
            start_y: 0,
            end_y: self.end_y - self.start_y,
        }
    }
}

impl Pentonimo {
    pub fn new(kind: PentonimoKind) -> Self {
        let (tile, bounds) = kind.generator_tile();
        Self { kind, tile, bounds }
    }

    pub fn rotate(self, rotate: Rotate) -> Self {
        Self {
            kind: self.kind,
            bounds: self.bounds.rotate(rotate),
            tile: self.tile.rotate(rotate),
        }
    }
    pub fn shift_x(self, d: i8) -> Option<Self> {
        Some(Self {
            kind: self.kind,
            bounds: self.bounds.shift_x(d)?,
            tile: self.tile.shift_x(d),
        })
    }
    pub fn shift_y(self, d: i8) -> Option<Self> {
        Some(Self {
            kind: self.kind,
            bounds: self.bounds.shift_y(d)?,
            tile: self.tile.shift_y(d),
        })
    }
    pub fn flip_x(self) -> Self {
        Self {
            kind: self.kind,
            tile: self.tile.flip_x(),
            bounds: self.bounds.flip_x(),
        }
    }
    pub fn flip_y(self) -> Self {
        Self {
            kind: self.kind,
            tile: self.tile.flip_y(),
            bounds: self.bounds.flip_y(),
        }
    }
    pub fn flip_xy(self) -> Self {
        Self {
            kind: self.kind,
            tile: self.tile.flip_xy(),
            bounds: self.bounds.flip_xy(),
        }
    }
    pub fn normalize(self) -> Self {
        Self {
            kind: self.kind,
            tile: self
                .tile
                .shift_x(-(self.bounds.start_x as i8))
                .shift_y(-(self.bounds.start_y as i8)),
            bounds: self.bounds.normalize(),
        }
    }
}

impl Display for Pentonimo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..8 {
            for x in 0..8 {
                let inbounds = self.bounds.contains(x, y);
                let c = if inbounds { "33" } else { "" };

                write!(
                    f,
                    "\x1b[{c}m{}",
                    if self.tile.get(x, y) { 'x' } else { '.' }
                )?;
            }

            writeln!(f, "\x1b[m")?;
        }
        Ok(())
    }
}

const fn pentonimo(positions: [&[u8; 5]; 5]) -> Tile {
    let mut grid = 0;

    let mut y = 0;

    while y < 5 {
        let row = positions[y];

        let mut x = 0;
        while x < 5 {
            let pos = row[x];

            if pos == b'x' {
                let index = 8 * y + x;
                grid |= 1 << index;
            } else if pos != b'.' {
                panic!("tried to created a pentonimo containing an invalid character");
            }

            x += 1;
        }

        y += 1;
    }

    Tile(grid)
}

impl PentonimoKind {
    #[rustfmt::skip]
    const fn generator_tile(self) -> (Tile, PentonimoBounds) {
        match self {
            PentonimoKind::F => const { (
                pentonimo([
                    b".xx..",
                    b"xx...",
                    b".x...",
                    b".....",
                    b".....",
                ]),
                PentonimoBounds::new(0..3, 0..3),
            )},
            PentonimoKind::L => const { (
                pentonimo([
                    b"x....",
                    b"x....",
                    b"x....",
                    b"xx...",
                    b".....",
                ]),
                PentonimoBounds::new(0..2, 0..4),
            )},
            PentonimoKind::N => const { (
                pentonimo([
                    b"x....",
                    b"x....",
                    b"xx...",
                    b".x...",
                    b".....",
                ]),
                PentonimoBounds::new(0..2, 0..4),
            )},
            PentonimoKind::P => const { (
                pentonimo([
                    b"xx...",
                    b"xx...",
                    b"x....",
                    b".....",
                    b".....",
                ]),
                PentonimoBounds::new(0..2, 0..3),
            )},
            PentonimoKind::T => const { (
                pentonimo([
                    b"xxx..",
                    b".x...",
                    b".x...",
                    b".....",
                    b".....",
                ]),
                PentonimoBounds::new(0..3, 0..3),
            )},
            PentonimoKind::U => const { (
                pentonimo([
                    b"x.x..",
                    b"xxx..",
                    b".....",
                    b".....",
                    b".....",
                ]),
                PentonimoBounds::new(0..3, 0..2),
            )},
            PentonimoKind::V => const { (
                pentonimo([
                    b"x....",
                    b"x....",
                    b"xxx..",
                    b".....",
                    b".....",
                ]),
                PentonimoBounds::new(0..3, 0..3),
            )},
            PentonimoKind::W => const { (
                pentonimo([
                    b"x....",
                    b"xx...",
                    b".xx..",
                    b".....",
                    b".....",
                ]),
                PentonimoBounds::new(0..3, 0..3),
            )},
            PentonimoKind::I => const { (
                pentonimo([
                    b"x....",
                    b"x....",
                    b"x....",
                    b"x....",
                    b"x....",
                ]),
                PentonimoBounds::new(0..1, 0..5),
            )},
            PentonimoKind::X => const { (
                pentonimo([
                    b".x...",
                    b"xxx..",
                    b".x...",
                    b".....",
                    b".....",
                ]),
                PentonimoBounds::new(0..3, 0..3),
            )},
            PentonimoKind::Y => const { (
                pentonimo([
                    b"x....",
                    b"xx...",
                    b"x....",
                    b"x....",
                    b".....",
                ]),
                PentonimoBounds::new(0..2, 0..4),
            )},
            PentonimoKind::Z => const { (
                pentonimo([
                    b"xx...",
                    b".x...",
                    b".xx..",
                    b".....",
                    b".....",
                ]),
                PentonimoBounds::new(0..3, 0..3),
            )},
        }
    }
}
