use crate::window::render::{D2,SimulationData};
use bitvec::prelude::*;

///////////////////////////////////////
////// LB PARAMS AND CONSTANTS ////////
///////////////////////////////////////
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
    bar: BitVec, 
    speed: Vec<f32>,
    dimensions: D2,
    timestep: u32,
    time: u32,
    height: usize,
    width: usize,
    omega: f32,
}

impl Lattice {
    pub fn new(width: usize, height: usize, omega: f32, barriers: BitVec) -> Lattice {
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
            bar: barriers, 
            speed: vec![0.0; length],
            dimensions: D2 {x: width, y: height},
            timestep: 0,
            time: 0,
            width, height, omega,
        }
    }

    pub fn get_coordinates(&self) -> D2 {
        self.dimensions
    }

    pub fn speed_show(&self) -> Vec<SimulationData> {
        self.speed.iter().enumerate()
            .map(|(index,&speed)| if !self.bar[index] {SimulationData{speed}} else {SimulationData{speed: -10.0}}).collect()
    }

    pub fn simulate(&mut self){
        self.timestep += 1;
        use std::time::Instant;
        let start = Instant::now();
        // Update all cells
        stream(self);
        bounce(self);
        collide(self);
    
        // Measure and print elapsed time
        let elapsed = start.elapsed();
        self.time += elapsed.as_micros() as u32;
        println!("T = {:?} micros :: AVG = {:?}", elapsed.as_micros(), self.time/ self.timestep);
    }

    pub fn initialize(&mut self,xtop: usize, ytop: usize, yheight: usize, u0: f32) {
        // Useful pre-computed constants
        let u0sq = u0 * u0;
        let u0sq_1_5 = 1.5 * u0sq;
        let u0sq_4_5 = 4.5 * u0sq;
        let u0_3 = 3.0 * u0;
    
        // Loop through the cells, initialize densities
        for i in 0..(self.height * self.width) {
            self.unit[i] = FOUR_NINTHS * (1.0 - u0sq_1_5);
            self.north[i] = ONE_NINTH * (1.0 - u0sq_1_5);
            self.south[i] = ONE_NINTH * (1.0 - u0sq_1_5);
            self.east[i] = ONE_NINTH * (1.0 + u0_3 + u0sq_4_5 - u0sq_1_5);
            self.west[i] = ONE_NINTH * (1.0 - u0_3 + u0sq_4_5 - u0sq_1_5);
            self.north_west[i] = ONE_THIRTYSIXTH * (1.0 - u0_3 + u0sq_4_5 - u0sq_1_5);
            self.north_east[i] = ONE_THIRTYSIXTH * (1.0 + u0_3 + u0sq_4_5 - u0sq_1_5);
            self.south_west[i] = ONE_THIRTYSIXTH * (1.0 - u0_3 + u0sq_4_5 - u0sq_1_5);
            self.south_east[i] = ONE_THIRTYSIXTH * (1.0 + u0_3 + u0sq_4_5 - u0sq_1_5);
    
            // // Initialize the barrier
            // let x = i % lattice.width;
            // let y = i / lattice.width;
            // if x == xtop && y >= ytop && y < (ytop + ylattice.lattice.lattice.lattice.lattice.lattice.lattice.height) {
            //     self.bar.set(i, true);
            // }
        }
    }
}


fn stream(lattice: &mut Lattice) {
    // Stream all internal cells
    for y in 0..(lattice.height - 1) {
        for x in 0..(lattice.width- 1) {
            let idx = y * lattice.width + x;
            // Movement north
            lattice.north[idx] = lattice.north[idx + lattice.width];
            // Movement northwest
            lattice.north_west[idx] = lattice.north_west[idx + lattice.width + 1];
            // Movement west
            lattice.west[idx] = lattice.west[idx + 1];
            // Movement south
            lattice.south[(lattice.height - y - 1) * lattice.width + x] = lattice.south[(lattice.height - y - 2) * lattice.width + x];
            // Movement southwest
            lattice.south_west[(lattice.height - y - 1) * lattice.width + x] = lattice.south_west[(lattice.height - y - 2) * lattice.width + x + 1];
            // Movement east
            lattice.east[y * lattice.width + (lattice.width - x - 1)] = lattice.east[y * lattice.width + (lattice.width - x - 2)];
            // Movement northeast
            lattice.north_east[y *lattice.width + (lattice.width - x - 1)] = lattice.north_east[y * lattice.width + lattice.width + (lattice.width - x - 2)];
            // Movement southeast
            lattice.south_east[(lattice.height - y - 1) * lattice.width + (lattice.width - x - 1)] = lattice.south_east[(lattice.height - y - 2) * lattice.width + (lattice.width - x - 2)];
        }
    }

    // Tidy up the edges
    let x = lattice.width - 1;
    for y in 1..(lattice.height - 1) {
        // Movement north on right boundary
        lattice.north[y * lattice.width + x] = lattice.north[y * lattice.width + x + lattice.width];
        // Movement south on right boundary
        lattice.south[(lattice.height - y - 1) * lattice.width + x] = lattice.south[(lattice.height - y - 2) * lattice.width + x];
    }
}

