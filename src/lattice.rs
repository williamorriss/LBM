#![allow(dead_code)]
pub const Q: usize = 9; //dimensions of model
const WEIGHTS: [f32;Q] = [
    1./36., 1./9., 1./36.,
    1./9.,  4./9., 1./9.,
    1./36., 1./9., 1./36.,
];

const DIRECTIONS: [[f32;2];9] = [
    [-1.,1.],  [0.,1.],  [1.,1.],
    [-1.,0.],  [0.,0.],  [1.,0.],
    [-1.,-1.], [0.,-1.], [1.,-1.],
];

enum Dir {}
impl Dir {
    const NW: usize = 0;
    const N: usize = 1;
    const NE: usize = 2;
    const W: usize = 3;
    const UNIT: usize = 4;
    const E: usize = 5;
    const SW: usize = 6;
    const S: usize = 7;
    const SE: usize = 8;
}

//Dimension Structs//
#[derive(Clone, Copy, Debug)]
pub struct D2 {
    pub x: usize,
    pub y: usize,
}
#[derive(Clone, Copy, Debug)]
pub struct D3 {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}
//#################//

#[derive(Clone, Debug)] //custom DEBUG
pub struct Table<T> {
    pub data: Box<[T]>,
    pub dimensions: D2,
}

#[derive(Clone,Debug)]
pub struct Settings {
    pub barriers: Table<(usize,usize)>,
    pub dimensions: D3,
    pub omega: f32,
}


#[derive(Debug)]
pub struct Lattice {
    lattice: [Table<f32>;Q],
    rho: Table<f32>,
    ux: Table<f32>,
    uy: Table<f32>,
    speed: Table<f32>,
    settings: Settings,
    barriers: Table<(usize,usize)>,
    debug: Log,

}

#[derive(Debug)]
struct Log {
    timestep: usize,
    ux_file: std::fs::File,
    uy_file: std::fs::File,
}

impl Log {
    const UX_OUT: &'static str = "./debug/ux.txt";
    const UY_OUT: &'static str = "./debug/uy.txt";

    fn new() -> Log {
        std::fs::remove_file(Log::UX_OUT).unwrap();
        std::fs::remove_file(Log::UY_OUT).unwrap();

        let ux_file = std::fs::File::create(Log::UX_OUT).unwrap();
        let uy_file = std::fs::File::create(Log::UY_OUT).unwrap();

        Log {
            timestep: 0,
            ux_file,
            uy_file,
        }
    }

    fn format_table(table: &Table<f32>) -> String {
        let mut out = String::with_capacity(table.data.len());
        for row in 0..table.dimensions.y {
            let start = row * table.dimensions.x;
            let end = start + table.dimensions.x;
            out.push_str(&format!("{:?}\n", &table.data[start..end]));
        }
        out
    }

    fn log(&mut self, ux: &Table<f32>, uy: &Table<f32>) {
        let number = format!("{:?}\n", self.timestep);
        let number = number.as_bytes();
        let divider = "#################\n".as_bytes();
        let ux_table = Log::format_table(ux);
        let uy_table = Log::format_table(uy);

        self.ux_file.write(number).unwrap();
        self.ux_file.write(ux_table.as_bytes()).unwrap();
        self.ux_file.write(divider).unwrap();

        self.uy_file.write(number).unwrap();
        self.uy_file.write(uy_table.as_bytes()).unwrap();
        self.uy_file.write(divider).unwrap();
    }
}

impl Lattice {
    pub fn new(settings: &Settings) -> Self {
        let dimensions = D2 {x: settings.dimensions.x, y: settings.dimensions.y};
        let ones = Table {
            data: vec![1.0; dimensions.x * dimensions.y].into_boxed_slice(),
            dimensions,
        };
        let zeroes = Table {
            data: vec![0.0; dimensions.x * dimensions.y].into_boxed_slice(),
            dimensions,
        };

        Lattice {
            lattice: std::array::from_fn(|_| ones.clone()),
            rho: zeroes.clone(),
            ux: zeroes.clone(),
            uy: zeroes.clone(),
            speed: zeroes.clone(),
            settings: settings.clone(),
            barriers: settings.barriers.clone(),
            debug: Log::new(),
        }
    }

    #[inline]
    fn rotate_left (array: &mut [f32],shape: &D2) {
        array.chunks_mut(shape.x).into_iter().for_each(|row| {
            let last = row[0];
            row.copy_within(1.., 0);
            row[row.len() - 1] = last;
        });
    }
    
