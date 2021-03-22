use rand::prelude::*;
use rand_xoshiro::Xoshiro256StarStar;
use image;
use imageproc::drawing::{draw_hollow_circle, draw_line_segment, draw_polygon};
use noise::{Fbm, Seedable, NoiseFn};
use std::time::{Instant, Duration};
use delaunator::{Point, Triangulation, next_halfedge};

mod voronoi;
use voronoi::Voronoi;
mod voronoi2;
use voronoi2::new_delaunay;

fn draw_voronoi(vor: &Voronoi, imgx: u32, imgy: u32, i: usize, show_water: bool) {
    let mut img = image::ImageBuffer::new(imgx as u32, imgy as u32);
    let interior = image::Rgb([255u8, 255, 255]);
    let frontier = image::Rgb([0u8, 0, 0]);
    let sand = image::Rgb([194u8, 178, 128]);
    let water = image::Rgb([0u8, 0, 255]);

    for (idx, points) in vor.cell_membership.iter().enumerate() {

        for p in points.iter() {
            let pixel = img.get_pixel_mut(p.0, p.1);
            *pixel = interior;

            if show_water {
                if vor.is_water[idx] {
                    *pixel = water;
                } else {
                    *pixel = sand;
                }
            }

            'neighbors: for dx in -1..1 {
                if p.0 == 0 && dx == -1 {
                    continue;
                }
                for dy in -1..1 {
                    if p.1 == 0 && dy == -1 {
                        continue;
                    }
                    if dx == 0 && dy == 0 {
                        continue;
                    }

                    let neighbor = ((p.0 as i32 + dx) as u32, (p.1 as i32 + dy) as u32);
                    if !points.contains(&neighbor) {
                        *pixel = frontier;
                        break 'neighbors;
                    }
                }
            }
        }
    }

    for center in vor.centers.iter() {
        img = draw_hollow_circle(&img, (center.0 as i32, center.1 as i32), 1, image::Rgb([255u8, 0, 0]))
    }

    img.save(format!("map_{:02}.png", i)).unwrap();
}

fn draw_delauney(points: &Vec::<Point>, delauney: &Triangulation, imgx: u32, imgy: u32) {
    let mut img = image::ImageBuffer::new(imgx as u32, imgy as u32);
    let fill = image::Rgb([221u8, 221, 213]);
    let color_edge = image::Rgb([0u8, 0, 0]);
    let delauney_point = image::Rgb([255u8, 0, 0]);
    let voronoi_corner = image::Rgb([0u8, 0, 255]);

    /*for triangle in 0..delauney.triangles.len() / 3 {
        let p: Vec::<imageproc::point::Point::<i32>> = voronoi2::points_of_triangle(delauney, triangle)
            .iter()
            .map(|&e| imageproc::point::Point::new(points[e].x as i32, points[e].y as i32))
            .collect();

        img = draw_polygon(&img, &p, fill);
    }*/

    /*for e in 0..delauney.triangles.len() {
        if e > delauney.halfedges[e] {
            let p = &points[delauney.triangles[e]];
            let q = &points[delauney.triangles[next_halfedge(e)]];

            img = draw_line_segment(&img, (p.x as f32, p.y as f32), (q.x as f32, q.y as f32), color_edge);
        }
    }*/

    println!("\t\tDrawing Voronoi polygons...");
    let mut seen = vec![false; delauney.triangles.len()];
    for e in 0..delauney.triangles.len() {
        let p = delauney.triangles[next_halfedge(e)];

        if !seen[p] {
            seen[p] = true;
            let edges = voronoi2::edges_around_point(delauney, e);
            let triangles: Vec::<usize> = edges.iter().map(|&e| voronoi2::triangle_of_edge(e)).collect();
            let mut vertices: Vec::<imageproc::point::Point::<i32>> = triangles.iter().map(|&t| {
                let p = voronoi2::triangle_center(points, delauney, t);
                imageproc::point::Point::new(p.x.round() as i32, p.y.round() as i32)
            }).collect();
            vertices.dedup();
            if vertices[0] == vertices[vertices.len() - 1] {
                vertices.pop();
            }
            //println!("{:?}", vertices);

            img = draw_polygon(&img, &vertices, fill);
        }
    }

    println!("\t\tDrawing Voronoi edges...");
    for e in 0..delauney.triangles.len() {
        if e > delauney.halfedges[e] {
            let p = voronoi2::triangle_center(points, delauney, voronoi2::triangle_of_edge(e));
            let q = voronoi2::triangle_center(points, delauney, voronoi2::triangle_of_edge(delauney.halfedges[e]));

            img = draw_line_segment(&img, (p.x as f32, p.y as f32), (q.x as f32, q.y as f32), voronoi_corner);
        }
    }

    println!("\t\tDrawing Voronoi seeds...");
    for &Point{x, y} in points.iter() {
        img = draw_hollow_circle(&img, (x as i32, y as i32), 1, delauney_point);
    }

    println!("\t\tDrawing Voronoi corners...");
    for triangle in 0..delauney.triangles.len() / 3 {
        let Point{x, y} = voronoi2::triangle_center(points, delauney, triangle);

        img = draw_hollow_circle(&img, (x.round() as i32, y.round() as i32), 1, voronoi_corner);
    }

    println!("\t\tSaving...");
    img.save("map_delauney.png").unwrap();
}

