use super::gradient::Gradient;
use bracket_noise::prelude::*;
use rand::prelude::*;
use rand_xoshiro::Xoshiro256StarStar;
use std::ops::{Index, IndexMut};

pub type Height = f64;

pub struct Elevation {
    elevation: Vec<Height>,
    coast: Vec<(u32, u32)>,
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
            coast: Vec::new(),
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
                elevation[idx] = height;
            }
        }

        // Normalize above-sea heights to [0.0, 1.0]
        for elev in elevation.elevation.iter_mut() {
            if *elev > 0.0 {
                *elev /= max_height;
            }
        }

        // Find the coast
        let mut ocean = vec![false; elevation.elevation.len()];
        elevation.coast = {
            let mut coast = Vec::with_capacity((width * height / 4) as usize);
            let mut active = vec![(0, 0)];

            while let Some((x, y)) = active.pop() {
                for (x, y) in elevation.get_neighbors(x, y) {
                    let idx = elevation.to_idx(x, y);
                    if ocean[idx] {
                        continue;
                    }

                    ocean[idx] = true;

                    if elevation[idx] > super::SEA_LEVEL {
                        coast.push((x as u32, y as u32));
                    } else {
                        active.push((x, y));
                    }
                }
            }

            coast.shrink_to_fit();
            coast
        };

        // Now we can find inland lakes and raise them up
        let mut lakes = Vec::new();
        let mut visited = vec![false; ocean.len()];
        for (idx, (elev, is_ocean)) in elevation.elevation.iter().zip(ocean).enumerate() {
            // We only care about non-ocean points below sea level
            if is_ocean || *elev > super::SEA_LEVEL {
                continue;
            }
            // Make sure we haven't already visited this one
            let xy = elevation.from_idx(idx);
            if visited[idx] {
                continue;
            }

            let mut lake = Vec::new();
            let mut active = vec![xy];
            // Set a really high initial value; we know our elevations<=1.0, so we're guaranteed to
            // find one lower than this
            let mut shore: f64 = 100.0;

            while let Some((x, y)) = active.pop() {
                for (x, y) in elevation.get_neighbors(x, y) {
                    // Make sure we haven't already visited this one
                    let idx = elevation.to_idx(x, y);
                    if visited[idx] {
                        continue;
                    }
                    // Mark it as visited so we don't re-visit
                    visited[idx] = true;

                    let elev = elevation[(x, y)];
                    if elev > super::SEA_LEVEL {
                        // Found a new shore point, check if it's lower
                        if elev < shore {
                            shore = elev;
                        }
                    } else {
                        // Add this point to our lake
                        lake.push(idx);
                        // Also add to our active list so we can test its neighbors too
                        active.push((x, y));
                    }
                }
            }
            // Because we're inside an iterator on elevation.elevation, we can't mutate it
            // So stash our lake for later update
            lakes.push((shore, lake));
        }
        // Now that we've found our lakes, pull them up to the height of their lowest shore point
        for (shore, lake) in lakes {
            for idx in lake {
                elevation[idx] = shore;
            }
        }

        elevation
    }

    fn get_neighbors(&self, x: u32, y: u32) -> impl Iterator<Item = (u32, u32)> {
        let width = self.width;
        let height = self.height;

        vec![
            (x.wrapping_sub(1), y.wrapping_sub(1)),
            (x.wrapping_sub(1), y),
            (x.wrapping_sub(1), y.wrapping_add(1)),
            (x, y.wrapping_sub(1)),
            (x, y.wrapping_add(1)),
            (x.wrapping_add(1), y.wrapping_sub(1)),
            (x.wrapping_add(1), y),
            (x.wrapping_add(1), y.wrapping_add(1)),
        ]
        .into_iter()
        .filter(move |(x, y)| *x < width && *y < height)
    }

    fn to_idx(&self, x: u32, y: u32) -> usize {
        (x * self.height + y) as usize
    }

    fn from_idx(&self, idx: usize) -> (u32, u32) {
        let idx = idx as u32;

        (idx / self.height, idx % self.height)
    }

    pub fn _iter(&self) -> impl Iterator<Item = &f64> {
        self.elevation.iter()
    }

    pub fn get_coast<'a>(&'a self) -> &'a Vec<(u32, u32)> {
        &self.coast
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

impl IndexMut<usize> for Elevation {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        &mut self.elevation[idx]
    }
}

impl IndexMut<(u32, u32)> for Elevation {
    fn index_mut(&mut self, key: (u32, u32)) -> &mut Self::Output {
        assert!(key.0 < self.width);
        assert!(key.1 < self.height);

        let idx = self.to_idx(key.0, key.1);

        &mut self[idx]
    }
}
