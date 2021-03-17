use rand::prelude::*;
use rand_xoshiro::Xoshiro256StarStar;
use image;
use imageproc::drawing::draw_hollow_circle;
use noise::{Fbm, Seedable, NoiseFn};

mod voronoi;
use voronoi::Voronoi;

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

fn main() {
    let imgx = 400;
    let imgy = 400;

    for seed in 0..12 {
        println!("Generating map {}...", seed);

        let mut rng = Xoshiro256StarStar::seed_from_u64(seed);
    
        println!("Generating Voronoi...");
        let mut v = Voronoi::new(&mut rng, 256, imgx, imgy);
        if seed == 0 {
            println!("\tDrawing...");
            draw_voronoi(&v, imgx, imgy, 1, false);
        }

        let relax_n = 2;
        println!("Performing {} iterations of Lloyd relaxation...", relax_n);
        for _ in 0..relax_n {
            v.improve_centers();
        }
        if seed == 0 {
            println!("\tDrawing...");
            draw_voronoi(&v, imgx, imgy, 2, false);
        }

        println!("Defining water/land boundaries...");
        let fbm = Fbm::new().set_seed(seed as u32);
        for (idx, (px, py)) in v.centers.iter().enumerate() {
            let x = (*px as i32 - imgx as i32 / 2) as f64 / (imgx / 2) as f64;
            let y = (*py as i32 - imgy as i32 / 2) as f64 / (imgy / 2) as f64;
            let d = x.powi(2) + y.powi(2);
            let n = fbm.get([x, y]);
            //print!("{} ", n);

            v.is_water[idx] = n + d > 0.5;
        }
        println!("\tDrawing...");
        draw_voronoi(&v, imgx, imgy, 3 + seed as usize, true);
        println!("");
    }
}
