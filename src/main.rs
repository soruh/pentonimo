use std::{fmt::Display, fs::File, io::Write, path::PathBuf};

use brute_force::find_best;
use candidates::Candidates;
use pathfinding::{dijkstra, BfsScratch};
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

    _ = std::fs::create_dir("results");

    let max = 6;
    for x in 3..=max {
        for y in 3..=max {
            if x >= y {
                let (max, tiles) = find_best((x, y));
                let grid = build_print_map((x, y), (max, tiles));
                ConsolePrinter.print((x, y), max, &grid);
                SvgPrinter(PathBuf::from(format!("results/{x}_{y}.svg"))).print((x, y), max, &grid);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PrintValue {
    Pentonimo(PentonimoKind),
    Nothing,
    Path(usize),
}

impl Display for PrintValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrintValue::Pentonimo(kind) => {
                let color = match kind {
                    PentonimoKind::F => 31,
                    PentonimoKind::L => 32,
                    PentonimoKind::N => 33,
                    PentonimoKind::P => 34,
                    PentonimoKind::T => 35,
                    PentonimoKind::U => 36,
                    PentonimoKind::V => 91,
                    PentonimoKind::W => 92,
                    PentonimoKind::I => 93,
                    PentonimoKind::X => 94,
                    PentonimoKind::Y => 95,
                    PentonimoKind::Z => 96,
                };

                write!(f, "\x1b[{color}mxx\x1b[m")
            }
            PrintValue::Nothing => write!(f, ".."),
            PrintValue::Path(n) => write!(f, "\x1b[90m{n:2}\x1b[m"),
        }
    }
}

fn build_print_map(
    (mx, my): (u16, u16),
    (max, tiles): (u16, Vec<PositionedPentonimo>),
) -> Vec<PrintValue> {
    let mut map = TileMap::new((mx, my));
    for &tile in &tiles {
        assert!(map.can_place(tile));
        map |= tile;
    }

    let path = {
        let mut scratch = BfsScratch::new((mx, my));
        let (new_max, path) = scratch.graph_diameter(&map);
        assert_eq!(new_max, max);
        path
    };

    let mut grid = vec![PrintValue::Nothing; mx as usize * my as usize];

    for x in 0..mx {
        for y in 0..my {
            for tile in &tiles {
                if tile.get(x, y) {
                    let index = mx as usize * y as usize + x as usize;
                    assert_eq!(grid[index], PrintValue::Nothing);
                    grid[index] = PrintValue::Pentonimo(tile.pentonimo().kind());
                }
            }
        }
    }

    for (i, point) in dijkstra(&map, path).iter().enumerate() {
        let index = mx as usize * point.1 as usize + point.0 as usize;
        assert_eq!(grid[index], PrintValue::Nothing);
        grid[index] = PrintValue::Path(i);
    }

    grid
}

trait Printer {
    fn print(&self, shape: (u16, u16), max: u16, grid: &[PrintValue]);
}

struct ConsolePrinter;
impl Printer for ConsolePrinter {
    fn print(&self, (mx, my): (u16, u16), max: u16, grid: &[PrintValue]) {
        println!("({mx},{my}): {max}");
        for y in 0..my {
            for x in 0..mx {
                print!("{} ", grid[mx as usize * y as usize + x as usize]);
            }
            println!();
        }
    }
}

struct SvgPrinter(PathBuf);

impl Printer for SvgPrinter {
    fn print(&self, shape: (u16, u16), max: u16, grid: &[PrintValue]) {
        let mut file = File::create(&self.0).unwrap();

        let scale = 10;

        let mut write = || {
            writeln!(
                file,
                r#"<svg viewBox="0 0 {} {}" xmlns="http://www.w3.org/2000/svg">"#,
                shape.0 * scale,
                shape.1 * scale
            )?;

            for y in 0..shape.1 {
                for x in 0..shape.0 {
                    let index = shape.0 as usize * y as usize + x as usize;

                    // match grid[index] {
                    //     PrintValue::Pentonimo(kind) => {
                    //         write!(
                    //             file,
                    //             r#"<rect x="{x}" y="{y}" width="{scale}" height="{scale}" fill="red" />"#,
                    //         )?; // stroke="black"
                    //     }
                    //     PrintValue::Nothing => {}
                    //     PrintValue::Path(n) => {
                    //         writeln!(
                    //             file,
                    //             r#"
                    //             <g>
                    //             <rect x="{x}" y="{y}" width="{scale}" height="{scale}" fill="none" />
                    //             <text
                    //                 x="{}"
                    //                 y="{}"
                    //                 dominant-baseline="middle"
                    //                 text-anchor="middle"
                    //                 font-size="1"
                    //             >
                    //                 {n}
                    //             </text>
                    //             </g>
                    //         "#,
                    //             x + scale / 2,
                    //             y + scale / 2
                    //         )?;
                    //     }
                    // }

                    let color = match grid[index] {
                        PrintValue::Pentonimo(kind) => match kind {
                            PentonimoKind::F => "#ed7b24",
                            PentonimoKind::L => "#d479ed",
                            PentonimoKind::N => "#007fff",
                            PentonimoKind::P => "#57f26e",
                            PentonimoKind::T => "#3252c7",
                            PentonimoKind::U => "#640eb0",
                            PentonimoKind::V => "#85fdff",
                            PentonimoKind::W => "#1fb585",
                            PentonimoKind::I => "#ff1745",
                            PentonimoKind::X => "#ff85de",
                            PentonimoKind::Y => "#089c08",
                            PentonimoKind::Z => "#ffd417",
                        },
                        PrintValue::Nothing => "none",
                        PrintValue::Path(n) => {
                            writeln!(
                                file,
                                r#"<g>
                                    <rect x="{x}" y="{y}" width="{scale}" height="{scale}" fill="none" stroke="black" stroke-width="{sw}" />
                                    <text x="{tx}" y="{ty}" font-size="{fw}" text-anchor="middle">{n}</text>
                                </g>"#,
                                x = scale * x,
                                y = scale * y,
                                tx = (scale * x) as f32 + scale as f32 / 2.,
                                ty = (scale * y) as f32 + scale as f32 / 1.5,
                                sw = scale as f32 / 200.,
                                fw = scale as f32 / 2.
                            )?;
                            continue;
                        }
                    };

                    write!(
                        file,
                        r#"<rect x="{x}" y="{y}" width="{scale}" height="{scale}" fill="{color}" stroke="black" stroke-width="{sw}" />"#,
                        x = scale * x,
                        y = scale * y,
                        sw = scale as f32 / 200.,
                    )?;
                }
            }

            writeln!(file, "</svg>")?;

            std::io::Result::Ok(())
        };

        write().unwrap();
    }
}
