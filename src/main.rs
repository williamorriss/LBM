use lbm::window::render::{run, SimulationData, State};
use lbm::lattice::Lattice;
use bitvec::prelude::*;
use winit::{
    event_loop::EventLoop,
    window::WindowBuilder,
};
use std::thread;
use std::sync::{Arc,Mutex};

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
    let barriers = cyllindircal_barrier((400,450), 100);
    let initial_data = vec![SimulationData{speed: 0.0}; WIDTH*HEIGHT];
    let output = Arc::new(Mutex::new(initial_data));
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut lattice = Lattice::new(WIDTH, HEIGHT,OMEGA, barriers, output.clone());
    let mut state: State = pollster::block_on(State::new(&window, output.clone(), lattice.get_coordinates()));
    // Initialize the simulation
    lattice.initialize(U0);

    thread::scope(|s| {
        s.spawn(|| {
            loop {lattice.simulate();}
        });

        pollster::block_on(run(&mut state, event_loop))
    });
}