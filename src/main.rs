use rand::prelude::*;
use rand_xoshiro::Xoshiro256StarStar;
use image;
use imageproc::drawing::draw_hollow_circle;

mod voronoi;
use voronoi::Voronoi;

fn draw_voronoi(vor: &Voronoi, imgx: u32, imgy: u32, i: usize) {
    let mut img = image::ImageBuffer::new(imgx as u32, imgy as u32);
    let interior = image::Rgb([255u8, 255, 255]);
    let frontier = image::Rgb([0u8, 0, 0]);

    for points in vor.cell_membership.iter() {

        for p in points.iter() {
            let pixel = img.get_pixel_mut(p.0, p.1);
            *pixel = interior;

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

    let mut rng = Xoshiro256StarStar::seed_from_u64(0);

    let mut v = Voronoi::new(&mut rng, 256, imgx, imgy);
    draw_voronoi(&v, imgx, imgy, 1);

    for i in 0..2 {
        v.improve_centers();
        draw_voronoi(&v, imgx, imgy, i+2);
    }
}
