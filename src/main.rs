use fast_poisson::Poisson2D;
use imageproc::drawing::{draw_filled_rect, draw_polygon};
use imageproc::rect::Rect;
use lerp::Lerp;
use noise::{Fbm, NoiseFn, Seedable};
use std::time::Instant;

mod voronoi;
use voronoi::Voronoi;

fn draw_voronoi(vor: &Voronoi, img_x: u32, img_y: u32, i: u64) {
    let mut img = image::ImageBuffer::new(img_x as u32, img_y as u32);
    let sand = image::Rgb([194_u8, 178, 128]);
    let water = image::Rgb([48_u8, 96, 130]);
    let coastal_waters = image::Rgb([203_u8, 219, 252]);
    //let edge = image::Rgb([0_u8, 0, 0]);
    //let delaunay_point = image::Rgb([255_u8, 0, 0]);
    //let voronoi_corner = image::Rgb([0_u8, 0, 255]);

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
            let q = &points[delaunay.triangles[vor.next_halfedge(e)]];

            img = draw_line_segment(&img, (p.x as f32, p.y as f32), (q.x as f32, q.y as f32), color_edge);
        }
    }*/

    img = draw_filled_rect(&img, Rect::at(0, 0).of_size(img_x, img_y), water);

    let mut preprocessing = 0.;
    let mut drawing = 0.;

    println!("\t\tDrawing Voronoi polygons...");
    let mut seen = vec![false; vor.delaunay.triangles.len()];
    for e in 0..vor.delaunay.triangles.len() {
        let p = vor.delaunay.triangles[vor.next_halfedge(e)];

        if !seen[p] {
            let start = Instant::now();
            seen[p] = true;
            let edges = vor.edges_around_point(e);
            let is_coast = vor.is_water[p]
                && edges.iter().any(|&e| {
                    !vor.is_water[vor.delaunay.triangles[e]]
                });
            let triangles: Vec<usize> = edges.iter().map(|&e| vor.triangle_of_edge(e)).collect();
            let mut vertices: Vec<imageproc::point::Point<i32>> = triangles
                .iter()
                .map(|&t| {
                    let p = vor.triangle_center(t);
                    imageproc::point::Point::new(p.x.round() as i32, p.y.round() as i32)
                })
                .collect();
            vertices.dedup();
            if vertices[0] == vertices[vertices.len() - 1] {
                vertices.pop();
            }
            preprocessing += start.elapsed().as_secs_f64();
            //println!("{:?}", vertices);

            let start = Instant::now();
            let fill = if is_coast {
                coastal_waters
            } else if vor.is_water[p] {
                water
            } else {
                sand
            };
            img = draw_polygon(&img, &vertices, fill);
            drawing += start.elapsed().as_secs_f64();
        }

        /*
        let point = &vor.seeds[p];
        let color = if vor.delaunay.hull.contains(&p) {
            image::Rgb([255_u8, 0, 0])
        } else {
            image::Rgb([0_u8, 255, 0])
        };
        img = draw_hollow_circle(&img, (point.x as i32, point.y as i32), 1, color);
        */
    }
    println!(
        "\t\t\tPreprocessing: {} seconds\n\t\t\tDrawing: {} seconds",
        preprocessing, drawing
    );

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
    img.save(format!("map_{:02}.png", i + 1)).unwrap();
}

fn main() {
    let img_x = 400;
    let img_y = 400;

    for [x, y] in Poisson2D::new() {
        print!("({:.2}, {:.2}), ", x, y);
    }
    println!();

    let center_x = f64::from(img_x / 2);
    let center_y = f64::from(img_y / 2);

    for seed in 0..12 {
        let mut map_duration = 0.;

        println!("Generating Voronoi graph {}...", seed + 1);
        let start = Instant::now();
        let mut map = Voronoi::new(seed, img_x, img_y);
        let duration = start.elapsed().as_secs_f64();
        println!("\tDone! ({:.2} seconds)", duration);
        map_duration += duration;

        println!("Defining water/land boundaries...");
        let start = Instant::now();
        let fbm = Fbm::new().set_seed(seed as u32);
        let mut minmax = (0.5, 0.5);
        for (idx, p) in map.seeds.iter().enumerate() {
            if (p.x - center_x).abs() > 190. || (p.y - center_y).abs() > 190. {
                // Cells whose seed is within 10 pixels of the border are water
                map.is_water[idx] = true;
            } else {
                // Calculate distance from the center, scaled to [0, 1] for the orthogonal directions
                // Diagonals can exceed 1, but that's okay
                let x = (f64::from(p.x) - center_x) / center_x;
                let y = (f64::from(p.y) - center_y) / center_y;
                // We square the distance to give greater weight to values further from the center
                let dist_sq = x.powi(2) + y.powi(2);

                // Get a noise value, and "pull" it toward 0.5; this raises low values while also
                // "blunting" high peaks
                let noise_val = fbm.get([x, y]).lerp(0.5, 0.5);
                minmax = (f64::min(minmax.0, noise_val), f64::max(minmax.1, noise_val));

                // Using noise and subtracting the distance gradient to define land or water
                map.is_water[idx] = noise_val - dist_sq * 0.5 < 0.0;
            }
        }
        let duration = start.elapsed().as_secs_f64();
        println!("\tDone! ({:.2} seconds)", duration);
        println!("\tNoise min/max: {:?}", minmax);
        map_duration += duration;

        let start = Instant::now();
        println!("\tDrawing...");
        draw_voronoi(&map, img_x, img_y, seed);
        let duration = start.elapsed();
        println!("\tDone! ({:.2} seconds)", duration.as_secs_f64());

        println!("Generated map in {:.2} seconds\n", map_duration);
    }
}
