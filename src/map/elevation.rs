use super::gradient::Gradient;
use bracket_noise::prelude::*;
use nalgebra as na;
use rand::prelude::*;
use rand_xoshiro::Xoshiro256StarStar;
use std::ops::{Index, IndexMut};

pub type Height = f64;

const HEIGHT_SCALE: f64 = 40.0;

pub struct Elevation {
    elevation: Vec<Height>,
    coast: Vec<(u32, u32)>,
    size: u32,
}

impl Elevation {
    pub fn new(rng: &mut Xoshiro256StarStar, size: u32) -> Self {
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
            elevation: vec![0.0; (size * size) as usize],
            coast: Vec::new(),
            size,
        };

        // Scale our (x, y) by our size
        let scale = f64::from(size);

        // Compute sea level to ensure a water border
        let sea_level = {
            // Establish the width of our water border
            let perimeter = 30;
            // Initialize sea level to a point we know will be within our border
            let mut sea_level = raw_height(0.0, 0.0);

            for x in 0..size {
                let x = f64::from(x) / scale;

                for dy in 0..perimeter {
                    let y1 = f64::from(dy) / scale;
                    let y2 = f64::from(size - dy) / scale;

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
            for y in perimeter..(size - perimeter) {
                let y = f64::from(y) / scale;

                for dx in 0..perimeter {
                    let x1 = f64::from(dx) / scale;
                    let x2 = f64::from(size - dx) / scale;

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

        // Set heightmap values
        for y in 0..size {
            // Pre-compute these values before entering the inner (x) loop
            let idx = elevation.to_idx(0, y);
            let y = f64::from(y) / scale;

            for x in 0..size {
                let idx = idx + x as usize; // Add x to the pre-computed index
                let x = f64::from(x) / scale;

                let height = raw_height(x, y);
                elevation[idx] = height;
            }
        }

        // Find the coast
        let mut ocean = vec![false; elevation.len()];
        elevation.coast = {
            let mut coast = Vec::with_capacity((size * size / 4) as usize);
            let mut active = vec![(0, 0)];

            while let Some((x, y)) = active.pop() {
                for (x, y) in elevation.get_neighbors(x, y) {
                    let idx = elevation.to_idx(x, y);
                    if ocean[idx] {
                        continue;
                    }

                    ocean[idx] = true;

                    if elevation[idx] > sea_level {
                        coast.push((x, y));
                    } else {
                        active.push((x, y));
                    }
                }
            }

            coast.shrink_to_fit();
            coast
        };

        // Perform a breadth-first search to rescale heights based on distance from the coast
        // By using our ocean as the initial value for visited points we can restrict this to land
        let mut visited = ocean.clone();
        let mut frontier = elevation.coast.clone(); // Start at the coast
        let mut next_frontier = Vec::with_capacity(frontier.len());
        let mut max_elev = 0.0; // Find the max height for the second rescale pass
        while !frontier.is_empty() {
            while let Some((x, y)) = frontier.pop() {
                for (x, y) in elevation.get_neighbors(x, y) {
                    let idx = elevation.to_idx(x, y);
                    if visited[idx] {
                        continue;
                    }
                    visited[idx] = true;

                    // Get the distance to the nearest coast point
                    let d = {
                        let d = elevation.coast.iter().fold(u32::MAX, |min, coast| {
                            // Using distance squared here to avoid costly square root at this stage
                            let d = (x - coast.0).pow(2) + (y - coast.1).pow(2);
                            if d < min {
                                d
                            } else {
                                min
                            }
                        });
                        // Found the nearest distance squared, now we can square root it to get distance
                        f64::from(d).sqrt()
                    };

                    elevation[idx] *= d.sqrt();
                    if elevation[idx] > max_elev {
                        max_elev = elevation[idx];
                    }

                    next_frontier.push((x, y));
                }
            }

            std::mem::swap(&mut frontier, &mut next_frontier);
        }
        // Subtract sea level and re-scale all our heights
        max_elev -= sea_level;
        for elev in elevation.iter_mut() {
            *elev = (*elev - sea_level) / max_elev;
        }

        // Now we can find inland lakes and raise them up
        let mut lakes = Vec::new();
        let mut visited = vec![false; ocean.len()];
        for (idx, (elev, is_ocean)) in elevation.iter().zip(ocean).enumerate() {
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
        let size = self.size;

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
        .filter(move |(x, y)| *x < size && *y < size)
    }

    #[inline(always)]
    pub fn to_idx(&self, x: u32, y: u32) -> usize {
        (x + y * self.size) as usize
    }

    #[inline(always)]
    pub fn from_idx(&self, idx: usize) -> (u32, u32) {
        let idx = idx as u32;

        (idx % self.size, idx / self.size)
    }

    #[inline(always)]
    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn get_normal(&self, x: u32, y: u32) -> na::Vector3<f64> {
        // Assign a vertical unit vector to the ocean
        if self[(x, y)] <= super::SEA_LEVEL {
            return na::Vector3::new(0.0, 0.0, -1.0);
        }

        // Calculate normal for a height map using central differencing
        // https://stackoverflow.com/questions/49640250/calculate-normals-from-heightmap
        // Because we know our edges are ocean, and thus handled above, we don't need to guard for
        // underflows or overflows when getting neighbors here
        let rl = self[(x - 1, y)] - self[(x + 1, y)];
        let bt = self[(x, y - 1)] - self[(x, y + 1)];

        // TODO: #1 Average with the normal using the diagonal neighbors too?

        na::Vector3::new(rl * HEIGHT_SCALE, bt * HEIGHT_SCALE, -2.0).normalize()
    }

    pub fn len(&self) -> usize {
        self.elevation.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &f64> {
        self.elevation.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut f64> {
        self.elevation.iter_mut()
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
        assert!(
            key.0 < self.size,
            "X coordinate is out of bounds! {:?}",
            key
        );
        assert!(
            key.1 < self.size,
            "Y coordinate is out of bounds! {:?}",
            key
        );

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
        assert!(
            key.0 < self.size,
            "X coordinate is out of bounds! {:?}",
            key
        );
        assert!(
            key.1 < self.size,
            "Y coordinate is out of bounds! {:?}",
            key
        );

        let idx = self.to_idx(key.0, key.1);

        &mut self[idx]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_and_from_idx() {
        let mut rng = Xoshiro256StarStar::seed_from_u64(1337);
        let size = 20;
        let elev = Elevation::new(&mut rng, size);

        for x in 0..size {
            for y in 0..size {
                let idx = elev.to_idx(x, y);
                let (x2, y2) = elev.from_idx(idx);
                let idx2 = elev.to_idx(x2, y2);

                assert_eq!(
                    (x, y),
                    (x2, y2),
                    "{:?} and {:?} aren't the same!",
                    (x, y),
                    (x2, y2)
                );
                assert_eq!(idx, idx2, "idx and idx2 aren't the same!");
            }
        }
    }
}