    #[inline]
    fn rotate_right (array: &mut [f32],shape: &D2) {
        array.chunks_mut(shape.x).into_iter().for_each(|row| {
            let first = row[row.len() - 1];
            row.copy_within(0..row.len() - 1, 1);
            row[0] = first;
        });
    }

    #[inline]
    fn rotate_down (array: &mut [f32], shape: &D2) {
        let (x,ln) = (shape.x,array.len());
        let first = &array[ln - x..].to_vec();
        array.copy_within(..ln - x, x);
        array[..x].copy_from_slice(&first)
    }

    #[inline]
    fn rotate_up(array: &mut [f32], shape: &D2) {
        let (x,ln) = (shape.x,array.len());
        let last = &array[..x].to_vec();
        array.copy_within(x.., 0);
        array[ln - x..].copy_from_slice(&last);
    }

    fn stream(&mut self) {
        for (table, [x,y]) in &mut self.lattice[..Q-1].iter_mut().zip(DIRECTIONS) {
            let (x,y) = (x as i8, y as i8);
            match x {
                1 => Lattice::rotate_right(&mut table.data, &table.dimensions),
                -1 => Lattice::rotate_left(&mut table.data, &table.dimensions),
                _ => (),
            }

            match y {
                1 => Lattice::rotate_up(&mut table.data, &table.dimensions),
                -1 => Lattice::rotate_down(&mut table.data, &table.dimensions),
                _ => (),
            }

        }
    }

    /// Normalises values for `Lattice::collide` 
    fn divide_rho(veloctiy: &mut Table<f32>, rho: &Table<f32>) {
        veloctiy.data.iter_mut()
        .zip(rho.data.iter())
        .filter(|(_, &rho)| rho != 0.0)
        .for_each(|(ux, rho)| *ux /= rho);
    }

    /// Collision Function
    fn collide(&mut self) {    
        //density//
        self.lattice.iter().for_each(|table| self.rho.add(table));

        //velocities//
            //ux//
        [Dir::E,Dir::NE,Dir::SE].into_iter().for_each(|dir| self.ux.add(&self.lattice[dir]));
        [Dir::W,Dir::NW,Dir::SW].into_iter().for_each(|dir| self.ux.sub(&self.lattice[dir]));
        Lattice::divide_rho(&mut self.ux, &self.rho);
        //uy//
        [Dir::N,Dir::NE,Dir::NW].into_iter().for_each(|dir| self.uy.add(&self.lattice[dir]));
        [Dir::S,Dir::SE,Dir::SW].into_iter().for_each(|dir| self.uy.sub(&self.lattice[dir]));
        Lattice::divide_rho(&mut self.uy, &self.rho);

        //collision//
        {
        let cells = self.settings.dimensions.x * self.settings.dimensions.y;
        for (table, weight, [x,y]) in itertools::izip!(self.lattice.iter_mut(),WEIGHTS,DIRECTIONS) {
            if x == 0. && y == 0. {continue;} // Unit
            let mut barrier_iter = self.barriers.data.iter().map(|indices| indices.1 * self.settings.dimensions.y + indices.0 );
            let mut barrier_index = barrier_iter.next().unwrap_or(usize::MAX);
            for i in 0..cells {
                if i == barrier_index {
                    barrier_index = barrier_iter.next().unwrap_or(usize::MAX);
                    continue;
                }
                let (ux, uy) = (self.ux.data[i],self.uy.data[i]);
                let magnitude = (ux * ux) + (uy * uy); //precompute these values somehwere else?
                let dot = x * ux + y * uy;
                let n_eq = self.rho.data[i] * weight * (1. + 3. * dot + 4.5 * (dot * dot) - 1.5 * magnitude);
                table.data[i] = table.data[i] + self.settings.omega * (table.data[i] - n_eq);
            }
        }
        }
        
        //Unit Velocity//
        self.lattice[Dir::UNIT].add(&self.rho);
        for i in (0..Q).filter(|&i| i != 4) {
            let lhs = &self.lattice[i] as *const Table<f32>;
            unsafe {self.lattice[Dir::UNIT].sub(&*lhs);}
        }
    }

