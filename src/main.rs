use bracket_noise::prelude::*;
use fast_poisson::Poisson2D;
use imageproc::drawing::{draw_filled_rect_mut, draw_polygon_mut};
use imageproc::rect::Rect;
use lerp::Lerp;
use std::time::Instant;

mod voronoi;
use voronoi::{Biome, Voronoi};

const SEA_LEVEL: f64 = 0.0;

fn draw_voronoi(vor: &Voronoi, img_x: u32, img_y: u32, i: u64) {
    let mut img = image::ImageBuffer::new(img_x as u32, img_y as u32);
    let _sand = image::Rgb([160_u8, 144, 119]);
    let water = image::Rgb([70_u8, 107, 159]);
    let coastal_waters = image::Rgb([184_u8, 217, 235]);
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

    draw_filled_rect_mut(&mut img, Rect::at(0, 0).of_size(img_x, img_y), water);

    let mut preprocessing = 0.;
    let mut drawing = 0.;

    println!("\tDrawing Voronoi polygons...");
    let mut seen = vec![false; vor.delaunay.triangles.len()];
    for e in 0..vor.delaunay.triangles.len() {
        let p = vor.delaunay.triangles[vor.next_halfedge(e)];

        if !seen[p] {
            let start = Instant::now();
            seen[p] = true;
            let edges = vor.edges_around_point(e);
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
            let fill = match vor.biomes[p] {
                Biome::Coast => coastal_waters,
                Biome::Lake => water,
                Biome::Ocean => water,
                Biome::Beach => {
                    //sand
                    image::Rgb([
                        108.0.lerp(255., vor.heightmap[p]) as u8,
                        152.0.lerp(255., vor.heightmap[p]) as u8,
                        95.0.lerp(255., vor.heightmap[p]) as u8,
                        ])
                },
                //_ => image::Rgb([0u8, 0, 0]),
            };

            draw_polygon_mut(&mut img, &vertices, fill);
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
        "\t\tPreprocessing: {} seconds\n\t\tDrawing: {} seconds",
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

    println!("\tSaving...");
    img.save(format!("map_{:02}.png", i + 1)).unwrap();
}

fn main() {
    let img_x = 800;
    let img_y = 800;

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
        println!("\tDone! ({:.2} seconds; {} polygons)", duration, map.points.len());
        map_duration += duration;

        println!("Generating heightmap...");
        let start = Instant::now();

        // I have no idea what these parameters do!
        // They're stolen directly from https://github.com/amethyst/bracket-lib/blob/master/bracket-noise/examples/simplex_fractal.rs
        // They do seem to give me results I like, though!
        let mut fbm = FastNoise::seeded(seed as u64);
        fbm.set_noise_type(NoiseType::SimplexFractal);
        fbm.set_fractal_type(FractalType::FBM);
        fbm.set_fractal_octaves(5);
        fbm.set_fractal_gain(0.6);
        fbm.set_fractal_lacunarity(2.0);
        fbm.set_frequency(2.0);

        let mut minmax = (0.2, 0.2);

        fn get_height(point: &delaunator::Point, center_x: f64, center_y: f64, noise: &FastNoise) -> f64 {
            // Calculate distance from the center, scaled to [0, 1] for the orthogonal directions
            // Diagonals can exceed 1, but that's okay
            let x = (f64::from(point.x) - center_x) / center_x;
            let y = (f64::from(point.y) - center_y) / center_y;
            // We square the distance to give greater weight to values further from the center
            let dist_sq = x.powi(2) + y.powi(2);

            // Get a noise value, and "pull" it toward 0.5; this raises low values while also
            // "blunting" high peaks
            let mut height = noise.get_noise(x as f32, y as f32) as f64;
            height = height.lerp(0.5, 0.5);
            height = height.lerp(-0.2, dist_sq);

            height
        }

        let max_x = f64::from(img_x);
        let max_y = f64::from(img_y);

        let mut seen = vec![false; map.delaunay.triangles.len()];
        for e in 0..map.delaunay.triangles.len() {
            let point_idx = map.delaunay.triangles[map.next_halfedge(e)];

            if !seen[point_idx] {
                seen[point_idx] = true;
                let edges = map.edges_around_point(e);
                let triangles: Vec<usize> = edges.iter().map(|&e| map.triangle_of_edge(e)).collect();
                let mut vertices: Vec<delaunator::Point> = triangles
                    .iter()
                    .map(|&t| map.triangle_center(t) )
                    .collect();
                if vertices.iter().any(|delaunator::Point {x, y}| {
                    *x < 0.0 || *x > max_x || *y < 0.0 || *y > max_y
                })
                {
                    // Some of these get ridiculously far out of bounds, not sure how
                    // Fortunately this only happens around the edges, so we can safely skip 'em
                    continue;
                }
                vertices.dedup();
                if vertices.first() == vertices.last() {
                    vertices.pop();
                }
                
                // Set the height of a cell to be the average of the heights of its corners
                let count = vertices.len() as f64;
                if count == 0.0 {
                    // No vertices? Not sure how, but skip it regardless
                    continue;
                }
                let sum: f64 = vertices
                    .iter()
                    .map(|p| get_height(&p, center_x, center_y, &fbm))
                    .sum();

                let height = sum / count;

                // Using noise and subtracting the distance gradient to define land or water
                //map.heightmap[idx] = noise_val - dist_sq * 0.75;
                map.heightmap[point_idx] = height;

                minmax = (f64::min(minmax.0, height), f64::max(minmax.1, height));
            }
        }

        let mut minmax2 = minmax;
        // Redistribute heights to "stretch" peaks to 1.0 -- we want mountains!
        for height in map.heightmap.iter_mut() {
            if *height > 0.0 {
                *height = height.lerp(1.0, *height / minmax.1).powi(2);
            }
            minmax2 = (f64::min(minmax2.0, *height), f64::max(minmax2.1, *height));
        }

        let duration = start.elapsed().as_secs_f64();
        println!("\tDone! ({:.2} seconds)", duration);
        println!("\tNoise min/max: {:?}", minmax);
        println!("\tRedistributed min/max: {:?}", minmax2);
        map_duration += duration;

        let start = Instant::now();
        println!("Assigning biomes...");

        // Build an index of points to an incoming half-edge; useful to find the point's cell
        // and neighbors later
        let point_halfedge = {
            let mut index = vec![usize::MAX; map.points.len()];
            for e in 0..map.delaunay.triangles.len() {
                let edge = map.delaunay.triangles[map.next_halfedge(e)];
                if index[edge] == usize::MAX {
                    index[edge] = e;
                }
            }
            index
        };

        // Initial pass just to define land/sea border; we use Lake instead of Ocean for now
        for i in 0..map.points.len() {
            let delaunator::Point{x,y} = map.points[i];
            map.biomes[i] = if map.heightmap[i] < SEA_LEVEL {
                Biome::Lake
            } else {
                if x < 10. || x > f64::from(img_x) - 10. || y < 10. || y > f64::from(img_y) - 10. {
                    Biome::Lake
                } else {
                    Biome::Beach
                }
            };
        }

        // Use a flood-fill to turn open ocean into, well, Ocean
        let (first, _) = map.points.iter().enumerate().min_by(|(_, p1), (_, p2)| {
            if (p1.x, p1.y) < (p2.x, p2.y) {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        }).unwrap();
        let mut active = vec![first];
        while !active.is_empty() {
            let p = active.pop().unwrap();
            let edges = map.edges_around_point(point_halfedge[p]);

            map.biomes[p] = {
                // Check adjacent cells if they're land
                if edges.iter().any(|&e| map.heightmap[map.delaunay.triangles[e]] > SEA_LEVEL ) {
                    Biome::Coast
                } else {
                    Biome::Ocean
                }
            };

            // Append all neighboring "Lake" cells
            active.extend(edges.iter().filter_map(|&e| {
                let p = map.delaunay.triangles[e];

                if map.biomes[p] == Biome::Lake {
                    Some(p)
                } else {
                    None
                }
            }));
        }

        let duration = start.elapsed().as_secs_f64();
        println!("\tDone! ({:.2} seconds)", duration);
        map_duration += duration;

        let start = Instant::now();
        println!("Drawing...");
        draw_voronoi(&map, img_x, img_y, seed);
        let duration = start.elapsed();
        println!("\tDone! ({:.2} seconds)", duration.as_secs_f64());

        println!("Generated map in {:.2} seconds\n", map_duration);
    }
}
