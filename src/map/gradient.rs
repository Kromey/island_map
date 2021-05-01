use lerp::Lerp;
use rand::prelude::*;
use rand_xoshiro::Xoshiro256StarStar;
use std::{f64::consts::{PI, TAU}, u32};

#[derive(Debug)]
struct Point {
    x: f64,
    y: f64,
}

impl Point {
    pub fn dist2(&self, x: f64, y: f64) -> f64 {
        (x - self.x).powi(2) + (y - self.y).powi(2)
    }

    pub fn dist(&self, x: f64, y: f64) -> f64 {
        self.dist2(x, y).sqrt()
    }
}

#[derive(Debug)]
pub struct Gradient {
    points: [Point; 4],
}

impl Gradient {
    pub fn new(rng: &mut Xoshiro256StarStar, width: u32, height: u32) -> Self {
        let width = f64::from(width);
        let height = f64::from(height);

        // Find the center and put our first point there
        let center_x = width / 2.0;
        let center_y = height / 2.0;
        let point1 = Point { x: center_x, y: center_y };

        // Generate a random angle anywhere in the circle
        let angle1 = rng.gen_range(0.0..TAU); // 0 - 2π (full circle)
        // Generate a second angle somewhere between "close" and "opposite"
        let angle2 = angle1 + rng.gen_range((PI / 4.0)..PI);

        // 2nd point gets a random distance from the center, projected out on angle1
        let dist = rng.gen_range(30.0..250.0);
        let point2 = Point {
            x: center_x + dist * angle1.cos(),
            y: center_y + dist * angle1.sin(),
        };
        // 3rd point also gets a random distance, on angle2
        let dist = rng.gen_range(15.0..250.0);
        let point3 = Point {
            x: center_x + dist * angle2.cos(),
            y: center_y + dist * angle2.sin(),
        };

        // Our third angle is between our first two...
        let mut angle3 = angle1.lerp(angle2, 0.5);
        if rng.gen() {
            // ...50% chance of being "between" on the "long side" instead
            angle3 += PI; // Adding π (in radians) is same as adding 180⁰ (in degrees)
        }
        // Give this point an extra "push" outward if it's close to either of our 1st or 2nd angles
        let dist = if (angle3 - angle1) % TAU < PI / 2.0 || (angle3 - angle2) % TAU < PI / 2.0 {
            225.0
        } else {
            160.0
        };
        let point4 = Point {
            x: center_x + dist * angle3.cos(),
            y: center_y + dist * angle3.sin(),
        };

        Gradient {
            points: [point1, point2, point3, point4],
        }
    }

    /// Get the gradient's value at point (x, y).
    ///
    /// # Returns
    ///
    /// A value within the closed range [0.0, 1.0]
    pub fn at(&self, x: f64, y: f64) -> f64 {
        let quotient = 10.0;

        let grad1 = quotient / self.points[0].dist(x, y);
        let grad2 = quotient / self.points[1].dist(x, y);
        let grad3 = quotient / self.points[2].dist(x, y);
        let grad4 = quotient / self.points[3].dist(x, y);

        ((grad1 * 1.4 + grad2 * 0.50 + grad3 * 0.75 - grad4.powi(2)) * 1.5)
            .clamp(0.0, 1.0)
    }
}
