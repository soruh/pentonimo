use std::{collections::HashSet, fmt::Display, ops::Range};

use strum::VariantArray;

use crate::tile::{Rotate, Tile};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::VariantArray)]
#[repr(u8)]
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

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pentonimo {
    kind: PentonimoKind,
    tile: Tile,
    bounds: PentonimoBounds,
}

#[derive(Clone, Copy)]
pub struct PositionedPentonimo {
    // normalized pentonimo
    pentonimo: Pentonimo,
    position: (u16, u16),
}

impl PositionedPentonimo {
    pub fn position(&self) -> (u16, u16) {
        self.position
    }

    pub fn shape(&self) -> (u8, u8) {
        self.pentonimo.shape()
    }

    pub fn pentonimo(&self) -> Pentonimo {
        self.pentonimo
    }
}

impl Pentonimo {
    pub fn kind(&self) -> PentonimoKind {
        self.kind
    }
    pub fn tile(self) -> Tile {
        self.tile
    }
    pub fn shape(&self) -> (u8, u8) {
        (
            self.bounds.end_x - self.bounds.start_x,
            self.bounds.end_y - self.bounds.start_y,
        )
    }

    pub fn position(self, x: u16, y: u16) -> PositionedPentonimo {
        PositionedPentonimo {
            pentonimo: self.normalize(),
            position: (x, y),
        }
    }

    pub fn variants(self) -> VariantIterator {
        // use symmetry of self.kind

        macro_rules! permutations {
            (FLIP_ROTATE, $self: ident) => {{
                let flipped = $self.flip_y();
                VariantIterator::Asymetric(
                    [
                        $self,
                        $self.rotate(Rotate::Right),
                        $self.rotate(Rotate::Left),
                        $self.rotate(Rotate::Full),
                        flipped,
                        flipped.rotate(Rotate::Right),
                        flipped.rotate(Rotate::Left),
                        flipped.rotate(Rotate::Full),
                    ]
                    .into_iter(),
                )
            }};
            (ROTATE, $self: ident) => {
                VariantIterator::Mirror(
                    [
                        $self,
                        $self.rotate(Rotate::Right),
                        $self.rotate(Rotate::Left),
                        $self.rotate(Rotate::Full),
                    ]
                    .into_iter(),
                )
            };
            (ROTATE_HALF, $self: ident) => {
                VariantIterator::HalfRotational([$self, $self.rotate(Rotate::Right)].into_iter())
            };
        }

        match self.kind {
            PentonimoKind::F => permutations!(FLIP_ROTATE, self),
            PentonimoKind::L => permutations!(FLIP_ROTATE, self),
            PentonimoKind::N => permutations!(FLIP_ROTATE, self),
            PentonimoKind::P => permutations!(FLIP_ROTATE, self),
            PentonimoKind::T => permutations!(ROTATE, self),
            PentonimoKind::U => permutations!(ROTATE, self),
            PentonimoKind::V => permutations!(ROTATE, self),
            PentonimoKind::W => permutations!(ROTATE, self),
            PentonimoKind::I => permutations!(ROTATE_HALF, self),
            PentonimoKind::X => VariantIterator::Rotational([self].into_iter()),
            PentonimoKind::Y => permutations!(FLIP_ROTATE, self),
            PentonimoKind::Z => permutations!(ROTATE, self),
        }
    }
}

pub enum VariantIterator {
    Rotational(std::array::IntoIter<Pentonimo, 1>),
    HalfRotational(std::array::IntoIter<Pentonimo, 2>),
    Mirror(std::array::IntoIter<Pentonimo, 4>),
    Asymetric(std::array::IntoIter<Pentonimo, 8>),
}

impl Iterator for VariantIterator {
    type Item = Pentonimo;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            VariantIterator::Rotational(inner) => inner.next(),
            VariantIterator::HalfRotational(inner) => inner.next(),
            VariantIterator::Mirror(inner) => inner.next(),
            VariantIterator::Asymetric(inner) => inner.next(),
        }
    }
}

