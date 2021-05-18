use super::strahler::Strahler;
use crate::Map;

pub struct Lake {
    cells: Vec<usize>,
    height: f64,
    #[allow(dead_code)]
    order: Strahler,
}

impl Lake {
    pub fn new_at(start: usize, map: &Map) -> Self {
        let mut lake = Self {
            height: map.heightmap[start],
            cells: vec![start],
            order: Default::default(),
        };

        loop {
            let neighbor = lake.lowest_neighbor(map);
            let neighbor_height = map.heightmap[neighbor];
            
            if neighbor_height < lake.height {
                // We've expanded to reach a lower-height neighbor, we're done!
                break;
            }

            // Fill our lake to this new cell
            lake.height = neighbor_height;
            // Add this cell to our lake
            lake.cells.push(neighbor);
        }

        lake
    }

    pub fn lowest_neighbor(&self, map: &Map) -> usize {
        self.cells
            .iter()
            .flat_map(|&cell| map.get_neighbors(cell as u32, cell as u32))
            .filter_map(|neighbor| {
                if !self.cells.contains(&neighbor) {
                    Some((neighbor, map.cells[neighbor].height))
                } else {
                    None
                }
            })
            .reduce(|a, b| if a.1 < b.1 { a } else { b })
            .unwrap()
            .0
    }

    pub fn apply(&self, map: &mut Map) {
        for &cell in self.cells.iter() {
            map.cells[cell].height = self.height;
            map.cells[cell].biome = Biome::Lake;
        }
    }
}