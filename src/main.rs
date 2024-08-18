use brute_force::find_best;
use candidates::Candidates;
use pathfinding::BfsScratch;
use pentonimo::{Pentonimo, PentonimoKind};
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

    dbg!(find_best((3, 3)));
    dbg!(find_best((4, 4)));
    dbg!(find_best((5, 5)));
}
