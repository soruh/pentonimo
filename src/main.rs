use pentonimo::{Pentonimo, PentonimoKind};
use tile::Rotate;

mod pentonimo;
mod tile;

fn main() {
    let pentonimo = Pentonimo::new(PentonimoKind::W)
        .shift_x(1)
        .unwrap()
        .shift_y(1)
        .unwrap();

    println!("{}", pentonimo.normalize());
    println!("{}", pentonimo.flip_x().normalize());
    println!("{}", pentonimo.flip_y().normalize());
    println!("{}", pentonimo.flip_xy().normalize());
    println!("{}", pentonimo.rotate(Rotate::Right).normalize());
    println!("{}", pentonimo.rotate(Rotate::Left).normalize());
    println!("{}", pentonimo.rotate(Rotate::Full).normalize());
}
