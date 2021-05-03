use bracket_noise::prelude::*;
use lerp::Lerp;
use rand::prelude::*;
use rand_xoshiro::Xoshiro256StarStar;

mod gradient;
use gradient::Gradient;

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
}
