use rand::prelude::*;
use rand::distributions::{Distribution, Uniform};
use delaunator::{Point, Triangulation, triangulate};

pub fn new_delauney<R: Rng + ?Sized>(mut rng: &mut R, cells: usize, width: u32, height: u32) -> (Vec::<Point>, Triangulation) {
    let dist_x = Uniform::from(0..width);
    let dist_y = Uniform::from(0..height);

    let mut seeds = Vec::new();
    while seeds.len() < cells {
        let point = Point{
            x: dist_x.sample(&mut rng) as f64,
            y: dist_y.sample(&mut rng) as f64,
        };
        if !seeds.contains(&point) {
            seeds.push(point);
        }
    }

    let triangulation = triangulate(&seeds).unwrap();

    (seeds, triangulation)
}

pub fn edges_of_triangle(t: usize) -> [usize; 3] {
    [
        3 * t,
        3 * t + 1,
        3 * t + 2,
    ]
}
pub fn triangle_of_edge(e: usize) -> usize {
    e / 3
}
pub fn points_of_triangle(delauney: &Triangulation, t: usize) -> Vec::<usize> {
    edges_of_triangle(t)
        .iter()
        .map(|e| delauney.triangles[*e])
        .collect()
}

pub fn adjacent_triangles(delauney: &Triangulation, t: usize) -> Vec::<usize> {
    edges_of_triangle(t)
        .iter()
        .filter_map(|&e| {
            let opposite = delauney.halfedges[e];
            if opposite != delaunator::EMPTY {
                Some(triangle_of_edge(opposite))
            } else {
                None
            }
        })
        .collect()
}
