use rand::prelude::*;
use rand::distributions::{Distribution, Uniform};
use rand_xoshiro::Xoshiro256StarStar;
use image;
use imageproc::drawing::draw_filled_circle;

fn main() {
    let mut rng = Xoshiro256StarStar::seed_from_u64(0);
    let distribution = Uniform::from(0..400);

    let mut centers: Vec<(u32, u32)> = Vec::new();
    while centers.len() < 256 {
        let point = (distribution.sample(&mut rng), distribution.sample(&mut rng));
        if !centers.contains(&point) {
            centers.push(point);
        }
    }

    let imgx = 400;
    let imgy = 400;

    let mut membership = vec![Vec::<(u32, u32)>::new(); centers.len()];
    for x in 0..imgx {
        for y in 0..imgy {
            let (idx, _) = centers.iter().enumerate().min_by_key(|point| (point.1.0 as i32 - x).pow(2) + (point.1.1 as i32 - y).pow(2)).unwrap();

            membership[idx].push((x as u32, y as u32));
        }
    }

    let mut img = image::ImageBuffer::new(imgx as u32, imgy as u32);
    let mut colors = [
        image::Rgb([0u8,0u8,0u8]),
        image::Rgb([34u8,32u8,52u8]),
        image::Rgb([69u8,40u8,60u8]),
        image::Rgb([102u8,57u8,49u8]),
        image::Rgb([143u8,86u8,59u8]),
        image::Rgb([223u8,113u8,38u8]),
        image::Rgb([217u8,160u8,102u8]),
        image::Rgb([238u8,195u8,154u8]),
        image::Rgb([251u8,242u8,54u8]),
        image::Rgb([153u8,229u8,80u8]),
        image::Rgb([106u8,190u8,48u8]),
        image::Rgb([55u8,148u8,110u8]),
        image::Rgb([75u8,105u8,47u8]),
        image::Rgb([82u8,75u8,36u8]),
        image::Rgb([50u8,60u8,57u8]),
        image::Rgb([63u8,63u8,116u8]),
        image::Rgb([48u8,96u8,130u8]),
        image::Rgb([91u8,110u8,225u8]),
        image::Rgb([99u8,155u8,255u8]),
        image::Rgb([95u8,205u8,228u8]),
        image::Rgb([203u8,219u8,252u8]),
        image::Rgb([255u8,255u8,255u8]),
        image::Rgb([155u8,173u8,183u8]),
        image::Rgb([132u8,126u8,135u8]),
        image::Rgb([105u8,106u8,106u8]),
        image::Rgb([89u8,86u8,82u8]),
        image::Rgb([118u8,66u8,138u8]),
        image::Rgb([172u8,50u8,50u8]),
        image::Rgb([217u8,87u8,99u8]),
        image::Rgb([215u8,123u8,186u8]),
        image::Rgb([143u8,151u8,74u8]),
        image::Rgb([138u8,111u8,48u8]),
    ].iter().cycle();

    for points in membership.iter() {
        let color = colors.next().unwrap();

        for (x, y) in points.iter() {
            let pixel = img.get_pixel_mut(*x, *y);
            *pixel = *color;
        }
    }

    img.save("map.png").unwrap();
}
