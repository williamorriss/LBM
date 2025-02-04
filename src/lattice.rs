use crate::window::render::{D2,SimulationData};

///////////////////////////////////////
////// LB PARAMS AND CONSTANTS ////////
///////////////////////////////////////
const HEIGHT: usize = 32; // grid height
const WIDTH: usize = 512; // grid width
const OMEGA: f32 = 1.0 / (3.0 * 0.002 + 0.5); // Relaxation parameter (function of viscosity)
const U0: f32 = 0.1; // Initial speed
const FOUR_NINTHS: f32 = 4.0 / 9.0; // 4/9
const ONE_NINTH: f32 = 1.0 / 9.0; // 1/9
const ONE_THIRTYSIXTH: f32 = 1.0 / 36.0; // 1/36

///////////////////////////////////////
////// LATTICE-BOLTZMANN GLOBALS //////
///////////////////////////////////////
// Microscopic densities
pub struct Lattice {
    unit : Vec<f32>,
    north : Vec<f32>,
    south : Vec<f32>,
    east : Vec<f32>,
    west : Vec<f32>,
    north_west : Vec<f32>,
    north_east : Vec<f32>,
    south_east : Vec<f32>,
    south_west : Vec<f32>,
    // Barriers
    bar : Vec<f32>, 
    speed: Vec<f32>,
    dimensions: D2,
}

impl Lattice {
    pub fn new(width: usize, height: usize) -> Lattice {
        let length = width * height;
        Lattice {
            unit: vec![0.0; length],
            north: vec![0.0; length],
            south: vec![0.0; length],
            east: vec![0.0; length],
            west: vec![0.0; length],
            north_west: vec![0.0; length],
            north_east: vec![0.0; length],
            south_east: vec![0.0; length],
            south_west: vec![0.0; length],
            // Barriers
            bar: vec![0.0; length], 
            speed: vec![0.0; length],
            dimensions: D2 {x: width, y: height},
        }
    }

    pub fn get_coordinates(&self) -> D2 {
        self.dimensions
    }

    pub fn speed_show(&self) -> Vec<SimulationData> {
        self.speed.iter().enumerate()
            .map(|(index,&speed)| if self.bar[index] == 0.0 {SimulationData{speed}} else {SimulationData{speed: -10.0}}).collect()
    }

    pub fn simulate(&mut self){
        use std::time::Instant;
        let start = Instant::now();
        // Update all cells
        stream(self);
        bounce(self);
        collide(self);
    
        // Measure and print elapsed time
        let elapsed = start.elapsed();
        println!("T = {:?} ms", elapsed.as_millis());
    }
}

pub fn initialize(lattice: &mut Lattice,xtop: usize, ytop: usize, yheight: usize, u0: f32) {
    // Useful pre-computed constants
    let u0sq = u0 * u0;
    let u0sq_1_5 = 1.5 * u0sq;
    let u0sq_4_5 = 4.5 * u0sq;
    let u0_3 = 3.0 * u0;

    // Loop through the cells, initialize densities
    for i in 0..(HEIGHT * WIDTH) {
        lattice.unit[i] = FOUR_NINTHS * (1.0 - u0sq_1_5);
        lattice.north[i] = ONE_NINTH * (1.0 - u0sq_1_5);
        lattice.south[i] = ONE_NINTH * (1.0 - u0sq_1_5);
        lattice.east[i] = ONE_NINTH * (1.0 + u0_3 + u0sq_4_5 - u0sq_1_5);
        lattice.west[i] = ONE_NINTH * (1.0 - u0_3 + u0sq_4_5 - u0sq_1_5);
        lattice.north_west[i] = ONE_THIRTYSIXTH * (1.0 - u0_3 + u0sq_4_5 - u0sq_1_5);
        lattice.north_east[i] = ONE_THIRTYSIXTH * (1.0 + u0_3 + u0sq_4_5 - u0sq_1_5);
        lattice.south_west[i] = ONE_THIRTYSIXTH * (1.0 - u0_3 + u0sq_4_5 - u0sq_1_5);
        lattice.south_east[i] = ONE_THIRTYSIXTH * (1.0 + u0_3 + u0sq_4_5 - u0sq_1_5);

        // Initialize the barrier
        let x = i % WIDTH;
        let y = i / WIDTH;
        if x == xtop && y >= ytop && y < (ytop + yheight) {
            lattice.bar[i] = 1.0;
        }
    }
}

fn stream(lattice: &mut Lattice) {
    // Stream all internal cells
    for x in 0..(WIDTH - 1) {
        for y in 0..(HEIGHT - 1) {
            let idx = y * WIDTH + x;
            // Movement north
            lattice.north[idx] = lattice.north[idx + WIDTH];
            // Movement northwest
            lattice.north_west[idx] = lattice.north_west[idx + WIDTH + 1];
            // Movement west
            lattice.west[idx] = lattice.west[idx + 1];
            // Movement south
            lattice.south[(HEIGHT - y - 1) * WIDTH + x] = lattice.south[(HEIGHT - y - 2) * WIDTH + x];
            // Movement southwest
            lattice.south_west[(HEIGHT - y - 1) * WIDTH + x] = lattice.south_west[(HEIGHT - y - 2) * WIDTH + x + 1];
            // Movement east
            lattice.east[y * WIDTH + (WIDTH - x - 1)] = lattice.east[y * WIDTH + (WIDTH - x - 2)];
            // Movement northeast
            lattice.north_east[y * WIDTH + (WIDTH - x - 1)] = lattice.north_east[y * WIDTH + WIDTH + (WIDTH - x - 2)];
            // Movement southeast
            lattice.south_east[(HEIGHT - y - 1) * WIDTH + (WIDTH - x - 1)] = lattice.south_east[(HEIGHT - y - 2) * WIDTH + (WIDTH - x - 2)];
        }
    }

    // Tidy up the edges
    let x = WIDTH - 1;
    for y in 1..(HEIGHT - 1) {
        // Movement north on right boundary
        lattice.north[y * WIDTH + x] = lattice.north[y * WIDTH + x + WIDTH];
        // Movement south on right boundary
        lattice.south[(HEIGHT - y - 1) * WIDTH + x] = lattice.south[(HEIGHT - y - 2) * WIDTH + x];
    }
}

