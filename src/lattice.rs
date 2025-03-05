use crate::window::render::{D2,SimulationData};
use bitvec::prelude::*;
use itertools::*;
use rayon::prelude::*;
use rand::Rng;
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
    density: Vec<f32>,
    velx: Vec<f32>,
    vely: Vec<f32>,
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
            density: vec![0.0; length],
            velx: vec![0.0; length],
            vely: vec![0.0; length],
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
        if self.timestep % 500 == 0 {
            print!("T = {:?} millis :: AVG = {:?}\n", elapsed.as_millis(), self.time/ self.timestep);
        }
    }

    pub fn initialize(&mut self, ux0: f32, uy0: f32) {
        use rand;
        let mut rand_thread = rand::rng();
        // Loop through the cells, initialize densities
        for idx in 0..(self.height * self.width) {   
            let ux = ux0 + rand_thread.random_range(-0.1..0.1);
            let uy = uy0 + rand_thread.random_range(-0.1..0.1);
            let vx2 = ux * ux;
            let vy2 = uy * uy;
            let vxvy2 = 2.0 * ux * uy;
            let v2 = vx2 + vy2;
            let v215 = 1.5 * v2;         

            self.unit[idx] = FOUR_NINTHS  * (1.0 - v215);
            self.east[idx] = ONE_NINTH * (1.0 + 3.0 * ux + 4.5 * vx2 - v215);
            self.west[idx] = ONE_NINTH * (1.0 - 3.0 * ux + 4.5 * vx2 - v215);
            self.north[idx] = ONE_NINTH * (1.0 + 3.0 * uy + 4.5 * vy2 - v215);
            self.south[idx] = ONE_NINTH * (1.0 - 3.0 * uy + 4.5 * vy2 - v215);
            self.north_east[idx] = ONE_THIRTYSIXTH * (1.0 + 3.0 * (ux + uy) + 4.5 * (v2 + vxvy2) - v215);
            self.north_west[idx] = ONE_THIRTYSIXTH *  (1.0 - 3.0 * ux + 3.0 * uy + 4.5 * (v2 - vxvy2) - v215);
            self.south_east[idx] = ONE_THIRTYSIXTH * (1.0 + 3.0 * ux - 3.0 * uy + 4.5 * (v2 - vxvy2) - v215);
            self.south_west[idx] = ONE_THIRTYSIXTH * (1.0 - 3.0 * (ux + uy) + 4.5 * (v2 + vxvy2) - v215);

        }
    }


    fn stream(&mut self) {
        let height = self.height;
        let width = self.width;

        let north = &mut self.north;
        let south = &mut self.south;
        let east = &mut self.east;
        let west = &mut self.west;
        let north_west = &mut self.north_west;
        let north_east = &mut self.north_east;
        let south_east = &mut self.south_east;
        let south_west = &mut self.south_west;
        // Stream all internal cells

        std::thread::scope(|s| {
            s.spawn(move|| {
                for y in 0..(height - 1) {
                    for x in 0..(width- 1) {
                        let idx = y * width + x;
                        north[idx] = north[idx + width];
                    }
                }
            });

            s.spawn(move|| {
                for y in 0..(height - 1) {
                    for x in 0..(width- 1) {
                        let idx = y * width + x;
                        north_west[idx] = north_west[idx + width + 1];
                    }
                }
            });

            s.spawn(move|| {
                for y in 0..(height - 1) {
                    for x in 0..(width- 1) {
                        let idx = y * width + x;
                        west[idx] = west[idx + 1];
                    }
                }
            });

            s.spawn(move|| {
                for y in 0..(height - 1) {
                    for x in 0..(width- 1) {
                        south[(height - y - 1) * width + x] = south[(height - y - 2) * width + x];
                    }
                }
            });

            s.spawn(move|| {
                for y in 0..(height - 1) {
                    for x in 0..(width- 1) {
                        south_west[(height - y - 1) * width + x] = south_west[(height - y - 2) * width + x + 1];
                    }
                }
            });

            s.spawn(move|| {
                for y in 0..(height - 1) {
                    for x in 0..(width- 1) {
                        east[y * width + (width - x - 1)] = east[y * width + (width - x - 2)];
                    }
                }
            });

            s.spawn(move|| {
                for y in 0..(height - 1) {
                    for x in 0..(width- 1) {
                        north_east[y * width + (width - x - 1)] = north_east[y * width + width + (width - x - 2)];
                    }
                }
            });

            s.spawn(move|| {
                for y in 0..(height - 1) {
                    for x in 0..(width- 1) {
                        south_east[(height - y - 1) * width + (width - x - 1)] = south_east[(height - y - 2) * width + (width - x - 2)];
                    }
                }
            });

        });

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
        let idx: Vec<usize> = iproduct!(0..self.width,0..self.height).map(|(x,y)|y * self.width + x).collect();
        // Do not touch cells on top, bottom, left, or right
        self.density = idx.clone()
            .into_par_iter() // Iterate over the indices of elements
            .map(|i| {
                if self.bar[i] {return self.density[i];}
                // Sum the ith element across all vectors
                [&self.unit, &self.north, &self.east, &self.south, &self.west
                ,&self.north_east,&self.south_east,&self.south_west, &self.north_west].into_iter().map(|vec| vec[i]).sum::<f32>()
            })
            .collect();

        self.velx = idx.clone()
        .into_par_iter() // Iterate over the indices of elements
        .map(|idx| {
            if self.bar[idx] {return self.velx[idx];}
            // Sum the ith element across all vectors
            (self.east[idx] + self.north_east[idx] + self.south_east[idx] 
            - (self.west[idx] + self.north_west[idx] + &self.south_west[idx]))/self.density[idx]
        })
        .collect();

        self.vely = idx.clone()
        .into_par_iter() // Iterate over the indices of elements
        .map(|idx| {
            if self.bar[idx] {return self.vely[idx];}
            // Sum the ith element across all vectors
            (self.north[idx] + self.north_east[idx] + self.north_west[idx] 
            - (self.south[idx] + self.south_east[idx] + &self.south_west[idx]))/self.density[idx]
        })
        .collect();

        self.east = izip!(&self.bar,&self.east,&self.velx,&self.vely,&self.density).map(|(bar,&cell,&ux,&uy,&rho)| {
            if *bar {return cell;}

            let vx2 = ux * ux;
            let vy2 = uy * uy;
            let v2 = vx2 + vy2;
            let v215 = 1.5 * v2;

            cell + self.omega * (ONE_NINTH * rho * (1.0 + 3.0 * ux + 4.5 * vx2 - v215) - cell)
        }).collect();

        self.west = izip!(&self.bar,&self.west,&self.velx,&self.vely,&self.density).map(|(bar,&cell,&ux,&uy,&rho)| {
            if *bar {return cell;}

            let vx2 = ux * ux;
            let vy2 = uy * uy;
            let v2 = vx2 + vy2;
            let v215 = 1.5 * v2;

            cell + self.omega * (ONE_NINTH * rho * (1.0 - 3.0 * ux + 4.5 * vx2 - v215) - cell)
        }).collect();

        

        self.north = izip!(&self.bar,&self.north,&self.velx,&self.vely,&self.density).map(|(bar,&cell,&ux,&uy,&rho)| {
            if *bar {return cell;}

            let vx2 = ux * ux;
            let vy2 = uy * uy;
            let v2 = vx2 + vy2;
            let v215 = 1.5 * v2;
            

            cell + self.omega * (ONE_NINTH * rho * (1.0 + 3.0 * uy + 4.5 * vy2 - v215) - cell)
        }).collect();

        self.south = izip!(&self.bar,&self.south,&self.velx,&self.vely,&self.density).map(|(bar,&cell,&ux,&uy,&rho)| {
            if *bar {return cell;}

            let vx2 = ux * ux;
            let vy2 = uy * uy;
            let v2 = vx2 + vy2;
            let v215 = 1.5 * v2;

            cell + self.omega * (ONE_NINTH * rho * (1.0 - 3.0 * uy + 4.5 * vy2 - v215) - cell)
        }).collect();

        self.north_east = izip!(&self.bar,&self.north_east,&self.velx,&self.vely,&self.density).map(|(bar,&cell,&ux,&uy,&rho)| {
            if *bar {return cell;}

            let vx2 = ux * ux;
            let vy2 = uy * uy;
            let v2 = vx2 + vy2;
            let vxvy2 = 2.0 * ux * uy;
            let v215 = 1.5 * v2;

            cell + self.omega * (ONE_THIRTYSIXTH * rho * (1.0 + 3.0 * (ux + uy) + 4.5 * (v2 + vxvy2) - v215) - cell)
        }).collect();



        self.south_east = izip!(&self.bar,&self.south_east,&self.velx,&self.vely,&self.density).map(|(bar,&cell,&ux,&uy,&rho)| {
            if *bar {return cell;}

            let vx2 = ux * ux;
            let vy2 = uy * uy;
            let v2 = vx2 + vy2;
            let vxvy2 = 2.0 * ux * uy;
            let v215 = 1.5 * v2;

            cell + self.omega * (ONE_THIRTYSIXTH * rho * (1.0 + 3.0 * ux - 3.0 * uy + 4.5 * (v2 - vxvy2) - v215) - cell)
        }).collect();

        self.north_west = izip!(&self.bar,&self.north_west,&self.velx,&self.vely,&self.density).map(|(bar,&cell,&ux,&uy,&rho)| {
            if *bar {return cell;}

            let vx2 = ux * ux;
            let vy2 = uy * uy;
            let v2 = vx2 + vy2;
            let vxvy2 = 2.0 * ux * uy;
            let v215 = 1.5 * v2;

            cell + self.omega * (ONE_THIRTYSIXTH * rho * (1.0 - 3.0 * ux + 3.0 * uy + 4.5 * (v2 - vxvy2) - v215) - cell)
        }).collect();

        self.south_west = izip!(&self.bar,&self.south_west,&self.velx,&self.vely,&self.density).map(|(bar,&cell,&ux,&uy,&rho)| {
            if *bar {return cell;}

            let vx2 = ux * ux;
            let vy2 = uy * uy;
            let v2 = vx2 + vy2;
            let vxvy2 = 2.0 * ux * uy;
            let v215 = 1.5 * v2;

            cell + self.omega * (ONE_THIRTYSIXTH * rho * (1.0 - 3.0 * (ux + uy) + 4.5 * (v2 + vxvy2) - v215) - cell)
        }).collect();


        
        self.unit = (0..self.height * self.width)
            .into_par_iter() // Iterate over the indices of elements
            .map(|i| {
                if self.bar[i] {return 0.0;}
                // Sum the ith element across all vectors
                self.density[i] - [&self.east, &self.west,&self.north,
                    &self.south,&self.north_east,&self.south_east,
                    &self.north_west,&self.south_west].into_iter().map(|vec| vec[i]).sum::<f32>()
            })
            .collect();

        self.speed = self.velx.par_iter().zip(self.vely.par_iter()).map(|(ux,uy)| ux * ux + uy * uy).collect();
    }
}