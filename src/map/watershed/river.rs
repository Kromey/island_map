use super::strahler::Strahler;

#[derive(Debug)]
pub struct River {
    river: Vec<usize>,
    order: Vec<Strahler>,
    branches: Vec<(usize, Self)>,
}

impl River {
    // pub fn new() -> Self {
    //     Self {
    //         river: Vec::new(),
    //         order: Vec::new(),
    //         branches: Vec::new(),
    //     }
    // }

    pub fn new_from(map: &crate::map::Map, start_idx: usize) -> Self {
        let mut river = vec![start_idx];

        let mut current = start_idx;
        loop {
            // Find the lowest neighbor
            let (x, y) = map.from_idx(current);
            if let Some(lowest) = map.get_neighbors(x, y)
                .into_iter()
                .filter_map(|(x, y)| {
                    let idx = map.to_idx(x, y);
                    if river.contains(&idx) {
                        None
                    } else {
                        Some(idx)
                    }
                })
                .min_by(|&idx1, &idx2| {
                    if map.elevation[idx1] < map.elevation[idx2] {
                        std::cmp::Ordering::Less
                    } else {
                        std::cmp::Ordering::Greater
                    }
                })
            {
                // // Make sure it's lower than us
                // if map.heightmap[lowest] > map.heightmap[current] {
                //     // TODO: Spawn a lake here
                //     break;
                // } else {
                //     current = lowest;
                // }
                current = lowest;

                // Check if we've reached water
                if map.elevation[current] <= crate::SEA_LEVEL {
                    break;
                }

                river.push(current);
            } else {
                break;
            }
        }

        // River is in source-to-mouth order, reverse for mouth-to-source
        river.reverse();

        // Convert this Vec into a River
        river.into()
    }

    pub fn merge(&mut self, other: River) {
        self.add_branch(other.river);
    }
    
    pub fn add_branch(&mut self, branch: Vec<usize>) {
        let (downstream, mut upstream): (Vec<usize>, Vec<usize>) = branch
            .into_iter()
            .partition(|i| self.river.contains(i));
            
        if upstream.len() == 0 || downstream.len() == self.river.len() {
            self.river.append(&mut upstream);
            self.order.resize_with(self.river.len(), Default::default);

            return;
        }
        
        let split_idx = downstream.len() - 1;
        
        for (idx, branch) in self.branches.iter_mut() {
            if *idx == split_idx && upstream[0] == branch.river[0] {
                branch.add_branch(upstream);
                self.update_order(split_idx);

                return;
            }
        }
            
        self.branches.push(
            (
                split_idx,
                Self::from(upstream),
            )
        );

        self.update_order(split_idx);
    }

    pub fn prune(&mut self) -> bool {
        if self.order() < Strahler::from(1) {
            return false;
        }

        // Prune all 0th-order river segments
        self.order.retain(|&s| s > Strahler::from(0));
        self.river.truncate(self.order.len());

        // If we're too short now, there's no reason to exist anymore
        if self.river.len() < 3 {
            return false;
        }

        // Now clean up our branches
        self.branches.retain(|(_, branch)| branch.order() > Strahler::from(0));
        for (_, branch) in self.branches.iter_mut() {
            branch.prune();
        }

        true
    }
    
    pub fn segments(&self) -> Vec<(usize, usize)> {
        let mut segments = Vec::new();
        let mut prev = self.river[0];
        for &cur in self.river.iter().skip(1) {
            segments.push((prev, cur));
            
            prev = cur;
        }
        
        for (idx, branch) in self.branches.iter() {
            segments.push((self.river[*idx], branch.river[0]));
            
            segments.append(&mut branch.segments());
        }
        
        segments
    }

    pub fn len(&self) -> usize {
        self.river.len()
    }

    pub fn mouth(&self) -> usize {
        self.river[0]
    }

    pub fn order(&self) -> Strahler {
        self.order[0]
    }

    fn update_order(&mut self, to_idx: usize) {
        let mut upstream = *self.order.get(to_idx + 1).unwrap_or(&Strahler::from(0));

        for (idx, order) in self.order.iter_mut().enumerate().take(to_idx + 1).rev() {
            let mut orders: Vec<_> = self.branches.iter().filter_map(|(split_idx, branch)| {
                if *split_idx == idx {
                    Some(branch.order())
                } else {
                    None
                }
            }).collect();
            if !orders.is_empty() {
                orders.push(upstream);

                // Sort our Strahler numbers in ascending order, then sum them.
                // This will let e.g. two 1st-order rivers joining a 2nd-order become 3rd-order
                // Because in Strahler terms, 1+1=2 and 2+2=3, but 2+1=2
                // Technically not accurate per the definition of Strahler number, which holds that
                // order i can only be achieved when two or more children are i-1
                orders.sort();

                upstream = orders.into_iter().reduce(std::ops::Add::add).unwrap(); // We know it's not empty so this will never panic
            }

            *order = upstream;
        }
    }
}

impl From<Vec<usize>> for River {
    fn from(river: Vec<usize>) -> River {
        let order = vec![Default::default(); river.len()];

        Self {
            river,
            order,
            branches: Vec::new(),
        }
    }
}