#[test]
fn number_of_variants() {
    fn expected_count(kind: PentonimoKind) -> usize {
        match kind {
            PentonimoKind::F => 8,
            PentonimoKind::L => 8,
            PentonimoKind::N => 8,
            PentonimoKind::P => 8,
            PentonimoKind::T => 4,
            PentonimoKind::U => 4,
            PentonimoKind::V => 4,
            PentonimoKind::W => 4,
            PentonimoKind::I => 2,
            PentonimoKind::X => 1,
            PentonimoKind::Y => 8,
            PentonimoKind::Z => 4,
        }
    }

    assert_eq!(
        PentonimoKind::VARIANTS
            .iter()
            .copied()
            .map(expected_count)
            .sum::<usize>(),
        63
    );

    assert!(PentonimoKind::VARIANTS
        .iter()
        .all(|&kind| Pentonimo::new(dbg!(kind)).variants().count() == expected_count(kind)));

    let variants = PentonimoKind::VARIANTS
        .iter()
        .flat_map(|&kind| Pentonimo::new(dbg!(kind)).variants())
        .collect::<HashSet<Pentonimo>>();

    assert_eq!(variants.len(), 63);
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
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
        debug_assert!(x <= 8);
        debug_assert!(y <= 8);

        let x = x as i8 - 4;
        let y = y as i8 - 4;

        let (x, y) = match rotate {
            Rotate::Left => (y, -x),
            Rotate::Right => (-y, x),
            Rotate::Full => (-x, -y),
        };

        ((x + 4) as u8, (y + 4) as u8)
    }

    fn shift_x(self, d: i8) -> Self {
        Self {
            start_x: (self.start_x as i8 + d).clamp(0, 7) as u8,
            end_x: (self.end_x as i8 + d).clamp(0, 7) as u8,
            start_y: self.start_y,
            end_y: self.end_y,
        }
    }
    fn shift_y(self, d: i8) -> Self {
        Self {
            start_x: self.start_x,
            end_x: self.end_x,
            start_y: (self.start_y as i8 + d).clamp(0, 7) as u8,
            end_y: (self.end_y as i8 + d).clamp(0, 7) as u8,
        }
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

    #[inline]
    pub fn rotate(self, rotate: Rotate) -> Self {
        Self {
            kind: self.kind,
            bounds: self.bounds.rotate(rotate),
            tile: self.tile.rotate(rotate),
        }
    }
    #[inline]
    pub fn shift_x(self, d: i8) -> Self {
        Self {
            kind: self.kind,
            bounds: self.bounds.shift_x(d),
            tile: self.tile.shift_x(d),
        }
    }
    #[inline]
    pub fn shift_y(self, d: i8) -> Self {
        Self {
            kind: self.kind,
            bounds: self.bounds.shift_y(d),
            tile: self.tile.shift_y(d),
        }
    }
    #[inline]
    pub fn flip_x(self) -> Self {
        Self {
            kind: self.kind,
            tile: self.tile.flip_x(),
            bounds: self.bounds.flip_x(),
        }
    }
    #[inline]
    pub fn flip_y(self) -> Self {
        Self {
            kind: self.kind,
            tile: self.tile.flip_y(),
            bounds: self.bounds.flip_y(),
        }
    }
    #[inline]
    pub fn flip_xy(self) -> Self {
        Self {
            kind: self.kind,
            tile: self.tile.flip_xy(),
            bounds: self.bounds.flip_xy(),
        }
    }
    #[inline]
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
    #[inline]
    pub fn is_empty(self) -> bool {
        self.tile.is_empty()
    }
    #[inline]
    pub fn shift_split(self, x: i8, y: i8) -> [Tile; 4] {
        [(0, 0), (1, 0), (0, 1), (1, 1)].map(|(i, j)| {
            let dx = x - 8 * i;
            let dy = y - 8 * j;

            let shifted_bounds = self.bounds.shift_x(dx).shift_y(dy);
            let wx = shifted_bounds.end_x - shifted_bounds.start_x;
            let wy = shifted_bounds.end_y - shifted_bounds.start_y;

            if wx > 0 && wy > 0 {
                self.tile.shift_x(dx).shift_y(dy)
            } else {
                Tile::empty()
            }
        })
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
