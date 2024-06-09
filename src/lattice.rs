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
#[allow(dead_code)]
pub struct D2 {
    pub x: usize,
    pub y: usize,
}
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub struct D3 {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}
//#################//

#[derive(Clone,Debug)]
pub struct Table<T> {
    pub data: Box<[T]>,
    pub dimensions: D2,
}

#[derive(Clone,Debug)]
pub struct Settings {
    pub barriers: Table<(usize,usize)>,
    pub dimensions: D3,
    pub omega: f32,
    pub time_steps: u128,
}


#[derive(Clone,Debug)]
#[allow(dead_code)]
pub struct Lattice {
    lattice: [Table<f32>;Q],
    rho: Table<f32>,
    ux: Table<f32>,
    uy: Table<f32>,
    speed: Table<f32>,
    settings: Settings,
    barriers: Table<(usize,usize)>,

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

    fn collide(&mut self) {
        //density//
        self.lattice.iter().for_each(|table| self.rho.add(table));

        //velocities//
            //ux//
        [Dir::E,Dir::NE,Dir::SE].into_iter().for_each(|dir| self.ux.add(&self.lattice[dir]));
        [Dir::W,Dir::NW,Dir::SW].into_iter().for_each(|dir| self.ux.sub(&self.lattice[dir]));
        self.ux.div(&self.rho);
            //uy//
        [Dir::N,Dir::NE,Dir::NW].into_iter().for_each(|dir| self.uy.add(&self.lattice[dir]));
        [Dir::S,Dir::SE,Dir::SW].into_iter().for_each(|dir| self.uy.sub(&self.lattice[dir]));
        self.uy.div(&self.rho);

        //collision//
        {
        use itertools::izip;
        let omega = self.settings.omega;
        for (table, weight, [x,y]) in izip!(self.lattice.iter_mut(),WEIGHTS,DIRECTIONS) {
            if x == 0. && y == 0. {continue;} // Unit
            for (cell, ux, uy) in izip!(table.data.iter_mut(), self.ux.data.iter(), self.uy.data.iter()) {
                let magnitude = (ux * ux) + (uy * uy);
                let dot = x * ux + y * uy;
                *cell = omega * weight * (1. + 3. * dot + 4.5 * (dot * dot) - 1.5 * magnitude);
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
        let (x_max, y_max) =  (self.settings.dimensions.x, self.settings.dimensions.y);
        for (index,[x,y]) in DIRECTIONS.into_iter().enumerate().rev() {
            if x == 0. && y == 0. {continue;} // Unit
            let (x,y) = (x as usize, y as usize);
            let opposite_index = Q - index - 1;
            let current_table = &mut self.lattice[opposite_index];
            for (row,column) in self.barriers.data.into_iter() {
                let (row,column) = (*row, *column);
                let min = (column as i8 + x as i8) < 0 || (row as i8 + y as i8) < 0;
                let max = (column + x) < x_max || (row + y) < y_max;
                if min || max {continue;}
                current_table[(row + y, column + x)] = current_table[(row, column)];
                current_table[(row, column)] = 0.0;
            }
        }
    }

    pub fn simulate(&mut self) {
        for _ in 0..self.settings.time_steps {
            self.stream();
            self.bounce();
            self.collide();
        }
    }
}

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
#[allow(dead_code)]
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
//##################//