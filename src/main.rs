use imageproc::drawing::{draw_line_segment_mut, draw_filled_rect_mut, draw_polygon_mut};
use imageproc::rect::Rect;
use lerp::Lerp;
//use std::cmp::Reverse;
use std::time::Instant;
//use rand::{prelude::*, seq::SliceRandom};
//use rand_xoshiro::Xoshiro256StarStar;

mod voronoi;
use voronoi::{Biome, Voronoi};
pub mod impluvium;
pub use impluvium::{river, lake};
mod map;
use map::Map;

const SEA_LEVEL: f64 = 0.0;

fn draw_map(map: &Map, i: u64) {
    let mut img = image::ImageBuffer::new(map.width(), map.height());

    let ocean = image::Rgb([70_u8, 107, 159]);

    draw_filled_rect_mut(&mut img, Rect::at(0, 0).of_size(map.width(), map.height()), ocean);

    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let height = map.get_height(x as f64, y as f64);

        let color = if height <= SEA_LEVEL {
            ocean
        } else {
            image::Rgb([
                108.0.lerp(255., height) as u8,
                152.0.lerp(255., height) as u8,
                95.0.lerp(255., height) as u8,
                ])
        };

        *pixel = color;
    }

    img.save(format!("noise_map_{:02}.png", i + 1)).unwrap();
}

