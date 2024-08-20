use std::fmt::Display;

use brute_force::find_best;
use candidates::Candidates;
use pathfinding::BfsScratch;
use pentonimo::{Pentonimo, PentonimoKind, PositionedPentonimo};
use strum::VariantArray;
use tile_map::TileMap;

mod brute_force;
mod candidates;
mod pathfinding;
mod pentonimo;
mod tile;
mod tile_map;

fn main() {
    // let pentonimo = Pentonimo::new(PentonimoKind::W)
    //     .shift_x(1)
    //     .unwrap()
    //     .shift_y(1)
    //     .unwrap();

    // println!("{}", pentonimo.normalize());
    // println!("{}", pentonimo.flip_x().normalize());
    // println!("{}", pentonimo.flip_y().normalize());
    // println!("{}", pentonimo.flip_xy().normalize());
    // println!("{}", pentonimo.rotate(Rotate::Right).normalize());
    // println!("{}", pentonimo.rotate(Rotate::Left).normalize());
    // println!("{}", pentonimo.rotate(Rotate::Full).normalize());

    let mut map = TileMap::new((10, 10));

    let pentonimo = Pentonimo::new(PentonimoKind::F).shift_x(1).shift_y(1);

    println!("{}", pentonimo.normalize());
    println!("{}", pentonimo.normalize().flip_x());
    println!(
        "{}",
        pentonimo.normalize().flip_x().rotate(tile::Rotate::Left)
    );

    assert!(map.can_place(pentonimo.position(0, 0)));
    map |= pentonimo.position(0, 0);

    assert!(!map.can_place(pentonimo.position(0, 1)));

    println!("{map}");

    assert!(map.can_place(pentonimo.position(7, 7)));
    map |= pentonimo.position(7, 7);

    println!("{map}");

    let mut scratch = BfsScratch::new(map.shape);

    for y in 0..map.shape.0 {
        for x in 0..map.shape.1 {
            let (e, _) = scratch.eccentricity(&map, x, y);
            print!("\x1b[{}m{:2}\x1b[m ", 17 + e, e);
        }
        println!()
    }

    dbg!(scratch.graph_diameter(&map));

    let max = 6;
    for x in 3..=max {
        for y in 3..=max {
            if x >= y {
                compute_and_print(x, y);
            }
        }
    }
}

fn compute_and_print(mx: u16, my: u16) {
    res_to_svg((mx, my), find_best((mx, my)));
}

// todo: produce nicer pictures (svg?)
fn print_res((mx, my): (u16, u16), (max, tiles): (u16, Vec<PositionedPentonimo>)) {
    println!("({mx},{my}): {max}");

    println!(
        "tiles: {:?}",
        tiles
            .iter()
            .map(|x| x.pentonimo().kind())
            .collect::<Vec<_>>()
    );

    let mut map = TileMap::new((mx, my));

    for &tile in &tiles {
        assert!(map.can_place(tile));

        map |= tile;
    }

    let mut scratch = BfsScratch::new((mx, my));

    let (new_max, path) = scratch.graph_diameter(&map);
    assert_eq!(new_max, max);
    println!("{path:?}");

    println!("{map}",);
}

// todo: produce nicer pictures (svg?)
fn res_to_svg((mx, my): (u16, u16), (max, tiles): (u16, Vec<PositionedPentonimo>)) {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Value {
        Pentonimo(PentonimoKind),
        Nothing,
        Path(usize),
    }

    impl Display for Value {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Value::Pentonimo(kind) => {
                    let color = match kind {
                        PentonimoKind::F => 31,
                        PentonimoKind::L => 32,
                        PentonimoKind::N => 33,
                        PentonimoKind::P => 34,
                        PentonimoKind::T => 35,
                        PentonimoKind::U => 36,
                        PentonimoKind::V => 41,
                        PentonimoKind::W => 42,
                        PentonimoKind::I => 43,
                        PentonimoKind::X => 44,
                        PentonimoKind::Y => 45,
                        PentonimoKind::Z => 46,
                    };

                    write!(f, "\x1b[{color}mxx\x1b[m")
                }
                Value::Nothing => write!(f, ".."),
                Value::Path(n) => write!(f, "{n:2}"),
            }
        }
    }

    let path = {
        let mut map = TileMap::new((mx, my));
        for &tile in &tiles {
            assert!(map.can_place(tile));
            map |= tile;
        }
        let mut scratch = BfsScratch::new((mx, my));
        let (new_max, path) = scratch.graph_diameter(&map);
        assert_eq!(new_max, max);
        path
    };

    let mut grid = vec![Value::Nothing; mx as usize * my as usize];

    for tile in &tiles {
        for x in 0..8 {
            for y in 0..8 {
                let px = tile.position().0 + x as u16;
                let py = tile.position().1 + y as u16;
                // TODO: this is wrong

                if px < mx && py < my && tile.pentonimo().tile().get(x, y) {
                    let index = mx as usize * y as usize + x as usize;
                    assert_eq!(grid[index], Value::Nothing);
                    grid[index] = Value::Pentonimo(tile.pentonimo().kind());
                }
            }
        }
    }

    println!("({mx},{my}): {max}");
    for y in 0..my {
        for x in 0..mx {
            print!("{} ", grid[mx as usize * y as usize + x as usize]);
        }
        println!();
    }
}
