use rand::prelude::*;
use rand::distributions::{Distribution, Uniform};

pub struct Voronoi {
    pub width: u32,
    pub height: u32,
    pub centers: Vec<(u32, u32)>,
    pub cell_membership: Vec<Vec<(u32, u32)>>,
    pub is_water: Vec<bool>,
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
            is_water: vec![true; cells],
        };
        vor.calc_membership();

        vor
    }

    pub fn improve_centers(&mut self) {
        let mut centers = Vec::new();
        for points in self.cell_membership.iter() {
            let acc = (0, (0, 0));
            let (count, sum) = points.iter().fold(acc, |acc, p| (acc.0 + 1, (acc.1.0 + p.0, acc.1.1 + p.1)));
            let new_x = sum.0 as f32 / count as f32;
            let new_y = sum.1 as f32 / count as f32;
            centers.push((new_x.round() as u32, new_y.round() as u32));
        }

        self.centers = centers;
        self.calc_membership();
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