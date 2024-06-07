mod lattice;
use lattice::{Lattice, Settings,D3,Q};

fn main() {  
    let mut lbm = Lattice::new(&Settings {
        dimensions: D3 {x: 3, y: 3, z: Q},
        omega: 0.5,
    });
    lbm.simulate()
}