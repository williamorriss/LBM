use crate::lattice::D3;
use plotters::prelude::*;


pub const FILE_NAME: &str = "plot.png";

pub fn new(dim: &D3) {
    let root = BitMapBackend::new(
        FILE_NAME, 
        (dim.x as u32, dim.y as u32),
    ).into_drawing_area();
    root.fill(&WHITE).unwrap();
    
}