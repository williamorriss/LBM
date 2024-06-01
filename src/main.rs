mod lattice;
use lattice::{Lattice, Settings,D3,Q};

fn main() {  
    let lbm = Lattice::new(&Settings {
        dimensions: D3 {x: 100, y: 100, z: Q},
        relaxation: 0.5,
    });
    println!("{:?}", lbm[0][(1,2)]);

}