use rand::prelude::*;
use rand::distributions::{Distribution, Uniform};
use rayon::prelude::*;

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
        // We're able to get massive performance gains from parallel processing,
        // however we have to do some non-trivial and non-intuitive things to make
        // it work:
        // First we process all (x,y) coordinates in parallel and store alongside
        // each one the *index* of the cell center it's closest to.
        let cells: Vec<(usize, (u32, u32))> = (0..self.width)
            .into_par_iter()
            // The .map().flatten() here is the magic that gives us an iterator
            // over all of our (x,y) points
            .map(|x| (0..self.height).into_par_iter().map(move |y| (x as i32, y as i32)))
            .flatten()
            .map(|(x, y)| {
                let (idx, _) = self.centers
                    .iter()
                    .enumerate()
                    .min_by_key(
                        |point| (point.1.0 as i32 - x).pow(2) + (point.1.1 as i32 - y).pow(2)
                    )
                    .unwrap();

                (idx, (x as u32, y as u32))
            })
            .collect();

        // Second, we re-process (again in parallel) that list we just built,
        // and for each cell center index we filter (yet again in parallel) our
        // master list down to just those points that belong to that cell.
        let mut membership = vec![Vec::<(u32, u32)>::new(); self.centers.len()];
        membership
            .par_iter_mut()
            .enumerate()
            .for_each(|(idx, v)| {
                // Here we extend our (empty) vectors using a parallel iterator
                // that filters our points and maps the results to just (x,y) pairs
                v.par_extend(cells.par_iter().filter_map(|d| {
                    if d.0 == idx {
                        Some(d.1)
                    } else {
                        None
                    }
                }));
            });

        self.cell_membership = membership;
    }
}