use rand::prelude::*;
use rand_xoshiro::Xoshiro256StarStar;
use image;

mod voronoi;
use voronoi::Voronoi;

fn main() {
    let imgx = 400;
    let imgy = 400;

    let mut rng = Xoshiro256StarStar::seed_from_u64(0);

    let v = Voronoi::new(&mut rng, 256, imgx, imgy);

    let mut img = image::ImageBuffer::new(imgx as u32, imgy as u32);
    let colors = [
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
    ];

    for (points, &color) in v.cell_membership.iter().zip(colors.iter().cycle()) {

        for (x, y) in points.iter() {
            let pixel = img.get_pixel_mut(*x, *y);
            *pixel = color;
        }
    }

    img.save("map.png").unwrap();
}
