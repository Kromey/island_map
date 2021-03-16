use rand::prelude::*;
use rand::distributions::{Distribution, Uniform};

pub struct Voronoi {
    pub width: u32,
    pub height: u32,
    pub centers: Vec<(u32, u32)>,
    pub cell_membership: Vec<Vec<(u32, u32)>>,
}

impl Voronoi {
    pub fn new<R: Rng + ?Sized>(mut rng: &mut R, cells: usize, width: u32, height: u32) -> Voronoi {
        let dist_x = Uniform::from(0..width);
        let dist_y = Uniform::from(0..height);

        let mut centers = Vec::new();
        while centers.len() < cells {
            let point = (dist_x.sample(&mut rng), dist_y.sample(&mut rng));
            if !centers.contains(&point) {
                centers.push(point);
            }
        }

        let mut vor = Voronoi {
            width,
            height,
            centers,
            cell_membership: Vec::new(),
        };
        vor.calc_membership();

        vor
    }

    fn calc_membership(&mut self) {
        let mut membership = vec![Vec::<(u32, u32)>::new(); self.centers.len()];
        for x in 0..self.width as i32 {
            for y in 0..self.height as i32 {
                let (idx, _) = self.centers
                    .iter()
                    .enumerate()
                    .min_by_key(
                        |point| (point.1.0 as i32 - x).pow(2) + (point.1.1 as i32 - y).pow(2)
                    )
                    .unwrap();

                membership[idx].push((x as u32, y as u32));
            }
        }

        self.cell_membership = membership;
    }
}