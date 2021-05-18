use bracket_noise::prelude::*;
use lerp::Lerp;
use rand::prelude::*;
use rand_xoshiro::Xoshiro256StarStar;

mod gradient;
use gradient::Gradient;

pub const SEA_LEVEL: f64 = 0.0;

pub struct Map {
    width: u32,
    height: u32,
    //rng: Xoshiro256StarStar,
    noise: FastNoise,
    gradient: Gradient,
    coast: Vec<(u32,u32)>,
    heightmap: Vec<f64>,
}

impl Map {
    pub fn new(seed: u64, width: u32, height: u32) -> Self {
        let mut rng = Xoshiro256StarStar::seed_from_u64(seed);
        let gradient = Gradient::new(&mut rng, 4);
        let mut heightmap = vec![None; (width * height) as usize];

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

        let mut map = Map {
            width,
            height,
            //rng,
            noise,
            gradient,
            coast: Vec::new(),
            heightmap: Vec::new(),
        };

        map.find_coast(&mut heightmap);
        // Make one pass on a clone just to find our maximal distance from the coast
        let max_height = map.scale_height_from_coast(&mut heightmap.clone(), 1.0);
        // Now make the real pass, re-scaling height based on how far it from the coast
        map.scale_height_from_coast(&mut heightmap, max_height);

        map.heightmap = heightmap.into_iter().collect::<Option<Vec<f64>>>().unwrap();

        map
    }

    fn to_idx(&self, x: u32, y: u32) -> usize {
        (x * self.height()+ y) as usize
    }

    fn find_coast(&mut self, heightmap: &mut Vec<Option<f64>>) {
        let mut coast = Vec::with_capacity(heightmap.len() / 4);
        let mut active = vec![(0, 0)];

        while let Some((x, y)) = active.pop() {
            for (x, y) in self.get_neighbors(x, y) {
                if heightmap[self.to_idx(x, y)].is_some() {
                    continue;
                }

                let height = self.height_from_gradient(x as f64, y as f64);
                heightmap[self.to_idx(x, y)] = Some(height);

                if height > SEA_LEVEL {
                    coast.push((x as u32, y as u32));
                } else {
                    active.push((x, y));
                }
            }
        }

        coast.shrink_to_fit();
        self.coast = coast;
    }

    fn scale_height_from_coast(&self, heightmap: &mut Vec<Option<f64>>, height_scale: f64) -> f64 {
        let mut frontier = self.coast.clone();
        let mut next_frontier = Vec::with_capacity(frontier.len());

        let mut max_height: f64 = 0.0;

        for i in 0.. {
            if frontier.is_empty() {
                break;
            }
            next_frontier.clear();
            // Add 1 because we're actually going to be computing the *next* level
            let dist = f64::from(i + 1);

            while let Some((x, y)) = frontier.pop() {
                // Get the height of our current frontier point, which will become the height of any lakes we might touch
                let frontier_height = heightmap[self.to_idx(x, y)].unwrap();

                for (x, y) in self.get_neighbors(x, y) {
                    let idx = self.to_idx(x, y);
                    if heightmap[idx].is_none() {
                        let height = self.height_from_gradient(f64::from(x), f64::from(y)) * dist / height_scale;

                        if height <= SEA_LEVEL {
                            next_frontier.append(&mut self.get_lake(x, y, frontier_height, heightmap));
                            continue;
                        }

                        max_height = max_height.max(height);
                        heightmap[idx] = Some(height);

                        next_frontier.push((x, y));
                    }
                }
            }

            std::mem::swap(&mut frontier, &mut next_frontier);
        }

        max_height
    }

    fn get_lake(&self, x: u32, y: u32, lake_height: f64, heightmap: &mut Vec<Option<f64>>) -> Vec<(u32, u32)> {
        let mut lake_frontier = Vec::new();
        let mut active = vec![(x, y)];

        while let Some((x, y)) = active.pop() {
            for (x, y) in self.get_neighbors(x, y) {
                let idx = self.to_idx(x, y);
                if heightmap[idx].is_some() {
                    continue;
                }

                let height = self.height_from_gradient(f64::from(x), f64::from(y));
                heightmap[idx] = Some(lake_height);

                if height > SEA_LEVEL {
                    lake_frontier.push((x as u32, y as u32));
                } else {
                    heightmap[idx] = Some(lake_height);
                    active.push((x, y));
                }
            }
        }

        lake_frontier
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    fn height_from_gradient(&self, x: f64, y: f64) -> f64 {
        let scale = self.width.min(self.height) as f64;
        let x = x / scale;
        let y = y / scale;

        // Get the gradient value at this point
        let gradient = self.gradient.at(x, y);

        // Get a noise value, and "pull" it up
        let mut noise = self.noise.get_noise(x as f32, y as f32) as f64;
        noise = noise.lerp(0.5, 0.5);
        // Lerp it towards a point below sea level, using our gradient as the t-value
        (-0.2).lerp(noise, gradient)
    }

    fn get_neighbors(&self, x: u32, y: u32) -> impl Iterator<Item=(u32,u32)> {
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

    pub fn get_coast<'a>(&'a self) -> &'a Vec<(u32,u32)> {
        &self.coast
    }

    pub fn get_height(&self, x: u32, y: u32) -> f64 {
        self.heightmap[self.to_idx(x, y)]
    }
}
