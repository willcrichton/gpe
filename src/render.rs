use std::iter::range_inclusive;

use encoding::{Encoding, Point, Color, CEncoding, RGB};

type BufColor = (u8, u8, u8);
pub type Image = Vec<BufColor>;

extern {
    fn cuda_render(img: CEncoding, output: *mut RGB, antialias: bool);
}

pub fn render(img: &Encoding, antialias: bool) -> Image {
    let (w, h) = img.dimensions;
    let mut imgbuf = Vec::from_fn((w * h) as uint, |_| RGB {r: 0, g: 0, b: 0});

    unsafe {
        cuda_render(img.clone().raw(), imgbuf.as_mut_ptr(), antialias);
    }

    return imgbuf.into_iter().map(|color| (color.r, color.g, color.b)).collect();

    /*for polygon in img.polygons.iter() {
        let (min, max) = polygon.bounding_box;
        for y in range_inclusive(min.y as u32, max.y as u32) {
            for x in range_inclusive(min.x as u32, max.x as u32) {
                let pt = Point {x: x as f32, y: y as f32};
                let (contains, dist) = polygon.query(&pt, antialias);

                if contains || (antialias && dist < 4.0) {
                    let mut new_color = polygon.color(pt);
                    if !contains {
                        let (r, g, b, a) = new_color;
                        let scale = (1.0 + dist) * (1.0 + dist);

                        // TODO: antialiasing introduces artifacts
                        new_color = (r, g, b, a / (scale as u8));
                    }

                    let old_color = imgbuf[(y * w + x) as uint];
                    imgbuf[(y * w + x) as uint] = blend(old_color, new_color);
                }
            }
        }
    }

    imgbuf*/
}