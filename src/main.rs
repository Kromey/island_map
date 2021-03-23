use image;
use imageproc::drawing::{/*draw_hollow_circle, draw_line_segment,*/ draw_polygon, draw_filled_rect};
use imageproc::rect::Rect;
use noise::{Fbm, Seedable, NoiseFn};
use std::time::Instant;
use delaunator::next_halfedge;

mod voronoi;
use voronoi::Voronoi;

fn draw_voronoi(vor: &Voronoi, imgx: u32, imgy: u32, i: u64) {
    let mut img = image::ImageBuffer::new(imgx as u32, imgy as u32);
    let sand = image::Rgb([194u8, 178, 128]);
    let water = image::Rgb([0u8, 0, 255]);
    //let edge = image::Rgb([0u8, 0, 0]);
    //let delaunay_point = image::Rgb([255u8, 0, 0]);
    //let voronoi_corner = image::Rgb([0u8, 0, 255]);

    /*for triangle in 0..delaunay.triangles.len() / 3 {
        let p: Vec::<imageproc::point::Point::<i32>> = voronoi2::points_of_triangle(delaunay, triangle)
            .iter()
            .map(|&e| imageproc::point::Point::new(points[e].x as i32, points[e].y as i32))
            .collect();

        img = draw_polygon(&img, &p, fill);
    }*/

    /*for e in 0..delaunay.triangles.len() {
        if e > delaunay.halfedges[e] {
            let p = &points[delaunay.triangles[e]];
            let q = &points[delaunay.triangles[next_halfedge(e)]];

            img = draw_line_segment(&img, (p.x as f32, p.y as f32), (q.x as f32, q.y as f32), color_edge);
        }
    }*/

    img = draw_filled_rect(&img, Rect::at(0, 0).of_size(imgx, imgy), water);

    println!("\t\tDrawing Voronoi polygons...");
    let mut seen = vec![false; vor.delaunay.triangles.len()];
    for e in 0..vor.delaunay.triangles.len() {
        let p = vor.delaunay.triangles[next_halfedge(e)];

        if !seen[p] {
            seen[p] = true;
            let edges = vor.edges_around_point(e);
            let triangles: Vec::<usize> = edges.iter().map(|&e| vor.triangle_of_edge(e)).collect();
            let mut vertices: Vec::<imageproc::point::Point::<i32>> = triangles.iter().map(|&t| {
                let p = vor.triangle_center(t);
                imageproc::point::Point::new(p.x.round() as i32, p.y.round() as i32)
            }).collect();
            vertices.dedup();
            if vertices[0] == vertices[vertices.len() - 1] {
                vertices.pop();
            }
            //println!("{:?}", vertices);

            let fill = if vor.is_water[p] {
                water
            } else {
                sand
            };
            img = draw_polygon(&img, &vertices, fill);
        }

        //let p = &vor.seeds[p];
        //img = draw_hollow_circle(&img, (p.x as i32, p.y as i32), 1, delaunay_point);
    }

    /*println!("\t\tDrawing Voronoi edges...");
    for e in 0..vor.delaunay.triangles.len() {
        if e > vor.delaunay.halfedges[e] {
            let p = vor.triangle_center(vor.triangle_of_edge(e));
            let q = vor.triangle_center(vor.triangle_of_edge(vor.delaunay.halfedges[e]));

            img = draw_line_segment(&img, (p.x as f32, p.y as f32), (q.x as f32, q.y as f32), edge);
        }
    }*/

    /*println!("\t\tDrawing Voronoi seeds...");
    for &Point{x, y} in vor.seeds.iter() {
        img = draw_hollow_circle(&img, (x as i32, y as i32), 1, delaunay_point);
    }*/

    /*println!("\t\tDrawing Voronoi corners...");
    for triangle in 0..vor.delaunay.triangles.len() / 3 {
        let Point{x, y} = vor.triangle_center(triangle);

        img = draw_hollow_circle(&img, (x.round() as i32, y.round() as i32), 1, voronoi_corner);
    }*/

    println!("\t\tSaving...");
    img.save(format!("map_{:02}.png", i+1)).unwrap();
}

fn main() {
    let imgx = 400;
    let imgy = 400;

    for seed in 0..12 {
        let mut map_duration = 0.;

        println!("Generating Voronoi graph...");
        let start = Instant::now();
        let mut v = Voronoi::new(seed, 256, imgx, imgy);
        let duration = start.elapsed().as_secs_f64();
        println!("\tDone! ({:.2} seconds)", duration);
        map_duration += duration;

        println!("Defining water/land boundaries...");
        let start = Instant::now();
        let fbm = Fbm::new().set_seed(seed as u32);
        for (idx, p) in v.seeds.iter().enumerate() {
            let x = (p.x as i32 - imgx as i32 / 2) as f64 / (imgx / 2) as f64;
            let y = (p.y as i32 - imgy as i32 / 2) as f64 / (imgy / 2) as f64;
            let d = x.powi(2) + y.powi(2);
            let n = fbm.get([x, y]);

            v.is_water[idx] = n + d > 0.5;
            // TODO: Force perimeter cells to ocean
        }
        let duration = start.elapsed().as_secs_f64();
        println!("\tDone! ({:.2} seconds)", duration);
        map_duration += duration;

        let start = Instant::now();
        println!("\tDrawing...");
        draw_voronoi(&v, imgx, imgy, seed);
        let duration = start.elapsed();
        println!("\tDone! ({:.2} seconds)", duration.as_secs_f64());

        println!("Generated map in {:.2} seconds\n", map_duration);
    }
}
