use rand::prelude::*;
use rand::distributions::{Distribution, Uniform};
use delaunator::{Point, Triangulation, triangulate, next_halfedge};

pub fn new_delaunay<R: Rng + ?Sized>(mut rng: &mut R, cells: usize, width: u32, height: u32) -> (Vec::<Point>, Triangulation) {
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

pub fn edges_of_triangle(triangle: usize) -> [usize; 3] {
    [
        3 * triangle,
        3 * triangle + 1,
        3 * triangle + 2,
    ]
}
pub fn triangle_of_edge(e: usize) -> usize {
    e / 3
}
pub fn points_of_triangle(delaunay: &Triangulation, triangle: usize) -> Vec::<usize> {
    edges_of_triangle(triangle)
        .iter()
        .map(|e| delaunay.triangles[*e])
        .collect()
}

pub fn adjacent_triangles(delaunay: &Triangulation, triangle: usize) -> Vec::<usize> {
    edges_of_triangle(triangle)
        .iter()
        .filter_map(|&e| {
            let opposite = delaunay.halfedges[e];
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
pub fn triangle_center(points: &Vec::<Point>, delaunay: &Triangulation, triangle: usize) -> Point {
    let p = points_of_triangle(delaunay, triangle);
    circumcenter(&points[p[0]], &points[p[1]], &points[p[2]])
}

pub fn edges_around_point(delaunay: &Triangulation, start: usize) -> Vec<usize> {
    let mut result = Vec::new();
    let mut incoming = start;
    loop {
        result.push(incoming);
        let outgoing = next_halfedge(incoming);
        incoming = delaunay.halfedges[outgoing];

        if incoming == delaunator::EMPTY || incoming == start {
            break;
        }
    }

    result
}