fn bounce(lattice: &mut Lattice) {
    // Loop through all interior cells
    for x in 2..(WIDTH - 2) {
        for y in 2..(HEIGHT - 2) {
            let idx = y * WIDTH + x;
            // If the cell contains a boundary
            if lattice.bar[idx] != 0.0 {
                // Push densities back from whence they came, then clear the cell
                lattice.north[idx - WIDTH] = lattice.south[idx];
                lattice.south[idx] = 0.0;
                lattice.south[idx + WIDTH] = lattice.north[idx];
                lattice.north[idx] = 0.0;
                lattice.east[idx + 1] = lattice.west[idx];
                lattice.west[idx] = 0.0;
                lattice.west[idx - 1] = lattice.east[idx];
                lattice.east[idx] = 0.0;
                lattice.north_east[idx - WIDTH + 1] = lattice.south_west[idx];
                lattice.south_west[idx] = 0.0;
                lattice.north_west[idx - WIDTH - 1] = lattice.south_east[idx];
                lattice.south_east[idx] = 0.0;
                lattice.south_east[idx + WIDTH + 1] = lattice.north_west[idx];
                lattice.north_west[idx] = 0.0;
                lattice.south_west[idx + WIDTH - 1] = lattice.north_east[idx];
                lattice.north_east[idx] = 0.0;

                // Clear zero density
                lattice.unit[idx] = 0.0;
            }
        }
    }
}

fn collide(lattice: &mut Lattice) {
    // Do not touch cells on top, bottom, left, or right
    for x in 1..(WIDTH - 1) {
        for y in 1..(HEIGHT - 1) {
            let idx = y * WIDTH + x;
            // Skip over cells containing barriers
            if lattice.bar[idx] == 0.0 {
                // Compute the macroscopic density
                let rho = lattice.unit[idx] + lattice.north[idx] + lattice.east[idx] + lattice.south[idx] + lattice.west[idx]
                    + lattice.north_east[idx] + lattice.south_east[idx] + lattice.south_west[idx] + lattice.north_west[idx];

                // Compute the macroscopic velocities (vx and vy)
                let ux = (lattice.east[idx] + lattice.north_east[idx] + lattice.south_east[idx] - lattice.west[idx] - lattice.north_west[idx] - lattice.south_west[idx]) / rho;
                let uy = (lattice.north[idx] + lattice.north_east[idx] + lattice.north_west[idx] - lattice.south[idx] - lattice.south_east[idx] - lattice.south_west[idx]) / rho;
                // Compute squares of velocities and cross-term
                let vx2 = ux * ux;
                let vy2 = uy * uy;
                let vxvy2 = 2.0 * ux * uy;
                let v2 = vx2 + vy2;
                let v215 = 1.5 * v2;

                lattice.speed[idx] = (vx2 + vy2).sqrt();
                //println!("{:?}", lattice.speed[idx]);

                // Perform collision updates
                lattice.east[idx] += OMEGA * (ONE_NINTH * rho * (1.0 + 3.0 * ux + 4.5 * vx2 - v215) - lattice.east[idx]);
                lattice.west[idx] += OMEGA * (ONE_NINTH * rho * (1.0 - 3.0 * ux + 4.5 * vx2 - v215) - lattice.west[idx]);
                lattice.north[idx] += OMEGA * (ONE_NINTH * rho * (1.0 + 3.0 * uy + 4.5 * vy2 - v215) - lattice.north[idx]);
                lattice.south[idx] += OMEGA * (ONE_NINTH * rho * (1.0 - 3.0 * uy + 4.5 * vy2 - v215) - lattice.south[idx]);
                lattice.north_east[idx] += OMEGA * (ONE_THIRTYSIXTH * rho * (1.0 + 3.0 * (ux + uy) + 4.5 * (v2 + vxvy2) - v215) - lattice.north_east[idx]);
                lattice.north_west[idx] += OMEGA * (ONE_THIRTYSIXTH * rho * (1.0 - 3.0 * ux + 3.0 * uy + 4.5 * (v2 - vxvy2) - v215) - lattice.north_west[idx]);
                lattice.south_east[idx] += OMEGA * (ONE_THIRTYSIXTH * rho * (1.0 + 3.0 * ux - 3.0 * uy + 4.5 * (v2 - vxvy2) - v215) - lattice.south_east[idx]);
                lattice.south_west[idx] += OMEGA * (ONE_THIRTYSIXTH * rho * (1.0 - 3.0 * (ux + uy) + 4.5 * (v2 + vxvy2) - v215) - lattice.south_west[idx]);

                // Conserve mass
                lattice.unit[idx] = rho - (lattice.east[idx] + lattice.west[idx] + lattice.north[idx] + lattice.south[idx] + lattice.north_east[idx] + lattice.south_east[idx] + lattice.north_west[idx] + lattice.south_west[idx]);
            }
        }
    }
}


