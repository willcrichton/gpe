use std::iter::range_inclusive;

use encoding::{Encoding, Point, Color, fmin};

type BufColor = (u8, u8, u8);
pub type Image = Vec<BufColor>;

pub fn render(img: &Encoding, antialias: bool) -> Image {
    let (w, h) = img.dimensions;
    let mut imgbuf = Vec::from_fn((w * h) as uint, |_| (255, 255, 255));

    //let mut updated = Vec::with_capacity((w * h) as uint);
    for polygon in img.polygons.iter() {
        let (min, max) = polygon.bounding_box;
        //updated.clear();

        for y in range_inclusive(min.y as u32 - 4, max.y as u32 + 4) {
            for x in range_inclusive(min.x as u32 - 4, max.x as u32 + 4) {
                if y >= h || x >= w { continue; }

                let pt = Point {x: x as f32, y: y as f32};
                let (contains, dist) = polygon.query(&pt, antialias);

                if contains || (antialias && dist < 3.0 /*+ fmin(1.0 / polygon.blur, 5.0)*/) {
                    let mut new_color = polygon.color;

                    if !contains {
                        let (r, g, b, a) = new_color;
                        let scale = (1.0 + dist) * (1.0 + dist);

                        new_color = (r, g, b, a / (scale as u8));
                    }

                    let old_color = imgbuf[(y * w + x) as uint];
                    imgbuf[(y * w + x) as uint] = blend(old_color, new_color);
                    //updated.push((x, y));
                }
            }
        }

        /*for &(x, y) in updated.iter() {
            let surrounding =
                if polygon.blur > 0.5 {
                    vec![(x - 1, y), (x, y - 1), (x, y + 1), (x + 1, y)]
                } else {
                    let mut v = vec![];
                    let n = fmin(1.0 / polygon.blur, 6.0) as u32;
                    for i in range_inclusive(0, n) {
                        for j in range_inclusive(0, n) {
                            if i == 0 && j == 0 { continue; }
                            v.push((x + i - n / 2, y + j - n / 2));
                        }
                    }
                    v
                };

            let (br, bg, bb) = imgbuf[(y * w + x) as uint];
            let (mut r, mut g, mut b) = (0.0, 0.0, 0.0);
            let mut num_valid = 0.0;

            for (px, py) in surrounding.into_iter() {
                if px >= w || py >= h { continue; }
                let (pr, pg, pb) = imgbuf[(py * w + px) as uint];
                r += pr as f32;
                g += pg as f32;
                b += pb as f32;
                num_valid += 1.0;
            }

            let add_scale = (1.0 - polygon.blur) / num_valid;
            imgbuf[(y * w + x) as uint] =
                ((r * add_scale + (br as f32) * polygon.blur) as u8,
                 (g * add_scale + (bg as f32) * polygon.blur) as u8,
                 (b * add_scale + (bb as f32) * polygon.blur) as u8);
        }*/
    }

    for pixel in img.pixels.iter() {
        imgbuf[(pixel.pos.y * (w as f32) + pixel.pos.x) as uint] = pixel.color;
    }

    imgbuf
}

#[inline(always)]
fn add(old: u8, new: u8, alpha: u8) -> u8 {
    let addend = (new as u32) * (alpha as u32) / 255;
    (addend + (old as u32) * ((255 - alpha) as u32) / 255) as u8
    //if addend + (old as u32) > 255 { 255 } else { (addend as u8) + old }
}

#[inline(always)]
fn blend(old_color: BufColor, new_color: Color) -> BufColor {
    let (or, og, ob) = old_color;
    let (nr, ng, nb, a) = new_color;
    (add(or, nr, a), add(og, ng, a), add(ob, nb, a))
}