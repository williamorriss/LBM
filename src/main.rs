mod lattice;
use lattice::{Lattice, Settings,D3,Q};


fn main() {  
    let settings = image_load();
    let mut lbm = Lattice::new(&settings);
    lbm.simulate()
}

fn image_load() -> Settings {
    use lattice::{Table,D2};
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