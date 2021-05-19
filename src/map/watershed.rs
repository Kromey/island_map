use std::cmp::Reverse;

//pub mod lake;
pub mod river;
pub mod strahler;

use river::River;

const WATERSHED_START: f64 = 0.8;

#[derive(Debug)]
pub struct Watershed {
    river: river::River,
}

impl Watershed {
    pub fn create_all(map: &super::Map) -> Vec<Watershed> {
        let mut seeds: Vec<_> = map.heightmap
            .iter()
            .enumerate()
            .filter(|&(_, &h)| h > WATERSHED_START)
            .collect();
        
        seeds.sort_unstable_by(|a, b| {
            // Arguments are reversed because we want a reversed sort
            b.1.partial_cmp(a.1).unwrap()
        });

        if seeds.len() > 5 {
            seeds.resize(5, (0, &0.0));
        }

        let mut rivers = Vec::new();
        while let Some(start) = seeds.pop() {
            let (idx, _) = start;
            rivers.push(River::new_from(map, idx));
        }

        // Sort rivers so that we start with the longest
        rivers.sort_unstable_by_key(|river| Reverse(river.len()));

        // Merge rivers
        let mut merged = Vec::new();
        for river in rivers {
            match merged.iter_mut().find(|r: &&mut River| r.mouth() == river.mouth()) {
                Some(parent_river) => parent_river.merge(river),
                None => merged.push(river),
            }
        }

        merged.into_iter().filter_map(|mut river| {
            if river.prune() {
                Some(
                    Watershed {
                        river
                    }
                )
            } else {
                None
            }
        }).collect()
    }

    pub fn river_segments(&self) -> Vec<(usize, usize)> {
        self.river.segments()
    }
}
