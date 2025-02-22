use lbm::window::render::run;
use lbm::lattice::Lattice;
use bitvec::prelude::*;
const HEIGHT: usize = 900; // grid height
const WIDTH: usize = 1600; // grid width
const U0: f32 = 0.2; // Initial speed
const OMEGA: f32 = 1.3; // Relaxation parameter (function of viscosity)

pub fn cyllindircal_barrier(center: (usize,usize), radius: u32) -> BitVec {
    let mut output = Vec::new();
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            output.push((x as i32-center.0 as i32).pow(2) + (y as i32 - center.1 as i32).pow(2) <= (radius * radius) as i32);
        }
    }
    output.into_iter().collect()
}
fn main() {
    let barriers = cyllindircal_barrier((400,450), 200);
    // Initialize the simulation
    let mut lattice = Lattice::new(WIDTH, HEIGHT,OMEGA, barriers);
    lattice.initialize(128*3, 11, 10, U0);

    // Main simulation loop
    pollster::block_on(run(&mut lattice))
}