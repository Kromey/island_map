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

pub fn circumcenter(a: &Point, b: &Point, c: &Point) -> Point {
    let ad = a.x.powi(2) + a.y.powi(2);
    let bd = b.x.powi(2) + b.y.powi(2);
    let cd = c.x.powi(2) + c.y.powi(2);

    let d = 2. * (a.x * (b.y - c.y) + b.x * (c.y - a.y) + c.x * (a.y - b.y));

    Point {
        x: 1. / d * (ad * (b.y - c.y) + bd * (c.y - a.y) + cd * (a.y - b.y)),
        y: 1. / d * (ad * (c.x - b.x) + bd * (a.x - c.x) + cd * (b.x - a.x)),
    }
}
pub fn triangle_center(points: &Vec::<Point>, delauney: &Triangulation, t: usize) -> Point {
    let p = points_of_triangle(delauney, t);
    circumcenter(&points[p[0]], &points[p[1]], &points[p[2]])
}
