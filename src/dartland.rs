use rand::prelude::*;
use rand_distr::Normal;
use rand_xoshiro::Xoshiro256StarStar;
use std::f64::consts::TAU; // TAU == 2*PI

#[derive(Debug)]
struct Extremum {
    x: f64,
    y: f64,
    h: f64,
}

#[derive(Debug)]
pub struct DartLand {
    extrema: Vec<Extremum>,
}

impl DartLand {
    pub fn new(num_positive: usize, num_negative: usize, width: u32, height: u32, seed: u64) -> Self {
        let center_x = width as f64 / 2.0;
        let center_y = width as f64 / 2.0;

        let extrema_count = num_positive + num_negative;
        
        let avg_dist = u32::min(width, height) as f64 / 4.0;
        let dist_gaus = Normal::new(width as f64 / 7.0, width as f64 / 14.0).unwrap();
        
        let positive_distr = Normal::new(65.0, 10.0).unwrap();
        let negative_distr = Normal::new(-35.0, 3.0).unwrap();
        
        let mut rng = Xoshiro256StarStar::seed_from_u64(seed);
        
        let mut extrema: Vec<Extremum> = Vec::with_capacity(extrema_count);
        
        while extrema.len() < extrema_count {
            let dist_min = dist_gaus.sample(&mut rng);
            
            let dist_from_center = Normal::new(avg_dist, avg_dist / 4.5).unwrap().sample(&mut rng);
            let angle = rng.gen_range(0.0..TAU);
            
            let x = center_x + dist_from_center * angle.cos();
            let y = center_y + dist_from_center * angle.sin();
            
            if extrema.iter().all(|e| (e.x - x).powi(2) + (e.y - y).powi(2) > dist_min.powi(2)) {
                let h = if extrema.len() < num_positive {
                    positive_distr
                } else {
                    negative_distr
                }.sample(&mut rng);
                
                extrema.push(Extremum { x, y, h });
            }
        }

        Self {
            extrema
        }
    }

    fn erfc(x: f64) -> f64 {
        let z = -1.0 * x.powi(2);
        z.exp() / 6.0 + (4.0 * z / 3.0).exp() / 2.0
    }

    pub fn get(&self, x: f64, y: f64) -> f64 {
        self.extrema.iter().fold(0.0, |acc, e| {
            let d = ((e.x - x).powi(2) + (e.y - y).powi(2)).sqrt();
            let i = 2.0 * d.powf(1.37) / e.h.abs().powf(1.5);

            acc + e.h * (2.0 - 2.0 / (Self::erfc(i).powf(0.1) + 1.0))
        }) / self.extrema.len() as f64
    }
}