    fn bounce(&mut self) {
        for positon in self.barriers.data.iter() {
            for (index,table) in self.lattice.iter_mut().enumerate() {
                let current = table[*positon];
                if current == 0.0 {
                    continue;
                } 
                let opposite_direction = Q - index - 1;
                let [x,y] = DIRECTIONS[opposite_direction];
                table[(positon.0 + x as usize, positon.1 + y as usize)] = current;
                table[*positon] = 0.0;

            }
        }
    }

    pub fn vorticity(&self) -> Vec<SimulationData> {
        let (width, height) = (self.settings.dimensions.x, self.settings.dimensions.y);
        // let (ux, uy) = (&self.ux, &self.uy);
        // let (dx, dy) = (1.0,1.0);
        let mut  out: Vec<SimulationData> = Vec::with_capacity(width * height);
        for (ux, uy) in self.ux.data.iter().zip(self.uy.data.iter()) {
            out.push(SimulationData {ux: *ux, uy: *uy});
        }
        out
    }

    pub fn simulate(&mut self) {
        self.debug.log(&self.ux, &self.uy);
        self.stream();
        self.bounce();
        self.collide();
        self.debug.timestep += 1;
    }
}

use std::io::Write;
//impl for indexing//
use std::ops::{Index, IndexMut};
impl Index<usize> for Lattice {
    type Output = Table<f32>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.lattice[index]
    }
}

impl IndexMut<usize> for Lattice {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.lattice[index]
    }
}

impl<T> Index<(usize,usize)> for Table<T> {
    type Output = T;
    fn index(&self, index: (usize,usize)) -> &Self::Output {
        &self.data[index.0 * self.dimensions.y + index.1]
    }
}

impl<T> IndexMut<(usize,usize)> for Table<T> {
    fn index_mut(&mut self, index: (usize,usize)) -> &mut Self::Output {
        &mut self.data[index.0 * self.dimensions.y + index.1]
    }
}
//Table Operations//
use std::ops::{AddAssign,SubAssign,MulAssign,DivAssign};

use crate::window::render::SimulationData;
impl<T: Copy + AddAssign + SubAssign + MulAssign + DivAssign> Table<T> {
    fn add(&mut self, rhs: &Self) {
        self.data.iter_mut()
            .zip(rhs.data.iter())
            .for_each(|(lhs,rhs)| *lhs += *rhs);
    }

    fn sub(&mut self, rhs: &Self) {
        self.data.iter_mut()
            .zip(rhs.data.iter())
            .for_each(|(lhs,rhs)| *lhs -= *rhs);
    }

    fn mul(&mut self, rhs: &Self) {
        self.data.iter_mut()
            .zip(rhs.data.iter())
            .for_each(|(lhs,rhs)| *lhs *= *rhs);
    }

    fn div(&mut self, rhs: &Self) {
        self.data.iter_mut()
            .zip(rhs.data.iter())
            .for_each(|(lhs,rhs)| *lhs /= *rhs);
    }
}


//Tests//
#[cfg(test)]
mod tests {
    mod rotation {
        use crate::lattice::{D2, Lattice};
        fn display_grid(grid: &[f32], shape: &D2) -> String {
            grid.chunks(shape.x).map(|line| format!("\n{:?}", line)).collect()
        }

        
        #[test]
        fn test_rotate_up() {
            let mut initial_grid = [
                00.0, 01.0, 02.0, 03.0, 04.0, 05.0,
                10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
                20.0, 21.0, 22.0, 23.0, 24.0, 25.0,
            ];

            let shape = D2 {
                x: 6,
                y: 3,
            };

            let expected = [
                10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
                20.0, 21.0, 22.0, 23.0, 24.0, 25.0,
                00.0, 01.0, 02.0, 03.0, 04.0, 05.0,
            ];

            Lattice::rotate_up(&mut initial_grid , &shape);
            assert_eq!(initial_grid, expected, "UP: {}", display_grid(&initial_grid, &shape))
        }

        #[test]
        fn test_rotate_down() {
            let mut initial_grid = [
                00.0, 01.0, 02.0, 03.0, 04.0, 05.0,
                10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
                20.0, 21.0, 22.0, 23.0, 24.0, 25.0,
            ];

            let shape = D2 {
                x: 6,
                y: 3,
            };

            let expected = [
                20.0, 21.0, 22.0, 23.0, 24.0, 25.0,
                00.0, 01.0, 02.0, 03.0, 04.0, 05.0,
                10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
            ];

            Lattice::rotate_down(&mut initial_grid , &shape);
            assert_eq!(initial_grid, expected, "DOWN: {}", display_grid(&initial_grid, &shape))
        }

