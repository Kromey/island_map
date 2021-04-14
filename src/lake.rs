use super::Biome;
use super::river::Strahler;
use super::voronoi::Voronoi;

#[allow(dead_code)]
pub struct Lake {
    cells: Vec<usize>,
    height: f64,
    order: Strahler,
}

impl Lake {
    pub fn new_at(start: usize, map: &Voronoi) -> Self {
        let mut lake = Self {
            height: map.cells[start].height,
            cells: vec![start],
            order: Default::default(),
        };

        loop {
            let neighbor = lake.lowest_neighbor(map);
            let neighbor_height = map.cells[neighbor].height;
            
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

    pub fn lowest_neighbor(&self, map: &Voronoi) -> usize {
        self.cells
            .iter()
            .flat_map(|&cell| map.neighbors_of_point(cell))
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

    pub fn apply(&self, map: &mut Voronoi) {
        for &cell in self.cells.iter() {
            map.cells[cell].height = self.height;
            map.cells[cell].biome = Biome::Lake;
        }
    }
}