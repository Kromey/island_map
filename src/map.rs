use lerp::Lerp;
use rand::prelude::*;
use rand_xoshiro::Xoshiro256StarStar;

mod gradient;
use gradient::Gradient;

#[derive(Debug)]
pub struct Map {
    width: u32,
    height: u32,
    rng: Xoshiro256StarStar,
    gradient: Gradient,
}

impl Map {
    pub fn new(seed: u64, width: u32, height: u32) -> Self {
        let mut rng = Xoshiro256StarStar::seed_from_u64(seed);
        let gradient = Gradient::new(&mut rng, width, height);

        Map {
            width,
            height,
            rng,
            gradient,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn get_height(&self, x: f64, y: f64) -> f64 {
        let gradient = self.gradient.at(x, y);

        1.0.lerp(-0.2, 1.0 - gradient)
    }
}
