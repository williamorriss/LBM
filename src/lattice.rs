pub const Q: usize = 9; //dimensions of model
const WEIGHTS: [f32;Q] = [
    1./36., 1./9., 1./36.,
    1./9.,  4./9., 1./9.,
    1./36., 1./9., 1./36.,
];

const DIRECTIONS: [[i8;2];9] = [
    [-1,1],  [0,1],  [1,1],
    [-1,0],  [0,0],  [1,0],
    [-1,-1], [0,-1], [1,-1],
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
struct D2 {
    x: usize,
    y: usize,
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
pub struct Table {
    data: Box<[f32]>,
    dimensions: D2,
}

#[derive(Clone,Debug)]
pub struct Settings {
    pub dimensions: D3,
    pub omega: f32,
}


#[derive(Clone,Debug)]
#[allow(dead_code)]
pub struct Lattice {
    lattice: [Table;Q],
    settings: Settings,
    rho: Table,
    ux: Table,
    uy: Table,
    speed: Table,

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
        }
    }

    pub fn density(&mut self) {
        self.lattice.iter().for_each(|table| self.rho.add(table));
    }

    pub fn velocities(&mut self) {
        //ux//
        [Dir::E,Dir::NE,Dir::SE].into_iter().for_each(|dir| self.ux.add(&self.lattice[dir]));
        [Dir::W,Dir::NW,Dir::SW].into_iter().for_each(|dir| self.ux.sub(&self.lattice[dir]));
        self.ux.div(&self.rho);
        //uy//
        [Dir::N,Dir::NE,Dir::NW].into_iter().for_each(|dir| self.uy.add(&self.lattice[dir]));
        [Dir::S,Dir::SE,Dir::SW].into_iter().for_each(|dir| self.uy.sub(&self.lattice[dir]));
        self.uy.div(&self.rho);
    }

    fn collide(&mut self) {
        self.density();
        self.velocities();
        
    }
}

//impl for indexing//
use std::ops::{Index, IndexMut};
impl Index<usize> for Lattice {
    type Output = Table;
    fn index(&self, index: usize) -> &Self::Output {
        &self.lattice[index]
    }
}

impl IndexMut<usize> for Lattice {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.lattice[index]
    }
}

impl Index<(usize,usize)> for Table {
    type Output = f32;
    fn index(&self, index: (usize,usize)) -> &Self::Output {
        &self.data[index.0 * self.dimensions.y + index.1]
    }
}

impl IndexMut<(usize,usize)> for Table {
    fn index_mut(&mut self, index: (usize,usize)) -> &mut Self::Output {
        &mut self.data[index.0 * self.dimensions.y + index.1]
    }
}
//Table Operations//
impl Table {
    fn add(&mut self, rhs: &Self) {
        self.data.iter_mut()
            .zip(rhs.data.iter())
            .for_each(|(lhs,rhs)| *lhs += rhs);
    }

    fn sub(&mut self, rhs: &Self) {
        self.data.iter_mut()
            .zip(rhs.data.iter())
            .for_each(|(lhs,rhs)| *lhs -= rhs);
    }

    fn mul(&mut self, rhs: &Self) {
        self.data.iter_mut()
            .zip(rhs.data.iter())
            .for_each(|(lhs,rhs)| *lhs *= rhs);
    }

    fn div(&mut self, rhs: &Self) {
        self.data.iter_mut()
            .zip(rhs.data.iter())
            .for_each(|(lhs,rhs)| *lhs /= rhs);
    }
}
//##################//