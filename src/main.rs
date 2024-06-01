mod lattice;
use lattice::{Lattice, Settings,D3,Q};

fn main() {  
    let lbm = Lattice::new(&Settings {
        dimensions: D3 {x: 100, y: 100, z: Q},
        omega: 0.5,
    });
    println!("{:?}", lbm[0][(1,2)]);
    let mut a = [1.0, 2.0, 3.0, 4.0];
    rotate_right(&mut a);
    rotate_left(&mut a);
    println!("{:?}", &mut a)

}

fn rotate_left (array: &mut [f32]) {
    let last = array[0];
    array.copy_within(1.., 0);
    array[array.len() - 1] = last;
}

fn rotate_right (array: &mut [f32]) {
    let first = array[array.len() - 1];
    array.copy_within(0..array.len() - 1, 1);
    array[0] = first;
}