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
        let gradient = Gradient::new(&mut rng, f64::from(width.max(height)));
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
        // Make one pass on a clone just to find out maximal distance from the coast
        let (max_height, max_dist) = map.scale_height_from_coast(&mut heightmap.clone(), 1.0, 1);
        // Now make the real pass, re-scaling height based on how far it from the coast
        map.scale_height_from_coast(&mut heightmap, max_height, max_dist);

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

    fn scale_height_from_coast(&self, heightmap: &mut Vec<Option<f64>>, height_scale: f64, max_dist: i32) -> (f64, i32) {
        let mut frontier = self.coast.clone();
        let mut next_frontier = Vec::with_capacity(frontier.len());

        let mut max_height: f64 = 0.0;
        let mut dist = 0;

        for i in 0.. {
            if frontier.is_empty() {
                break;
            }
            next_frontier.clear();
            dist = i;
            let scale = 1.0.lerp(height_scale, (f64::from(dist + 1) / f64::from(max_dist)).powi(2));

            while let Some((x, y)) = frontier.pop() {
                for (x, y) in self.get_neighbors(x, y) {
                    let idx = self.to_idx(x, y);
                    if heightmap[idx].is_none() {
                        let height = self.height_from_gradient(f64::from(x), f64::from(y));
                        max_height = max_height.max(height);
                        heightmap[idx] = Some(height / scale); //Some(height.lerp(1.0, height_scale));

                        next_frontier.push((x, y));
                    }
                }
            }

            std::mem::swap(&mut frontier, &mut next_frontier);
        }

        (max_height, dist)
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    fn height_from_gradient(&self, x: f64, y: f64) -> f64 {
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
