use std::ops::{Add, AddAssign};

/// Strahler Number
/// 
/// https://en.wikipedia.org/wiki/Strahler_number#River_networks
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Strahler(pub usize);

impl Add for Strahler {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        if self == other {
            Self(self.0 + 1)
        } else {
            Self(self.0.max(other.0))
        }
    }
}

impl AddAssign for Strahler {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl Default for Strahler {
    fn default() -> Self {
        Strahler(1)
    }
}

#[derive(Debug)]
pub struct River {
    river: Vec<usize>,
    order: Vec<Strahler>,
    branches: Vec<(usize, Self)>,
}

impl River {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            river: Vec::new(),
            order: Vec::new(),
            branches: Vec::new(),
        }
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

    pub fn mouth(&self) -> usize {
        self.river[0]
    }

    pub fn order(&self) -> Strahler {
        self.order[0]
    }

    fn update_order(&mut self, to_idx: usize) {
        let mut upstream = *self.order.get(to_idx + 1).unwrap_or(&Strahler(0));

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

                upstream = orders.into_iter().reduce(Add::add).unwrap(); // We know it's not empty so this will never panic
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