fn main() {
    let imgx = 400;
    let imgy = 400;

    println!("Generating Delauney triangulation...");
    let start = Instant::now();
    let mut rng = Xoshiro256StarStar::seed_from_u64(0);
    let (points, delauney) = new_delaunay(&mut rng, 256, imgx, imgy);
    let duration = start.elapsed();
    println!("\tDone! ({:.2} seconds)", duration.as_secs_f64());
    let start = Instant::now();
    println!("\tDrawing...");
    draw_delauney(&points, &delauney, imgx, imgy);
    let duration = start.elapsed();
    println!("\tDone! ({:.2} seconds)", duration.as_secs_f64());

    /*for seed in 0..12 {
        println!("Generating map {}...", seed);
        let mut map_duration = Duration::default();

        let mut rng = Xoshiro256StarStar::seed_from_u64(seed);
    
        println!("Generating Voronoi...");
        let start = Instant::now();
        let mut v = Voronoi::new(&mut rng, 256, imgx, imgy);
        let duration = Instant::now() - start;
        map_duration += duration;
        println!("\tDone! ({:.2} seconds)", duration.as_secs_f64());
        if seed == 0 {
            println!("\tDrawing...");
            draw_voronoi(&v, imgx, imgy, 1, false);
        }

        let relax_n = 2;
        println!("Performing {} iterations of Lloyd relaxation...", relax_n);
        let start = Instant::now();
        for _ in 0..relax_n {
            v.improve_centers();
        }
        let duration = Instant::now() - start;
        map_duration += duration;
        println!("\tDone! ({:.2} seconds)", duration.as_secs_f64());
        if seed == 0 {
            println!("\tDrawing...");
            draw_voronoi(&v, imgx, imgy, 2, false);
        }

        println!("Defining water/land boundaries...");
        let start = Instant::now();
        let fbm = Fbm::new().set_seed(seed as u32);
        for (idx, (px, py)) in v.centers.iter().enumerate() {
            let x = (*px as i32 - imgx as i32 / 2) as f64 / (imgx / 2) as f64;
            let y = (*py as i32 - imgy as i32 / 2) as f64 / (imgy / 2) as f64;
            let d = x.powi(2) + y.powi(2);
            let n = fbm.get([x, y]);
            //print!("{} ", n);

            v.is_water[idx] = n + d > 0.5;
        }
        for (idx, points) in v.cell_membership.iter().enumerate() {
            if v.is_water[idx] {
                continue;
            }
            if points.iter().any(|&point| point.0 <= 5 || point.1 <= 5 || point.0 >= imgx - 6 || point.1 >= imgy - 6) {
                v.is_water[idx] = true;
            }
        }
        let duration = Instant::now() - start;
        map_duration += duration;
        println!("\tDone! ({:.2} seconds)", duration.as_secs_f64());

        println!("\tDrawing...");
        draw_voronoi(&v, imgx, imgy, 3 + seed as usize, true);
        println!("Finished in {:.2} seconds\n", map_duration.as_secs_f64());
    }*/
}
