use delaunator::{Point, Triangulation, triangulate, next_halfedge};

mod poisson;
use poisson::generate_poisson;

/// A map represented by Voronoi polygons built from the Delaunay triangulation of random points.
/// 
/// The implementation of the Voronoi graph from its Delaunay triangulation is based on the article
/// and code at https://mapbox.github.io/delaunator/
pub struct Voronoi {
    pub width: u32,
    pub height: u32,
    pub seeds: Vec<Point>,
    pub delaunay: Triangulation,
    pub is_water: Vec<bool>,
}

impl Voronoi {
    /// Return a new Voronoi map.
    /// 
    /// # Arguments
    /// 
    /// * `seed` - The RNG seed for generating Voronoi "seeds", i.e. the points around which Voronoi cells are built
    /// * `num_cells` - The number of Voronoi "seeds" to create
    /// * `width` - The width of the map
    /// * `height` - The height of the map
    pub fn new(seed: u64, _num_cells: usize, width: u32, height: u32) -> Voronoi {
        let seeds: Vec<Point> = generate_poisson(width as f64, height as f64, 5., seed)
            .map(|p| Point { x: p.0, y: p.1 })
            .collect();

        println!("\t\tGenerated {} seed points", seeds.len());
    
        let delaunay = triangulate(&seeds).unwrap();
        let is_water = vec![false; seeds.len()];

        Voronoi {
            width,
            height,
            seeds,
            delaunay,
            is_water,
        }
    }

    /// For the specified triangle return its half-edges.
    /// 
    /// We define a triangle to be represented by the half-edges that run counter-clockwise around its perimeter.
    /// 
    /// # Arguments
    /// 
    /// * `triangle` - The index of the triangle to find half-edges for
    pub fn edges_of_triangle(&self, triangle: usize) -> [usize; 3] {
        [
            3 * triangle,
            3 * triangle + 1,
            3 * triangle + 2,
        ]
    }

    /// Find the triangle the given half-edge belongs to
    /// 
    /// # Arguments
    /// 
    /// * `halfedge` - The index of the halfedge we want to find the triangle for
    pub fn triangle_of_edge(&self, halfedge: usize) -> usize {
        halfedge / 3
    }

    /// Find the points for the given triangle
    /// 
    /// # Arguments
    /// 
    /// * `triangle` - The index of the triangle to find points for
    pub fn points_of_triangle(&self, triangle: usize) -> Vec::<usize> {
        self.edges_of_triangle(triangle)
            .iter()
            .map(|&e| self.delaunay.triangles[e])
            .collect()
    }

    /// Find the triangles adjacent to the given triangle
    /// 
    /// # Arguments
    /// 
    /// * `triangle` - The triangle to find neighbors for
    #[allow(dead_code)]
    pub fn adjacent_triangles(&self, triangle: usize) -> Vec::<usize> {
        self.edges_of_triangle(triangle)
            .iter()
            .filter_map(|&e| {
                let opposite = self.delaunay.halfedges[e];
                if opposite != delaunator::EMPTY {
                    Some(self.triangle_of_edge(opposite))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Find the circumcenter of the points `a`, `b`, and `c`.
    pub fn circumcenter(&self, a: &Point, b: &Point, c: &Point) -> Point {
        let ad = a.x.powi(2) + a.y.powi(2);
        let bd = b.x.powi(2) + b.y.powi(2);
        let cd = c.x.powi(2) + c.y.powi(2);
    
        let d = 2. * (a.x * (b.y - c.y) + b.x * (c.y - a.y) + c.x * (a.y - b.y));
    
        Point {
            x: 1. / d * (ad * (b.y - c.y) + bd * (c.y - a.y) + cd * (a.y - b.y)),
            y: 1. / d * (ad * (c.x - b.x) + bd * (a.x - c.x) + cd * (b.x - a.x)),
        }
    }

    /// Find the circumcenter of the given triangle
    pub fn triangle_center(&self, triangle: usize) -> Point {
        let p = self.points_of_triangle(triangle);
        self.circumcenter(&self.seeds[p[0]], &self.seeds[p[1]], &self.seeds[p[2]])
    }

    /// Find the edges that point in to the specified start point.
    pub fn edges_around_point(&self, start: usize) -> Vec<usize> {
        let mut result = Vec::new();
        let mut incoming = start;
        loop {
            result.push(incoming);
            let outgoing = next_halfedge(incoming);
            incoming = self.delaunay.halfedges[outgoing];
    
            if incoming == delaunator::EMPTY || incoming == start {
                break;
            }
        }
    
        result
    }
}
