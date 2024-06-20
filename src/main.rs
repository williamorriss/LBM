pub mod window {
    pub mod render;
    pub mod texture;
}

mod lattice;
use crate::lattice::{Settings, Table, D2, D3,Q};


fn image_load() -> Settings {
    let img = image::open("lbm.png").unwrap();
    let rbg_img = img.as_rgb8().unwrap();
    let (width, height) = rbg_img.dimensions();
    let (x,y) = (width as usize, height as usize);
    //let water_mask = rbg_img.to_vec().chunks(3).into_iter().map(|pixel| pixel[1] == 255);
    let collision_mask: Vec<(usize, usize)> = rbg_img
        .to_vec()
        .chunks(3)
        .into_iter()
        .enumerate()
        .filter(|(_,pixel)| pixel[0] + pixel[1] + pixel[2] == 0) //black pixels
        .map(|(index,_)| {
            let high = index / x;
            (high, index % x * high)
        })
        .collect();
        
    let barriers = Table {
        data: collision_mask.into_boxed_slice(),
        dimensions: D2 {x,y},
    };

    Settings {
        barriers, 
        dimensions: D3 {x, y, z: Q},
        omega: 0.5,
        time_steps: 100,
    }


}


fn main() {  
    let settings = image_load();
    //let mut lbm = lattice::Lattice::new(&settings);
    //lbm.simulate();
    let generate = convert(settings.dimensions);
    pollster::block_on(window::render::run(generate));
}


use window::render::Vertex;
fn convert(dimensions: D3) -> impl Fn () -> (Vec<Vertex>, Vec<u16>) {

    let capacity = dimensions.x * dimensions.y;
    let (height,width) = (dimensions.y, dimensions.x);
    let x_res = 2.0/(width - 1) as f32;
    let y_res = 2.0/(height -1) as f32;
    let u16height = height as u16;

    return move || -> (Vec<Vertex>, Vec<u16>) { //lattice input will go here
        let mut cells: Vec<(f32,f32)> = Vec::with_capacity(capacity);
        let mut indices = Vec::with_capacity(capacity * 6);
        for y in 0..height {
            for x in 0..width {
                let x_pos = x as f32 * x_res - 1.0;
                let y_pos = 1.0 - y as f32 * y_res; 
                cells.push((x_pos, y_pos));

                let top_left = (y * height + x) as u16;
                let top_right = top_left + 1;
                let bottom_left = top_left + u16height;
                let bottom_right = bottom_left + 1;
                //left triangle
                indices.push(bottom_left);
                indices.push(bottom_right);
                indices.push(top_right);
                //right triangle
                indices.push(top_right);
                indices.push(top_left);
                indices.push(bottom_left);

                         
            }
        }
        use rand::*;
        let mut rng = rand::thread_rng();
        let vertices: Vec<Vertex> = cells.into_iter().map(|cell| Vertex {
            position: [cell.0, cell.1, 0.0], 
            tex_coords: [rng.gen(),rng.gen()],
        }).collect();
        (vertices, indices)
    }
}