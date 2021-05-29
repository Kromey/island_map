use nalgebra as na;
use rand::prelude::*;
use rand_xoshiro::Xoshiro256StarStar;

mod elevation;
mod erosion;
mod gradient;
//mod watershed;
use elevation::Elevation;

pub const SEA_LEVEL: f64 = 0.0;

pub struct Map {
    size: u32,
    rng: Xoshiro256StarStar,
    elevation: Elevation,
    //watersheds: Vec<watershed::Watershed>,
}

impl Map {
    pub fn new(seed: u64, size: u32) -> Self {
        let mut rng = Xoshiro256StarStar::seed_from_u64(seed);
        let elevation = Elevation::new(&mut rng, size);

        let map = Map {
            size,
            rng,
            elevation,
            //watersheds: Vec::new(),
        };

        //map.watersheds = watershed::Watershed::create_all(&map);

        map
    }

    pub fn erode(&mut self, cycles: u32) {
        erosion::erode(&mut self.elevation, &mut self.rng, cycles);
    }

    #[allow(dead_code)]
    #[inline(always)]
    fn to_idx(&self, x: u32, y: u32) -> usize {
        self.elevation.to_idx(x, y)
    }

    #[allow(dead_code)]
    #[inline(always)]
    fn from_idx(&self, idx: usize) -> (u32, u32) {
        self.elevation.from_idx(idx)
    }

    #[inline(always)]
    pub fn size(&self) -> u32 {
        self.size
    }

    // fn get_neighbors(&self, x: u32, y: u32) -> impl Iterator<Item = (u32, u32)> {
    //     let width = self.width;
    //     let height = self.height;

    //     vec![
    //         (x.wrapping_sub(1), y.wrapping_sub(1)),
    //         (x.wrapping_sub(1), y),
    //         (x.wrapping_sub(1), y.wrapping_add(1)),
    //         (x, y.wrapping_sub(1)),
    //         (x, y.wrapping_add(1)),
    //         (x.wrapping_add(1), y.wrapping_sub(1)),
    //         (x.wrapping_add(1), y),
    //         (x.wrapping_add(1), y.wrapping_add(1)),
    //     ]
    //     .into_iter()
    //     .filter(move |(x, y)| *x < width && *y < height)
    // }

    #[allow(dead_code)]
    pub fn get_coast<'a>(&'a self) -> &'a Vec<(u32, u32)> {
        &self.elevation.get_coast()
    }

    // pub fn get_river_segments(&self) -> Vec<((u32, u32), (u32, u32))> {
    //     self.watersheds
    //         .iter()
    //         .map(|watershed| watershed.river_segments())
    //         .flatten()
    //         .map(|(start, end)| {
    //             let (x1, y1) = self.from_idx(start);
    //             let (x2, y2) = self.from_idx(end);

    //             ((x1, y1), (x2, y2))
    //         })
    //         .collect()
    // }

    #[inline(always)]
    pub fn get_elevation(&self, x: u32, y: u32) -> f64 {
        self.elevation[(x, y)]
    }

    pub fn get_normal(&self, x: u32, y: u32) -> na::Vector3<f64> {
        self.elevation.get_normal(x, y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_and_from_idx() {
        let size = 20;
        let map = Map::new(1, size);

        for x in 0..size {
            for y in 0..size {
                let idx = map.to_idx(x, y);
                let (x2, y2) = map.from_idx(idx);
                let idx2 = map.to_idx(x2, y2);

                assert_eq!(
                    (x, y),
                    (x2, y2),
                    "{:?} and {:?} aren't the same!",
                    (x, y),
                    (x2, y2)
                );
                assert_eq!(idx, idx2, "idx and idx2 aren't the same!");
            }
        }
    }
}
