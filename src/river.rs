#[derive(Debug)]
pub struct River {
    river: Vec<usize>,
    branches: Vec<(usize, Self)>,
}

impl River {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            river: Vec::new(),
            branches: Vec::new(),
        }
    }
    
    pub fn add_branch(&mut self, branch: Vec<usize>) {
        let (downstream, mut upstream): (Vec<usize>, Vec<usize>) = branch
            .into_iter()
            .partition(|i| self.river.contains(i));
            
        if upstream.len() == 0 || downstream.len() == self.river.len() {
            self.river.append(&mut upstream);
            return;
        }
        
        let offset = downstream.len() - 1;
        
        for (depth, network) in self.branches.iter_mut() {
            if *depth == offset && upstream[0] == network.river[0] {
                network.add_branch(upstream);
                return;
            }
        }
            
        self.branches.push(
            (
                offset,
                Self::from(upstream),
            )
        );
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
}

impl From<Vec<usize>> for River {
    fn from(river: Vec<usize>) -> River {
        Self {
            river,
            branches: Vec::new(),
        }
    }
}
