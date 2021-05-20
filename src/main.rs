use imageproc::drawing::{draw_filled_rect_mut, draw_line_segment_mut};
use imageproc::rect::Rect;
use lerp::Lerp;

mod map;
use map::{Map, SEA_LEVEL};

fn draw_map(map: &Map, i: u64) {
    let mut img = image::ImageBuffer::new(map.width(), map.height());

    let ocean = image::Rgb([70_u8, 107, 159]);

    draw_filled_rect_mut(
        &mut img,
        Rect::at(0, 0).of_size(map.width(), map.height()),
        ocean,
    );

    // Draw terrain
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let height = map.get_elevation(x, y);

        let color = if height <= SEA_LEVEL {
            continue;
        } else {
            let bands = 8.0;
            let height = (height * bands).floor() / bands;
            image::Rgb([
                108.0.lerp(255., height) as u8,
                152.0.lerp(255., height) as u8,
                95.0.lerp(255., height) as u8,
            ])
        };

        *pixel = color;
    }

    // // Draw rivers
    // let river = image::Rgb([70_u8, 107, 159]);
    // for ((x1, y1), (x2, y2)) in map.get_river_segments() {
    //     draw_line_segment_mut(
    //         &mut img,
    //         (x1 as f32, y1 as f32),
    //         (x2 as f32, y2 as f32),
    //         river,
    //     );
    // }

    let sand = image::Rgb([160_u8, 144, 119]);
    for (x, y) in map.get_coast() {
        img.put_pixel(*x, *y, sand);
    }

    img.save(format!("noise_map_{:02}.png", i + 1)).unwrap();
}

fn main() {
    let img_x = 800;
    let img_y = 800;

    for seed in 0..12 {
        println!("Generating island {}...", seed + 1);

        let map = Map::new(seed, img_x, img_y);

        draw_map(&map, seed);
    }
}
