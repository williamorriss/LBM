use bitvec::prelude::*;
use image::io::Reader;
use image::{GenericImageView, Pixel};
use lbm::lattice::Lattice;
use lbm::window::render::{run, SimulationData, State};
use std::env;
use std::sync::{Arc, Mutex};
use std::thread;
use winit::{event_loop::EventLoop, window::WindowBuilder};

const UX0: f32 = 0.2; // Initial speed
const UY0: f32 = -0.1; // Initial speed
const OMEGA: f32 = 0.5; // Relaxation parameter (function of viscosity)

pub fn cyllindircal_barrier(
    width: usize,
    height: usize,
    center: (usize, usize),
    radius: u32,
) -> BitVec {
    let mut output = Vec::new();
    for y in 0..height {
        for x in 0..width {
            output.push(
                (x as i32 - center.0 as i32).pow(2) + (y as i32 - center.1 as i32).pow(2)
                    <= (radius * radius) as i32,
            );
        }
    }
    output.into_iter().collect()
}

fn read_img() -> (usize, usize, BitVec) {
    let img = Reader::open("src/title.bmp").unwrap().decode().unwrap();
    let (width, height) = img.dimensions();
    let mut output = Vec::new();
    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel(x as u32, y as u32).to_rgb();
            output.push(!(pixel[0] == 0 && pixel[1] == 0 && pixel[2] == 0));
        }
    }
    (
        width as usize,
        height as usize,
        output.into_iter().collect(),
    )
}

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("Arguments: {:?}", &args[1..]); // Rest are user-provided args
    let scale_factor: usize = args[1].parse().unwrap();
    let (width, height): (usize, usize) = (16 * scale_factor, 9 * scale_factor);
    let barriers = cyllindircal_barrier(width, height, (400, 450), 100);
    let initial_data = vec![SimulationData { speed: 0.0 }; width * height];
    let output = Arc::new(Mutex::new(initial_data));
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut lattice = Lattice::new(width, height, OMEGA, barriers, output.clone());
    let mut state: State = pollster::block_on(State::new(
        &window,
        output.clone(),
        lattice.get_coordinates(),
    ));
    // Initialize the simulation
    lattice.initialize(UX0);

    thread::scope(|s| {
        s.spawn(|| {
            for _ in 0..2 {
                lattice.simulate();
            }
        });

        pollster::block_on(run(&mut state, event_loop))
    });
}
