use rand::prelude::*;
use rand::distributions::{Distribution, Uniform};
use rand_xoshiro::Xoshiro256StarStar;
use delaunator::{Point, Triangulation, triangulate, next_halfedge};

pub struct Voronoi {
    pub width: u32,
    pub height: u32,
    pub seeds: Vec<Point>,
    pub delaunay: Triangulation,
    pub is_water: Vec<bool>,
}

impl Voronoi {
    pub fn new(seed: u64, num_cells: usize, width: u32, height: u32) -> Voronoi {
        let mut rng = Xoshiro256StarStar::seed_from_u64(seed);
        let dist_x = Uniform::from(0..width);
        let dist_y = Uniform::from(0..height);
    
        let mut seeds = Vec::new();
        while seeds.len() < num_cells {
            let point = Point{
                x: dist_x.sample(&mut rng) as f64,
                y: dist_y.sample(&mut rng) as f64,
            };
            if !seeds.contains(&point) {
                seeds.push(point);
            }
        }
    
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

    pub fn edges_of_triangle(&self, triangle: usize) -> [usize; 3] {
        [
            3 * triangle,
            3 * triangle + 1,
            3 * triangle + 2,
        ]
    }

    pub fn triangle_of_edge(&self, e: usize) -> usize {
        e / 3
    }

    pub fn points_of_triangle(&self, triangle: usize) -> Vec::<usize> {
        self.edges_of_triangle(triangle)
            .iter()
            .map(|&e| self.delaunay.triangles[e])
            .collect()
    }

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

    pub fn triangle_center(&self, triangle: usize) -> Point {
        let p = self.points_of_triangle(triangle);
        self.circumcenter(&self.seeds[p[0]], &self.seeds[p[1]], &self.seeds[p[2]])
    }

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
