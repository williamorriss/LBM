use lbm::window::render::run;
use lbm::lattice::{Lattice,initialize};

const HEIGHT: usize = 32; // grid height
const WIDTH: usize = 512; // grid width
const U0: f32 = 0.1; // Initial speed

fn main() {
    // Initialize the simulation
    let mut lattice = Lattice::new(WIDTH, HEIGHT);
    initialize(&mut lattice, 25, 11, 10, U0);

    // Main simulation loop
    pollster::block_on(run(&mut lattice))
}