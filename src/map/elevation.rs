use super::gradient::Gradient;
use bracket_noise::prelude::*;
use rand::prelude::*;
use rand_xoshiro::Xoshiro256StarStar;
use std::ops::Index;

pub type Height = f64;

pub struct Elevation {
    elevation: Vec<Height>,
    width: u32,
    height: u32,
}

impl Elevation {
    pub fn new(rng: &mut Xoshiro256StarStar, width: u32, height: u32) -> Self {
        // Our gradient helps define our overall island shape
        let gradient = Gradient::new(rng, 4);

        // Noise gives us natural-looking terrain
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

        // A closure to allow us to easily use noise+gradient to calculate the base height
        let raw_height = move |x: f64, y: f64| -> f64 {
            // Get a noise value, and "pull" it up
            let mut noise = noise.get_noise(x as f32, y as f32) as f64;
            noise = (noise + 0.5) / 2.0;

            // Add our gradient
            noise + gradient.at(x, y)
        };

        let mut elevation = Elevation {
            elevation: vec![0.0; (width * height) as usize],
            width,
            height,
        };

        // Scale our (x, y) by the smallest of width or height
        let scale = f64::from(std::cmp::min(width, height));

        // Compute sea level to ensure a water border
        let sea_level = {
            // Establish the width of our water border
            let perimeter = 15;
            // Initialize sea level to a point we know will be within our border
            let mut sea_level = raw_height(0.0, 0.0);

            for x in 0..width {
                let x = f64::from(x) / scale;

                for dy in 0..perimeter {
                    let y1 = f64::from(dy) / scale;
                    let y2 = f64::from(height - dy) / scale;

                    let height = raw_height(x, y1);
                    if height > sea_level {
                        sea_level = height;
                    }

                    let height = raw_height(x, y2);
                    if height > sea_level {
                        sea_level = height;
                    }
                }
            }

            // Since we've already tackled x to with {perimiter} of top and bottom, we can limit y
            for y in perimeter..(height - perimeter) {
                let y = f64::from(y) / scale;

                for dx in 0..perimeter {
                    let x1 = f64::from(dx) / scale;
                    let x2 = f64::from(width - dx) / scale;

                    let height = raw_height(x1, y);
                    if height > sea_level {
                        sea_level = height;
                    }

                    let height = raw_height(x2, y);
                    if height > sea_level {
                        sea_level = height;
                    }
                }
            }

            // Give our sea level just a slight nudge
            sea_level + 0.01
        };

        // Find our max height so we can normalize our elevations
        let mut max_height = 0.0;
        for x in 0..width {
            // Pre-compute these values before entering the inner (y) loop
            let idx = elevation.to_idx(x, 0);
            let x = f64::from(x) / scale;

            for y in 0..height {
                let idx = idx + y as usize; // Add y to the pre-computed index
                let y = f64::from(y) / scale;

                let height = raw_height(x, y) - sea_level;
                if height > max_height {
                    max_height = height;
                }
                elevation.elevation[idx] = height;
            }
        }

        // Normalize above-sea heights to [0.0, 1.0]
        for elev in elevation.elevation.iter_mut() {
            if *elev > 0.0 {
                *elev /= max_height;
            }
        }

        elevation
    }

    fn to_idx(&self, x: u32, y: u32) -> usize {
        (x * self.height + y) as usize
    }

    pub fn iter(&self) -> impl Iterator<Item = &f64> {
        self.elevation.iter()
    }
}

impl Index<usize> for Elevation {
    type Output = Height;

    fn index(&self, idx: usize) -> &Self::Output {
        &self.elevation[idx]
    }
}

impl Index<(u32, u32)> for Elevation {
    type Output = Height;

    fn index(&self, key: (u32, u32)) -> &Self::Output {
        assert!(key.0 < self.width);
        assert!(key.1 < self.height);

        &self[self.to_idx(key.0, key.1)]
    }
}
