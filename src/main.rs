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



use window::render::{Vertex,Instance};
fn convert(dimensions: D3) -> impl Fn () -> (Vec<Instance>, [Vertex;4]) {
    let (height,width) = (dimensions.y, dimensions.x);

    let capacity = height * width;
    let x_res = 2.0/width as f32;
    let y_res = 2.0/height as f32;

    let vertices: [Vertex;4] = [
        Vertex {
            position: [-1.0,1.0, 0.0],
            tex_coords: [0.0,0.0],
        },
        Vertex {
            position: [-1.0 + x_res, 1.0, 0.0],
            tex_coords: [1.0,0.0],
        },
        Vertex {
            position: [-1.0, 1.0 - y_res,0.0],
            tex_coords: [0.0,1.0],
        },
        Vertex {
            position:[-1.0 + x_res, 1.0 - y_res,0.0],
            tex_coords: [1.0,1.0],
        }
    ];
    use rand::*;

    return move || -> (Vec<Instance>,[Vertex;4]) { //lattice input will go here
        let mut rng = rand::thread_rng();
        let mut instances: Vec<Instance> = Vec::with_capacity(capacity);
        for y in 0..height {
            for x in 0..width {
                let delta_x = x as f32 * x_res;
                let delta_y = -(y as f32 * y_res); 
                instances.push(
                    Instance {
                        position: [delta_x, delta_y],
                        colour: [rng.gen(),rng.gen(), rng.gen()],
                    }
                );                         
            }
        }
        (instances,vertices)
    }
}