fn _draw_voronoi(vor: &Voronoi, img_x: u32, img_y: u32, i: u64) {
    let mut img = image::ImageBuffer::new(img_x as u32, img_y as u32);
    let _sand = image::Rgb([160_u8, 144, 119]);
    let ocean = image::Rgb([70_u8, 107, 159]);
    let coastal_waters = image::Rgb([184_u8, 217, 235]);
    let lagoon = image::Rgb([184_u8, 245, 235]);
    let lake = image::Rgb([70_u8, 107, 159]);
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

    draw_filled_rect_mut(&mut img, Rect::at(0, 0).of_size(img_x, img_y), ocean);

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
            let fill = match vor.cells[p].biome {
                Biome::Coast => coastal_waters,
                Biome::Lake => lake,
                Biome::Ocean => ocean,
                Biome::Lagoon => lagoon,
                Biome::Beach => {
                    //sand
                    image::Rgb([
                        108.0.lerp(255., vor.cells[p].height) as u8,
                        152.0.lerp(255., vor.cells[p].height) as u8,
                        95.0.lerp(255., vor.cells[p].height) as u8,
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

    println!("\t\tDrawing rivers...");
    for river in vor.rivers.iter() {
        for (p1, p2) in river.segments() {
            let delaunator::Point{ x, y } = vor.cells[p1].as_point();
            let prev = (x as f32, y as f32);
            let delaunator::Point{ x, y } = vor.cells[p2].as_point();
            let current = (x as f32, y as f32);
            draw_line_segment_mut(&mut img, prev, current, image::Rgb([0_u8, 0, 0]));
        }

        //let delaunator::Point{ x, y } = vor.points[river.mouth()];
        //let radius = 2 * river.order().0 as i32;
        //draw_filled_circle_mut(&mut img, (x.round() as i32, y.round() as i32), radius, image::Rgb([0_u8, 0, 0]));
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

    println!("\tSaving...");
    img.save(format!("map_{:02}.png", i + 1)).unwrap();
}

/* fn _get_height(point: &delaunator::Point, dimensions: [f64; 2], angle1: f64, angle2: f64) -> f64 {
    let scale = dimensions[0].max(dimensions[1]) / 3.0;

    let center_x = dimensions[0] / 2.0;
    let center_y = dimensions[1] / 2.0;

    // For our first point we just use the center
    let x = (f64::from(point.x) - center_x) / scale;
    let y = (f64::from(point.y) - center_y) / scale;
    // Use the distance function, but squared to give greater dropoff
    let grad1 = 1.0 - x.powi(2) - y.powi(2);

    // Add a second point
    let grad2 = {
        let dist = 180.0;
        let x = center_x + dist * angle1.cos();
        let y = center_y + dist * angle1.sin();

        let dx = (f64::from(point.x) - x) / scale;
        let dy = (f64::from(point.y) - y) / scale;

        1.0 - dx.powi(2) - dy.powi(2)
    };

    // Add a third point, sometimes equal-but-opposite, sometimes on angle2
    let grad2_2 = {
        let dist = 180.0;
        let angle1 = match (angle1 * 100.0).round() as usize % 3 {
            0 => angle2,
            1 => angle1 + std::f64::consts::PI,
            2 => angle1.min(angle2) + (angle1 - angle2).abs() / 2.0,
            _ => unreachable!(),
        };
        let x = center_x + dist * angle1.cos();
        let y = center_y + dist * angle1.sin();

        let dx = (f64::from(point.x) - x) / scale;
        let dy = (f64::from(point.y) - y) / scale;

        1.0 - dx.powi(2) - dy.powi(2)
    };

    // And another point
    let grad3 = {
        let dist = 300.0;
        let x = center_x + dist * angle2.cos();
        let y = center_y + dist * angle2.sin();

        let dx = (f64::from(point.x) - x) / scale;
        let dy = (f64::from(point.y) - y) / scale;

        dx.powi(2) + dy.powi(2)
    };

    // Now merge them into a single gradient
    let mut gradient = grad1.max(grad2 * 0.85).max(grad2_2 * 0.80).min(grad3.powi(3) + 0.45);
    gradient = gradient.clamp(0.0, 1.0);

    // Get a noise value, and "pull" it up
    let mut height = noise.get_noise(x as f32, y as f32) as f64;
    height = height.lerp(0.5, 0.5);
    // Lerp it towards a point below sea level, using our gradient as the t-value
    height = height.lerp(-0.2, 1.0 - gradient);

    height
} */

fn main() {
    let img_x = 800;
    let img_y = 800;

    for seed in 0..12 {
        let mut _map_duration = 0.;

        //let mut rng = Xoshiro256StarStar::seed_from_u64(seed);

        /* println!("Generating Voronoi graph {}...", seed + 1);
        let start = Instant::now();
        let mut map = Voronoi::new(seed, img_x, img_y);
        let duration = start.elapsed().as_secs_f64();
        println!("\tDone! ({:.2} seconds; {} polygons)", duration, map.cells.len());
        map_duration += duration; */

        println!("Generating coastline...");
        let _start = Instant::now();

        let map = Map::new(seed, img_x, img_y);

        draw_map(&map, seed);
        continue;

        /*
        let duration = start.elapsed().as_secs_f64();
        println!("\tDone! ({:.2} seconds)", duration);
        map_duration += duration;

        let start = Instant::now();
        println!("Defining waterlines...");

        // Initial pass just to define land/sea border; we use Lake instead of Ocean for now
        for cell in map.cells.iter_mut() {
            let voronoi::Cell{x,y,..} = cell;
            cell.biome = if cell.height < SEA_LEVEL {
                Biome::Lake
            } else {
                if *x < 10. || *x > f64::from(img_x) - 10. || *y < 10. || *y > f64::from(img_y) - 10. {
                    Biome::Lake
                } else {
                    Biome::Beach
                }
            };
        }

        // Use a flood-fill to turn open ocean into, well, Ocean
        // By starting from a point on the hull we guarantee we start from open ocean
        let first = map.delaunay.hull[0];
        let mut active = vec![first];
        while !active.is_empty() {
            let p = active.pop().unwrap();

            map.cells[p].biome = {
                // Check adjacent cells if they're land
                if map.neighbors_of_point(p).into_iter().any(|p| map.cells[p].height > SEA_LEVEL ) {
                    Biome::Coast
                } else {
                    Biome::Ocean
                }
            };

            // Append all neighboring "Lake" cells
            active.extend(map.neighbors_of_point(p).into_iter().filter(|&p| map.cells[p].biome == Biome::Lake ));
        }

        // Now flood-fill again, looking for "real" open Ocean and Ocean-adjacent coasts
        let mut real_ocean = vec![false; map.cells.len()];
        let mut active = vec![first];
        while !active.is_empty() {
            let p = active.pop().unwrap();

            real_ocean[p] = true;

            // Append all neighboring Ocean or Coast cells, IF the current cell is Ocean
            if map.cells[p].biome == Biome::Ocean {
                active.extend(
                    map
                        .neighbors_of_point(p)
                        .into_iter()
                        .filter(|&p| (map.cells[p].biome == Biome::Coast || map.cells[p].biome == Biome::Ocean) && !real_ocean[p] )
                );
            }
        }
        // Now we'll flood-fill Ocean/Coast that isn't "real", but only from disconnected Ocean
        active = real_ocean.iter().enumerate().filter_map(|(i, is_real)| {
            if !is_real && map.cells[i].biome == Biome::Ocean {
                Some(i)
            } else {
                None
            }
        }).collect();
        while !active.is_empty() {
            let p = active.pop().unwrap();

            map.cells[p].biome = Biome::Lagoon;

            // Append all neighboring Coast cells that aren't "real"
            active.extend(
                map
                    .neighbors_of_point(p)
                    .into_iter()
                    .filter(|&p| map.cells[p].biome == Biome::Coast && !real_ocean[p] )
            );
        }

        let duration = start.elapsed().as_secs_f64();
        println!("\tDone! ({:.2} seconds)", duration);
        map_duration += duration;

        // Define a new heightmap based on distance from Coast
        // Lagoons are considered "coast", and Lakes do not add to elevation
        let start = Instant::now();
        println!("Generating heightmap...");

        let mut new_heights = vec![f64::MAX; map.cells.len()];
        let mut changed = true;
        while changed {
            changed = false;
            for i in 0..map.cells.len() {
                let my_height = new_heights[i];

                if map.cells[i].biome.is_water() {
                    new_heights[i] = 0.0;
                } else {
                    let start = my_height;
                    let min = map.neighbors_of_point(i)
                        .iter()
                        .filter(|&&p| new_heights[p] > -0.1)
                        .fold(start, |min, &p| min.min(new_heights[p]) );

                    let dist = if map.cells[i].biome == Biome::Lake {
                        min
                    } else {
                        min + 1.0
                    };

                    if dist >= start {
                        continue;
                    }

                    new_heights[i] = new_heights[i].min(dist);
                }

                if (my_height - new_heights[i]).abs() > f64::EPSILON {
                    changed = true;
                }
            }
        }
        let max_height = new_heights.iter()
            .fold(new_heights[0], |max, &h| max.max(h));
        for (i, height) in new_heights.iter().enumerate() {
            if *height > 0.0 {
                let height = (*height / max_height).powi(2);
                map.cells[i].height = map.cells[i].height.lerp(1.0, height);
            }
        }

        let duration = start.elapsed().as_secs_f64();
        println!("\tDone! ({:.2} seconds)", duration);
        map_duration += duration;

        // Rivers
        let start = Instant::now();
        println!("Creating rivers...");

        // Start by erasing Lakes; we'll add some later
        for cell in map.cells.iter_mut() {
            if cell.biome == Biome::Lake {
                cell.biome = Biome::Beach;
            }
        }

        // Finding the high polygons; rivers will start here
        let mut sources: Vec<_> = map.cells
            .iter()
            .enumerate()
            .filter_map(|(idx, cell)| {
                if cell.height >= 0.3 {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect();

        // From a random sampling of high ground, create rivers from each by flowing downhill
        let amount = sources.len(); //usize::min(sources.len() / 5, 17);
        let (starts, _) = sources.partial_shuffle(&mut rng, amount);

        let mut rivers = Vec::new();
        for start in starts.into_iter() {
            let mut river = Vec::new();

            river.push(*start);
            let mut point = *start;

            loop {
                // Find the lowest neighbor
                if let Some(p) = map.neighbors_of_point(point)
                    .into_iter()
                    .filter(|p| !river.contains(p))
                    .min_by(|&p1, &p2| {
                        if map.cells[p1].height < map.cells[p2].height {
                            std::cmp::Ordering::Less
                        } else {
                            std::cmp::Ordering::Greater
                        }
                    })
                {
                    // Make sure it's lower than us
                    if map.cells[p].height > map.cells[point].height {
                        // We'll handle these later
                        break;
                    } else {
                        point = p;
                    }

                    river.push(point);

                    // Check if we've reached water
                    if map.cells[point].biome.is_water() {
                        break;
                    }
                } else {
                    break;
                }
            }

            rivers.push(river);
        }

        // Sort rivers so that we start with the longest
        rivers.sort_unstable_by_key(|river| Reverse(river.len()));
        for mut river in rivers {
            // Rivers are in source-to-mouth order, reverse for mouth-to-source
            river.reverse();
            if let Some(parent_river) = map.rivers.iter_mut().find(|r| r.mouth() == river[0]) {
                // This river's part of an existing network, add it as a branch
                parent_river.add_branch(river);
            } else {
                // New river network
                map.rivers.push(river.into());
            }
        }
        map.rivers = map.rivers.into_iter().filter_map(|mut river| {
            if river.prune() {
                Some(river)
            } else {
                None
            }
        }).collect();
        println!("\tCreated {} river networks", map.rivers.len());

        let mut lakes = Vec::new();
        // Create lakes wherever rivers end within the island
        for mouth in map.rivers.iter().filter_map(|river| {
            if map.cells[river.mouth()].biome.is_water() {
                None
            } else {
                Some(river.mouth())
            }
        }) {
            lakes.push(lake::Lake::new_at(mouth, &map));
        }
        for lake in lakes {
            lake.apply(&mut map);
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
        */
    }
}
