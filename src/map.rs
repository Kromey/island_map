use rand::prelude::*;
use rand_xoshiro::Xoshiro256StarStar;

mod elevation;
mod gradient;
//mod watershed;
use elevation::Elevation;

pub const SEA_LEVEL: f64 = 0.0;

pub struct Map {
    width: u32,
    height: u32,
    //rng: Xoshiro256StarStar,
    elevation: Elevation,
    //watersheds: Vec<watershed::Watershed>,
}

impl Map {
    pub fn new(seed: u64, width: u32, height: u32) -> Self {
        let mut rng = Xoshiro256StarStar::seed_from_u64(seed);
        let elevation = Elevation::new(&mut rng, width, height);

        let mut map = Map {
            width,
            height,
            //rng,
            elevation,
            //watersheds: Vec::new(),
        };

        //map.watersheds = watershed::Watershed::create_all(&map);

        map
    }

    fn to_idx(&self, x: u32, y: u32) -> usize {
        (x * self.height() + y) as usize
    }

    fn from_idx(&self, idx: usize) -> (u32, u32) {
        let idx = idx as u32;

        (idx / self.height(), idx % self.height())
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
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

    pub fn get_elevation(&self, x: u32, y: u32) -> f64 {
        self.elevation[(x, y)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_and_from_idx() {
        let map = Map::new(1, 20, 20);

        for x in 0..20 {
            for y in 0..20 {
                let idx = map.to_idx(x, y);
                let (x2, y2) = map.from_idx(idx);
                let idx2 = map.to_idx(x2, y2);

                //eprintln!("({},{}) ⇒ {} ⇒ ({},{}) ⇒ {}", x, y, idx, x2, y2, idx2);

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

    #[test]
    fn to_and_from_uneven_idx() {
        let map = Map::new(1, 20, 35);

        for x in 0..20 {
            for y in 0..35 {
                let idx = map.to_idx(x, y);
                let (x2, y2) = map.from_idx(idx);
                let idx2 = map.to_idx(x2, y2);

                //eprintln!("({},{}) ⇒ {} ⇒ ({},{}) ⇒ {}", x, y, idx, x2, y2, idx2);

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

        let map = Map::new(1, 35, 20);

        for x in 0..35 {
            for y in 0..20 {
                let idx = map.to_idx(x, y);
                let (x2, y2) = map.from_idx(idx);
                let idx2 = map.to_idx(x2, y2);

                //eprintln!("({},{}) ⇒ {} ⇒ ({},{}) ⇒ {}", x, y, idx, x2, y2, idx2);

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
