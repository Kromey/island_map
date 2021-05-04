use bracket_noise::prelude::*;
use lerp::Lerp;
use rand::prelude::*;
use rand_xoshiro::Xoshiro256StarStar;
use std::collections::HashSet;

mod gradient;
use gradient::Gradient;

pub const SEA_LEVEL: f64 = 0.0;

pub struct Map {
    width: u32,
    height: u32,
    //rng: Xoshiro256StarStar,
    noise: FastNoise,
    gradient: Gradient,
}

impl Map {
    pub fn new(seed: u64, width: u32, height: u32) -> Self {
        let mut rng = Xoshiro256StarStar::seed_from_u64(seed);
        let gradient = Gradient::new(&mut rng, f64::from(width.max(height)));

        // I have no idea what these parameters do!
        // They're stolen directly from https://github.com/amethyst/bracket-lib/blob/master/bracket-noise/examples/simplex_fractal.rs
        // They do seem to give me results I like, though!
        let mut noise = FastNoise::seeded(rng.gen());
        noise.set_noise_type(NoiseType::SimplexFractal);
        noise.set_fractal_type(FractalType::FBM);
        noise.set_fractal_octaves(5);
        noise.set_fractal_gain(0.6);
        noise.set_fractal_lacunarity(2.0);
        noise.set_frequency(2.0);

        Map {
            width,
            height,
            //rng,
            noise,
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
        // Get the gradient value at this point
        let gradient = self.gradient.at(x, y).powi(2);

        let x = x / self.width as f64;
        let y = y / self.height as f64;

        // Get a noise value, and "pull" it up
        let mut height = self.noise.get_noise(x as f32, y as f32) as f64;
        height = height.lerp(0.5, 0.5);
        // Lerp it towards a point below sea level, using our gradient as the t-value
        height.lerp(-0.2, 1.0 - gradient)
    }

    fn get_neighbors(&self, x: u32, y: u32) -> Vec<(u32, u32)> {
        vec![
            (x.wrapping_sub(1), y.wrapping_sub(1)),
            (x.wrapping_sub(1), y),
            (x.wrapping_sub(1), y .wrapping_add(1)),
            (x, y.wrapping_sub(1)),
            (x, y .wrapping_add(1)),
            (x .wrapping_add(1), y.wrapping_sub(1)),
            (x .wrapping_add(1), y),
            (x .wrapping_add(1), y .wrapping_add(1)),
        ]
    }

    pub fn get_coast(&self) -> Vec<(f64, f64)> {
        let mut coast = HashSet::new();
        let mut active = vec![(0, 0)];
        let mut visited = HashSet::new();

        while let Some((x, y)) = active.pop() {
            for (x, y) in self.get_neighbors(x, y) {
                if x >= self.width || y >= self.height {
                    continue;
                }

                if self.get_height(f64::from(x), f64::from(y)) > SEA_LEVEL {
                    coast.insert((x, y));
                } else if visited.insert((x, y)) {
                    active.push((x, y));
                }
            }
        }

        coast.drain().map(|(x, y)| (f64::from(x), f64::from(y))).collect()
    }
}