        #[test]
        fn test_rotate_right() {
            let mut initial_grid = [
                00.0, 01.0, 02.0, 03.0, 04.0, 05.0,
                10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
                20.0, 21.0, 22.0, 23.0, 24.0, 25.0,
            ];

            let shape = D2 {
                x: 6,
                y: 3,
            };

            let expected = [
                05.0, 00.0, 01.0, 02.0, 03.0, 04.0,
                15.0, 10.0, 11.0, 12.0, 13.0, 14.0,
                25.0, 20.0, 21.0, 22.0, 23.0, 24.0,
            ];

            Lattice::rotate_right(&mut initial_grid , &shape);
            assert_eq!(initial_grid, expected, "RIGHT: {}", display_grid(&initial_grid, &shape))
        }

        #[test]
        fn test_rotate_left() {
            let mut initial_grid = [
                00.0, 01.0, 02.0, 03.0, 04.0, 05.0,
                10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
                20.0, 21.0, 22.0, 23.0, 24.0, 25.0,
            ];

            let shape = D2 {
                x: 6,
                y: 3,
            };

            let expected = [
                01.0, 02.0, 03.0, 04.0, 05.0, 00.0, 
                11.0, 12.0, 13.0, 14.0, 15.0, 10.0, 
                21.0, 22.0, 23.0, 24.0, 25.0, 20.0, 
            ];

            Lattice::rotate_left(&mut initial_grid , &shape);
            assert_eq!(initial_grid, expected, "LEFT: {}", display_grid(&initial_grid, &shape))
        }

        #[test]
        fn null() {
            let mut initial_grid = [
                00.0, 01.0, 02.0, 03.0, 04.0, 05.0,
                10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
                20.0, 21.0, 22.0, 23.0, 24.0, 25.0,
            ];

            let expected = initial_grid.clone();

            let shape = D2 {
                x: 6,
                y: 3,
            };

            Lattice::rotate_up(&mut initial_grid , &shape);
            Lattice::rotate_right(&mut initial_grid , &shape);
            Lattice::rotate_down(&mut initial_grid , &shape);
            Lattice::rotate_left(&mut initial_grid , &shape);

            assert_eq!(initial_grid, expected, "LEFT: {}", display_grid(&initial_grid, &shape))
        }

        
    }

    mod index {
        use crate::lattice::{Lattice, Settings, Table, D2, D3,Q};

        fn table_eq<T: PartialEq>(table1: &Table<T>, table2: &Table<T>) -> bool {
            for i in 0..table1.dimensions.x * table1.dimensions.y  {
                if table1.data[i] != table2.data[i] {
                    return false;
                }
            }
            true
        }


        #[test]
        fn lattice_index() {
            const X: usize = 6;
            const Y: usize = 3;
            let test_settings = Settings {
                barriers: Table {
                    data: vec![(1,1); X * Y].into_boxed_slice(),
                    dimensions: D2 {x: X, y: Y},
                },
                dimensions: D3 {x: X, y: Y, z: Q},
                omega: 0.0
            };
            let test_lattice = Lattice::new(&test_settings);
            (0..Q).into_iter().for_each(|index| assert!(table_eq(&test_lattice.lattice[0], &test_lattice[0]), "Table index error at {}", index));
        }

        #[test]
        fn table_index() {
            const X: usize = 6;
            const Y: usize = 3;
            let test_settings = Settings {
                barriers: Table {
                    data: vec![(1,1); X * Y].into_boxed_slice(),
                    dimensions: D2 {x: X, y: Y},
                },
                dimensions: D3 {x: X, y: Y, z: Q},
                omega: 0.0
            };
            let mut test_lattice = Lattice::new(&test_settings);
            test_lattice[0] = Table {
                data : vec![
                    01.0, 02.0, 03.0, 04.0, 05.0, 00.0, 
                    11.0, 12.0, 13.0, 14.0, 15.0, 10.0, 
                    21.0, 22.0, 23.0, 24.0, 25.0, 20.0, 
                ].into_boxed_slice(),
                dimensions: D2 {x: X, y: Y},
            };
            for i in 0..Y {
                for j in 0..X {
                    assert_eq!(test_lattice[0].data[i * Y + j], test_lattice[0][(i,j)], "{}", "ahhh")
                }
            }
        }
    }
}
//##################//