fn bounce(lattice: &mut Lattice) {
    // Loop through all interior cells
    for y in 2..(lattice.height - 2) {
        for x in 2..(lattice.width - 2) {
            let idx = y * lattice.width + x;
            // If the cell contains a boundary
            if lattice.bar[idx] {
                // Push densities back from whence they came, then clear the cell
                lattice.north[idx - lattice.width] = lattice.south[idx];
                lattice.south[idx] = 0.0;
                lattice.south[idx + lattice.width] = lattice.north[idx];
                lattice.north[idx] = 0.0;
                lattice.east[idx + 1] = lattice.west[idx];
                lattice.west[idx] = 0.0;
                lattice.west[idx - 1] = lattice.east[idx];
                lattice.east[idx] = 0.0;
                lattice.north_east[idx - lattice.width + 1] = lattice.south_west[idx];
                lattice.south_west[idx] = 0.0;
                lattice.north_west[idx - lattice.width - 1] = lattice.south_east[idx];
                lattice.south_east[idx] = 0.0;
                lattice.south_east[idx + lattice.width + 1] = lattice.north_west[idx];
                lattice.north_west[idx] = 0.0;
                lattice.south_west[idx + lattice.width - 1] = lattice.north_east[idx];
                lattice.north_east[idx] = 0.0;

                // Clear zero density
                lattice.unit[idx] = 0.0;
            }
        }
    }
}

fn collide(lattice: &mut Lattice) {
    // Do not touch cells on top, bottom, left, or right
    for y in 1..(lattice.height - 1) {
        for x in 1..(lattice.width - 1) {
            let idx = y * lattice.width + x;
            // Skip over cells containing barriers
            if !lattice.bar[idx] {
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

                lattice.speed[idx] = vx2 + vy2;
                //println!("{:?}", lattice.speed[idx]);

                // Perform collision updates
                lattice.east[idx] += lattice.omega * (ONE_NINTH * rho * (1.0 + 3.0 * ux + 4.5 * vx2 - v215) - lattice.east[idx]);
                lattice.west[idx] += lattice.omega * (ONE_NINTH * rho * (1.0 - 3.0 * ux + 4.5 * vx2 - v215) - lattice.west[idx]);
                lattice.north[idx] += lattice.omega * (ONE_NINTH * rho * (1.0 + 3.0 * uy + 4.5 * vy2 - v215) - lattice.north[idx]);
                lattice.south[idx] += lattice.omega * (ONE_NINTH * rho * (1.0 - 3.0 * uy + 4.5 * vy2 - v215) - lattice.south[idx]);
                lattice.north_east[idx] += lattice.omega * (ONE_THIRTYSIXTH * rho * (1.0 + 3.0 * (ux + uy) + 4.5 * (v2 + vxvy2) - v215) - lattice.north_east[idx]);
                lattice.north_west[idx] += lattice.omega * (ONE_THIRTYSIXTH * rho * (1.0 - 3.0 * ux + 3.0 * uy + 4.5 * (v2 - vxvy2) - v215) - lattice.north_west[idx]);
                lattice.south_east[idx] += lattice.omega * (ONE_THIRTYSIXTH * rho * (1.0 + 3.0 * ux - 3.0 * uy + 4.5 * (v2 - vxvy2) - v215) - lattice.south_east[idx]);
                lattice.south_west[idx] += lattice.omega * (ONE_THIRTYSIXTH * rho * (1.0 - 3.0 * (ux + uy) + 4.5 * (v2 + vxvy2) - v215) - lattice.south_west[idx]);

                // Conserve mass
                lattice.unit[idx] = rho - (lattice.east[idx] + lattice.west[idx] + lattice.north[idx] + lattice.south[idx] + lattice.north_east[idx] + lattice.south_east[idx] + lattice.north_west[idx] + lattice.south_west[idx]);
            }
        }
    }
}