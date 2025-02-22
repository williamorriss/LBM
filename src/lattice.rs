
use crate::window::render::{D2,SimulationData};
use bitvec::bitvec;
use bitvec::prelude::*;

///////////////////////////////////////
////// LB PARAMS AND CONSTANTS ////////
///////////////////////////////////////
const HEIGHT: usize = 32; // grid height
const WIDTH: usize = 512; // grid width
const OMEGA: f32 = 1.0 / (3.0 * 0.002 + 0.5); // Relaxation parameter (function of viscosity)
const FOUR_NINTHS: f32 = 4.0 / 9.0; // 4/9
const ONE_NINTH: f32 = 1.0 / 9.0; // 1/9
const ONE_THIRTYSIXTH: f32 = 1.0 / 36.0; // 1/36

///////////////////////////////////////
////// Lattice-BOLTZMANN GLOBALS //////
///////////////////////////////////////
// Microscopic densities
pub struct Lattice {
    unit: Vec<f32>,
    north: Vec<f32>,
    south: Vec<f32>,
    east: Vec<f32>,
    west: Vec<f32>,
    north_west: Vec<f32>,
    north_east: Vec<f32>,
    south_east: Vec<f32>,
    south_west: Vec<f32>,
    rho: Vec<f32>,
    ux: Vec<f32>,
    uy: Vec<f32>,
    // Barriers
    bar: BitVec, 
    speed: Vec<f32>,
    dimensions: D2,
    timestep: u32,
    time: u32,
}

