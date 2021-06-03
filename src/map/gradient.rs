use rand::prelude::*;
use rand_xoshiro::Xoshiro256StarStar;
use std::f64::consts::{E, PI, TAU};

/// 1.135 * PI.sqrt(); used when calculating the erfc
const ERF_DENOM: f64 = 2.0117351207777605;

#[derive(Debug, Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
}

#[allow(dead_code)]
impl Point {
    pub fn dist2(&self, x: f64, y: f64) -> f64 {
        (x - self.x).powi(2) + (y - self.y).powi(2)
    }

    pub fn dist(&self, x: f64, y: f64) -> f64 {
        self.dist2(x, y).sqrt()
    }

    /// Approximation of the complementary error function
    ///
    /// From Karagiannidis & Lioumpas (2007)
    /// https://en.wikipedia.org/wiki/Error_function#Approximation_with_elementary_functions
    pub fn erfc(&self, x: f64, y: f64) -> f64 {
        let d = self.dist(x, y) * 8.0 + 0.00001;

        ((1.0 - E.powf(-1.98 * d)) * E.powf(-d.powi(2))) / (ERF_DENOM * d)
    }
}

#[derive(Debug)]
struct Layer([Point; 4]);

impl Layer {
    /// Get the layer's value at point (x, y).
    ///
    /// # Returns
    ///
    /// A value within the closed range [0.0, 1.0]
    pub fn at(&self, x: f64, y: f64) -> f64 {
        let dist_1 = self.0[0].dist2(x, y);
        let dist_2 = self.0[1].dist2(x, y);
        let dist_3 = self.0[2].dist2(x, y);
        let dist_4 = self.0[3].dist2(x, y);

        1.0 - ((dist_1 * 1.4 + dist_2 * 0.50 + dist_3 * 0.75 - dist_4.powi(2)) * 1.5)
            .clamp(0.0, 1.0)
    }
}

#[derive(Debug)]
pub struct Gradient {
    layers: Vec<Layer>,
}

impl Gradient {
    pub fn new(rng: &mut Xoshiro256StarStar, num_layers: u32) -> Self {
        let mut layers = Vec::new();
        // Put our first point in the center
        let center_x = 0.5;
        let center_y = 0.5;
        let point1 = Point { x: 0.5, y: 0.5 };

        for _ in 0..num_layers {
            // Generate a random angle anywhere in the circle (0 - 2π is a full circle)
            let angle1 = rng.gen_range(0.0..TAU);
            // Generate a second angle somewhere between "close" and "opposite"
            let angle2 = angle1 + rng.gen_range((PI / 4.0)..PI);

            // 2nd point gets a random distance from the center, projected out on angle1
            let dist = rng.gen_range(0.1..0.3);
            let point2 = Point {
                x: center_x + dist * angle1.cos(),
                y: center_y + dist * angle1.sin(),
            };
            // 3rd point also gets a random distance, on angle2
            let dist = rng.gen_range(0.1..0.3);
            let point3 = Point {
                x: center_x + dist * angle2.cos(),
                y: center_y + dist * angle2.sin(),
            };

            // Our third angle is between our first two...
            let mut angle3 = (angle1 + angle2) / 2.0;
            if rng.gen() {
                // ...50% chance of being "between" on the "long side" instead
                angle3 += PI; // Adding π (in radians) is same as adding 180⁰ (in degrees)
            }
            // Give this point an extra "push" outward if it's close to either of our 1st or 2nd angles
            let dist = if (angle3 - angle1) % TAU < PI / 2.0 || (angle3 - angle2) % TAU < PI / 2.0 {
                0.28
            } else {
                0.2
            };
            let point4 = Point {
                x: center_x + dist * angle3.cos(),
                y: center_y + dist * angle3.sin(),
            };

            layers.push(Layer([point1, point2, point3, point4]));
        }

        Gradient { layers }
    }

    /// Get the gradient's value at point (x, y).
    ///
    /// # Returns
    ///
    /// A value within the closed range [0.0, 1.0]
    pub fn at(&self, x: f64, y: f64) -> f64 {
        // Get the gradient value at this point
        let mut pow = 3;

        self.layers.iter().fold(0.0, |sum, layer| {
            pow += 1;
            sum + layer.at(x, y).powi(pow / 2)
        }) / self.layers.len() as f64
    }
}
