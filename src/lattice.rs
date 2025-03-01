use crate::window::render::{D2,SimulationData};
use bitvec::prelude::*;
use std::sync::{Arc,Mutex};
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
    output: Arc<Mutex<Vec<SimulationData>>>,
}

impl Lattice {
    pub fn new(width: usize, height: usize, omega: f32, barriers: BitVec, output: Arc<Mutex<Vec<SimulationData>>>) -> Lattice {
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
            output,
        }
    }

    pub fn get_coordinates(&self) -> D2 {
        self.dimensions
    }

    pub fn speed_show(&self) {
        let mut output = self.output.lock().unwrap();
        let vortex: Vec<SimulationData> = self.speed.iter().enumerate()
            .map(|(index,&speed)| if !self.bar[index] {SimulationData{speed}} else {SimulationData{speed: -10.0}}).collect();
        output.copy_from_slice(&vortex);
    }

    pub fn simulate(&mut self){
        self.timestep += 1;
        use std::time::Instant;
        let start = Instant::now();
        // Update all cells
        self.stream();
        self.bounce();
        self.collide();
        self.speed_show();
    
        // Measure and print elapsed time
        let elapsed = start.elapsed();
        self.time += elapsed.as_millis() as u32;
        if self.timestep % 100 == 0 {
            println!("T = {:?} millis :: AVG = {:?}", elapsed.as_millis(), self.time/ self.timestep);
        }
    }

    pub fn initialize(&mut self, u0: f32) {
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


    fn stream(&mut self) {
        // Stream all internal cells
        for y in 0..(&self.height - 1) {
            for x in 0..(&self.width- 1) {
                let idx = y * self.width + x;
                // Movement north
                self.north[idx] = self.north[idx + self.width];
                // Movement northwest
                self.north_west[idx] = self.north_west[idx + self.width + 1];
                // Movement west
                self.west[idx] = self.west[idx + 1];
                // Movement south
                self.south[(self.height - y - 1) * self.width + x] = self.south[(self.height - y - 2) * self.width + x];
                // Movement southwest
                self.south_west[(self.height - y - 1) * self.width + x] = self.south_west[(self.height - y - 2) * self.width + x + 1];
                // Movement east
                self.east[y * self.width + (self.width - x - 1)] = self.east[y * self.width + (self.width - x - 2)];
                // Movement northeast
                self.north_east[y *self.width + (self.width - x - 1)] = self.north_east[y * self.width + self.width + (self.width - x - 2)];
                // Movement southeast
                self.south_east[(self.height - y - 1) * self.width + (self.width - x - 1)] = self.south_east[(self.height - y - 2) * self.width + (self.width - x - 2)];
            }
        }

        // Tidy up the edges
        let x = self.width - 1;
        for y in 1..(self.height - 1) {
            // Movement north on right boundary
            self.north[y * self.width + x] = self.north[y * self.width + x + self.width];
            // Movement south on right boundary
            self.south[(self.height - y - 1) * self.width + x] = self.south[(self.height - y - 2) * self.width + x];
        }
    }


    fn bounce(&mut self) {
        // Loop through all interior cells
        for y in 2..(self.height - 2) {
            for x in 2..(self.width - 2) {
                let idx = y * self.width + x;
                // If the cell contains a boundary
                if self.bar[idx] {
                    // Push densities back from whence they came, then clear the cell
                    self.north[idx - self.width] = self.south[idx];
                    self.south[idx] = 0.0;
                    self.south[idx + self.width] = self.north[idx];
                    self.north[idx] = 0.0;
                    self.east[idx + 1] = self.west[idx];
                    self.west[idx] = 0.0;
                    self.west[idx - 1] = self.east[idx];
                    self.east[idx] = 0.0;
                    self.north_east[idx - self.width + 1] = self.south_west[idx];
                    self.south_west[idx] = 0.0;
                    self.north_west[idx - self.width - 1] = self.south_east[idx];
                    self.south_east[idx] = 0.0;
                    self.south_east[idx + self.width + 1] = self.north_west[idx];
                    self.north_west[idx] = 0.0;
                    self.south_west[idx + self.width - 1] = self.north_east[idx];
                    self.north_east[idx] = 0.0;

                    // Clear zero density
                    self.unit[idx] = 0.0;
                }
            }
        }
    }


    fn collide(&mut self) {
        // Do not touch cells on top, bottom, left, or right
        for y in 1..(self.height - 1) {
            for x in 1..(self.width - 1) {
                let idx = y * self.width + x;
                // Skip over cells containing barriers
                if !self.bar[idx] {
                    // Compute the macroscopic density
                    let rho = self.unit[idx] + self.north[idx] + self.east[idx] + self.south[idx] + self.west[idx]
                        + self.north_east[idx] + self.south_east[idx] + self.south_west[idx] + self.north_west[idx];

                    // Compute the macroscopic velocities (vx and vy)
                    let ux = (self.east[idx] + self.north_east[idx] + self.south_east[idx] - self.west[idx] - self.north_west[idx] - self.south_west[idx]) / rho;
                    let uy = (self.north[idx] + self.north_east[idx] + self.north_west[idx] - self.south[idx] - self.south_east[idx] - self.south_west[idx]) / rho;
                    // Compute squares of velocities and cross-term
                    let vx2 = ux * ux;
                    let vy2 = uy * uy;
                    let vxvy2 = 2.0 * ux * uy;
                    let v2 = vx2 + vy2;
                    let v215 = 1.5 * v2;

                    self.speed[idx] = vx2 + vy2;
                    //println!("{:?}", self.speed[idx]);

                    // Perform collision updates
                    self.east[idx] += self.omega * (ONE_NINTH * rho * (1.0 + 3.0 * ux + 4.5 * vx2 - v215) - self.east[idx]);
                    self.west[idx] += self.omega * (ONE_NINTH * rho * (1.0 - 3.0 * ux + 4.5 * vx2 - v215) - self.west[idx]);
                    self.north[idx] += self.omega * (ONE_NINTH * rho * (1.0 + 3.0 * uy + 4.5 * vy2 - v215) - self.north[idx]);
                    self.south[idx] += self.omega * (ONE_NINTH * rho * (1.0 - 3.0 * uy + 4.5 * vy2 - v215) - self.south[idx]);
                    self.north_east[idx] += self.omega * (ONE_THIRTYSIXTH * rho * (1.0 + 3.0 * (ux + uy) + 4.5 * (v2 + vxvy2) - v215) - self.north_east[idx]);
                    self.north_west[idx] += self.omega * (ONE_THIRTYSIXTH * rho * (1.0 - 3.0 * ux + 3.0 * uy + 4.5 * (v2 - vxvy2) - v215) - self.north_west[idx]);
                    self.south_east[idx] += self.omega * (ONE_THIRTYSIXTH * rho * (1.0 + 3.0 * ux - 3.0 * uy + 4.5 * (v2 - vxvy2) - v215) - self.south_east[idx]);
                    self.south_west[idx] += self.omega * (ONE_THIRTYSIXTH * rho * (1.0 - 3.0 * (ux + uy) + 4.5 * (v2 + vxvy2) - v215) - self.south_west[idx]);

                    // Conserve mass
                    self.unit[idx] = rho - (self.east[idx] + self.west[idx] + self.north[idx] + self.south[idx] + self.north_east[idx] + self.south_east[idx] + self.north_west[idx] + self.south_west[idx]);
                }
            }
        }
    }
}