impl Lattice {
    pub fn new(width: usize, height: usize) -> Self {
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
            rho: vec![0.0; length],
            ux: vec![0.0; length],
            uy: vec![0.0; length],
            // Barriers
            bar: bitvec![0;length], 
            speed: vec![0.0; length],
            dimensions: D2 {x: width, y: height},
            timestep: 0,
            time: 0,
        }
    }

    pub fn get_coordinates(&self) -> D2 {
        self.dimensions
    }

    #[inline]
    fn rotate_left (array: &mut [f64],width: usize) {
        array.chunks_mut(width).into_iter().for_each(|row| {
            let last = row[1];
            row.copy_within(2.., 1);
            row[row.len() - 2] = last;
        });
    }

    #[inline]
    fn rotate_right (array: &mut [f64],width: usize) {
        array.chunks_mut(width).into_iter().for_each(|row| {
            let first = row[row.len() - 2];
            row.copy_within(1..row.len() - 2, 2);
            row[1] = first;
        });
    }

    #[inline]
    fn rotate_down (array: &mut [f64],width: usize) {
        let end = array.len() - 2*width;
        let first = &array[end..end+width].to_vec();
        array.copy_within(width..end, 2*width);
        array[width..2*width].copy_from_slice(&first)
    }

    #[inline]
    fn rotate_up(array: &mut [f64], width: usize) {
        let end = array.len() - 2*width;
        let last = &array[width..2*width].to_vec();
        array.copy_within(2*width..end, width);
        array[end..end+width].copy_from_slice(&last);
    }

    pub fn speed_show(&self) -> Vec<SimulationData> {

        self.speed.iter().enumerate()
            .map(|(index,&speed)| if !self.bar[index] {SimulationData{speed}} else {SimulationData{speed: -10.0}})
            //.inspect(|data| println!("{:?}", data))
            .collect()
    }

    pub fn simulate(&mut self){
        self.timestep += 1;
        use std::time::Instant;
        let start = Instant::now();
        // Update all cells
        self.stream();
        self.bounce();
        self.collide();
    
        // Measure and print elapsed time
        let elapsed = start.elapsed();
        self.time += elapsed.as_micros() as u32;
        println!("T = {:?} micros :: AVG = {:?}", elapsed.as_micros(), self.time/ self.timestep);
    }

    

    pub fn initialize(&mut self, xtop: usize, ytop: usize, yheight: usize, u0: f32) {
        // Useful pre-computed constants
        let u0sq = u0 * u0;
        let u0sq_1_5 = 1.5 * u0sq;
        let u0sq_4_5 = 4.5 * u0sq;
        let u0_3 = 3.0 * u0;

        self.unit.iter_mut().for_each(|cell| *cell = FOUR_NINTHS * (1.0 - u0sq_1_5));
        self.north.iter_mut().for_each(|cell| *cell = ONE_NINTH * (1.0 - u0sq_1_5));
        self.south.iter_mut().for_each(|cell| *cell = ONE_NINTH * (1.0 - u0sq_1_5));
        self.east.iter_mut().for_each(|cell| *cell = ONE_NINTH * (1.0 + u0_3 + u0sq_4_5 - u0sq_1_5));
        self.west.iter_mut().for_each(|cell| *cell = ONE_NINTH * (1.0 - u0_3 + u0sq_4_5 - u0sq_1_5));
        self.north_west.iter_mut().for_each(|cell| *cell = ONE_THIRTYSIXTH * (1.0 - u0_3 + u0sq_4_5 - u0sq_1_5));
        self.north_east.iter_mut().for_each(|cell| *cell = ONE_THIRTYSIXTH * (1.0 + u0_3 + u0sq_4_5 - u0sq_1_5));
        self.south_west.iter_mut().for_each(|cell| *cell = ONE_THIRTYSIXTH * (1.0 - u0_3 + u0sq_4_5 - u0sq_1_5));
        self.south_east.iter_mut().for_each(|cell| *cell = ONE_THIRTYSIXTH * (1.0 + u0_3 + u0sq_4_5 - u0sq_1_5));

        for i in 0..HEIGHT*WIDTH {
            let x = i % WIDTH;
            let y = i / WIDTH;
            if x == xtop && y >= ytop && y < (ytop + yheight) {
                self.bar.set(i, true);
            }
        }
    }

    fn stream(&mut self) {
        // Stream all internal cells
        for x in 0..(WIDTH - 1) {
            for y in 0..(HEIGHT - 1) {
                let idx = y * WIDTH + x;
                // Movement north
                self.north[idx] = self.north[idx + WIDTH];
                // Movement northwest
                self.north_west[idx] = self.north_west[idx + WIDTH + 1];
                // Movement west
                self.west[idx] = self.west[idx + 1];
                // Movement south
                self.south[(HEIGHT - y - 1) * WIDTH + x] = self.south[(HEIGHT - y - 2) * WIDTH + x];
                // Movement southwest
                self.south_west[(HEIGHT - y - 1) * WIDTH + x] = self.south_west[(HEIGHT - y - 2) * WIDTH + x + 1];
                // Movement east
                self.east[y * WIDTH + (WIDTH - x - 1)] = self.east[y * WIDTH + (WIDTH - x - 2)];
                // Movement northeast
                self.north_east[y * WIDTH + (WIDTH - x - 1)] = self.north_east[y * WIDTH + WIDTH + (WIDTH - x - 2)];
                // Movement southeast
                self.south_east[(HEIGHT - y - 1) * WIDTH + (WIDTH - x - 1)] = self.south_east[(HEIGHT - y - 2) * WIDTH + (WIDTH - x - 2)];
            }
        }

        // Tidy up the edges
        let x = WIDTH - 1;
        for y in 1..(HEIGHT - 1) {
            // Movement north on right boundary
            self.north[y * WIDTH + x] = self.north[y * WIDTH + x + WIDTH];
            // Movement south on right boundary
            self.south[(HEIGHT - y - 1) * WIDTH + x] = self.south[(HEIGHT - y - 2) * WIDTH + x];
        }
    }

    fn bounce(&mut self) {
        // Loop through all interior cells
        for x in 2..(WIDTH - 2) {
            for y in 2..(HEIGHT - 2) {
                let idx = y * WIDTH + x;
                // If the cell contains a boundary
                if self.bar[idx] {
                    // Push densities back from whence they came, then clear the cell
                    self.north[idx - WIDTH] = self.south[idx];
                    self.south[idx] = 0.0;
                    self.south[idx + WIDTH] = self.north[idx];
                    self.north[idx] = 0.0;
                    self.east[idx + 1] = self.west[idx];
                    self.west[idx] = 0.0;
                    self.west[idx - 1] = self.east[idx];
                    self.east[idx] = 0.0;
                    self.north_east[idx - WIDTH + 1] = self.south_west[idx];
                    self.south_west[idx] = 0.0;
                    self.north_west[idx - WIDTH - 1] = self.south_east[idx];
                    self.south_east[idx] = 0.0;
                    self.south_east[idx + WIDTH + 1] = self.north_west[idx];
                    self.north_west[idx] = 0.0;
                    self.south_west[idx + WIDTH - 1] = self.north_east[idx];
                    self.north_east[idx] = 0.0;

                    // Clear zero density
                    self.unit[idx] = 0.0;
                }
            }
        }
    }

    fn add_table(lhs: &mut [f32], rhs: &[f32]) {
        lhs.iter_mut().zip(rhs).for_each(|(l,r)| *l += r);
    }
    fn sub_table(lhs: &mut [f32], rhs: &[f32]) {
        lhs.iter_mut().zip(rhs).for_each(|(l,r)| *l -= r);
    }
    fn div_table(lhs: &mut [f32], rhs: &[f32]) {
        lhs.iter_mut().zip(rhs).for_each(|(l,r)| *l /= r);
    }


    fn calc_ux(&mut self) {
        self.ux = self.unit.clone();
        Lattice::add_table(&mut self.ux, &self.east);
        Lattice::add_table(&mut self.ux, &self.north_east);
        Lattice::add_table(&mut self.ux, &self.south_east);

        Lattice::sub_table(&mut self.ux, &self.west);
        Lattice::sub_table(&mut self.ux, &self.north_west);
        Lattice::sub_table(&mut self.ux, &self.south_west);

        Lattice::div_table(&mut self.ux, &self.rho);
    }

    fn calc_uy(&mut self) {
        self.uy = self.unit.clone();
        Lattice::add_table(&mut self.uy, &self.north);
        Lattice::add_table(&mut self.uy, &self.north_east);
        Lattice::add_table(&mut self.uy, &self.north_west);

        Lattice::sub_table(&mut self.uy, &self.south);
        Lattice::sub_table(&mut self.uy, &self.south_east);
        Lattice::sub_table(&mut self.uy, &self.south_west);

        Lattice::div_table(&mut self.uy, &self.rho);
    }

    fn calc_rho(&mut self) {
        self.rho = self.unit.clone();
        Lattice::add_table(&mut self.uy, &self.north);
        Lattice::add_table(&mut self.uy, &self.north_east);
        Lattice::add_table(&mut self.uy, &self.east);
        Lattice::add_table(&mut self.uy, &self.south_east);
        Lattice::add_table(&mut self.uy, &self.south);
        Lattice::add_table(&mut self.uy, &self.south_west);
        Lattice::add_table(&mut self.uy, &self.west);
        Lattice::add_table(&mut self.uy, &self.north_west);
    }


    fn collide(&mut self) {
        self.calc_rho();
        self.calc_ux();
        self.calc_uy();

        // Do not touch cells on top, bottom, left, or right
        for x in 1..(WIDTH - 1) {
            for y in 1..(HEIGHT - 1) {
                let idx = y * WIDTH + x;
                // Skip over cells containing barriers
                if !self.bar[idx] {
                    let rho = self.rho[idx];
                    let ux = self.rho[idx];
                    let uy = self.rho[idx];
                    // Compute squares of velocities and cross-term
                    let vx2 = ux * ux;
                    let vy2 = uy * uy;
                    let vxvy2 = 2.0 * ux * uy;
                    let v2 = vx2 + vy2;
                    let v215 = 1.5 * v2;

                    self.speed[idx] = (vx2 + vy2).sqrt();
                    //println!("{:?}", self.speed[idx]);

                    self.east[idx] += OMEGA * (ONE_NINTH * rho * (1.0 + 3.0 * ux + 4.5 * vx2 - v215) - self.east[idx]);
                    self.west[idx] += OMEGA * (ONE_NINTH * rho * (1.0 - 3.0 * ux + 4.5 * vx2 - v215) - self.west[idx]);
                    self.north[idx] += OMEGA * (ONE_NINTH * rho * (1.0 + 3.0 * uy + 4.5 * vy2 - v215) - self.north[idx]);
                    self.south[idx] += OMEGA * (ONE_NINTH * rho * (1.0 - 3.0 * uy + 4.5 * vy2 - v215) - self.south[idx]);
                    self.north_east[idx] += OMEGA * (ONE_THIRTYSIXTH * rho * (1.0 + 3.0 * (ux + uy) + 4.5 * (v2 + vxvy2) - v215) - self.north_east[idx]);
                    self.north_west[idx] += OMEGA * (ONE_THIRTYSIXTH * rho * (1.0 - 3.0 * ux + 3.0 * uy + 4.5 * (v2 - vxvy2) - v215) - self.north_west[idx]);
                    self.south_east[idx] += OMEGA * (ONE_THIRTYSIXTH * rho * (1.0 + 3.0 * ux - 3.0 * uy + 4.5 * (v2 - vxvy2) - v215) - self.south_east[idx]);
                    self.south_west[idx] += OMEGA * (ONE_THIRTYSIXTH * rho * (1.0 - 3.0 * (ux + uy) + 4.5 * (v2 + vxvy2) - v215) - self.south_west[idx]);
                    self.unit[idx] = rho - (self.east[idx] + self.west[idx] + self.north[idx] + self.south[idx] + self.north_east[idx] + self.south_east[idx] + self.north_west[idx] + self.south_west[idx]);
                }
            }
        }
    }
}

