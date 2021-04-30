use lerp::Lerp;
use rand::prelude::*;
use rand_xoshiro::Xoshiro256StarStar;
use std::f64::consts::{PI, TAU};

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
    scale: f64,
}

impl Gradient {
    pub fn new(rng: &mut Xoshiro256StarStar, width: u32, height: u32) -> Self {
        let width = f64::from(width);
        let height = f64::from(height);

        let scale = width.max(height) / 3.0;

        let center_x = width / 2.0;
        let center_y = width / 2.0;

        let point1 = Point { x: center_x, y: center_y };

        let angle1 = rng.gen_range(0.0..TAU); // 0 - 2π
        let angle2 = rng.gen_range(0.0..TAU); // 0 - 2π

        let point2 = Point {
            x: center_x + 180.0 * angle1.cos(),
            y: center_y + 180.0 * angle1.sin(),
        };
        let point3 = Point {
            x: center_x + 200.0 * angle2.cos(),
            y: center_y + 200.0 * angle2.sin(),
        };


        let angle3 = match rng.gen_range(1..3) {
            0 => angle2,
            1 => angle1 + PI,
            2 => angle1.lerp(angle2, 0.5),
            _ => unreachable!(),
        };
        let point4 = Point {
            x: center_x + 220.0 * angle3.cos(),
            y: center_y + 220.0 * angle3.sin(),
        };

        Gradient {
            points: [point1, point2, point3, point4],
            scale,
        }
    }

    pub fn at(&self, x: f64, y: f64) -> f64 {
        let _scale = self.scale.powi(2);

        let quotient = 10.0;

        let grad1 = quotient / self.points[0].dist(x, y);
        let grad2 = quotient / self.points[1].dist(x, y);
        let grad3 = quotient / self.points[2].dist(x, y);
        let grad4 = quotient / self.points[3].dist(x, y);

        ((grad1 * 1.4 + grad2 * 0.50 + grad3 * 0.75 - grad4.powi(2)) * 1.5)
            .clamp(0.0, 1.0)
    }
}
