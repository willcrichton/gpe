use std::cmp::{min, max};
use std::iter::range_inclusive;

use encoding::{Encoding, Point, Color};

type BufColor = (u8, u8, u8);
pub type Image = Vec<BufColor>;

pub fn render(img: &Encoding, antialias: bool) -> Image {
    let (w, h) = img.dimensions;
    let mut imgbuf = Vec::from_fn((w * h) as uint, |_| (0, 0, 0));

    for polygon in img.polygons.iter() {
        let (mut minx, mut miny, mut maxx, mut maxy) =
            (w - 1, h - 1, 0, 0);

        for vertex in polygon.vertices.iter() {
            minx = min(minx, vertex.x as u32);
            miny = min(miny, vertex.y as u32);
            maxx = max(maxx, vertex.x as u32);
            maxy = max(maxy, vertex.y as u32);
        }

        for y in range_inclusive(miny, maxy) {
            for x in range_inclusive(minx, maxx) {
                let pt = Point {x: x as f32, y: y as f32};
                let (contains, dist) = polygon.query(&pt, antialias);
                if contains || (antialias && dist < 4.0) {
                    let mut new_color = polygon.get_color(pt);
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

    imgbuf
}

#[inline(always)]
fn add(old: u8, new: u8, alpha: u8) -> u8 {
    let addend = (new as u32) * (alpha as u32) / 255;
    if addend + (old as u32) > 255 { 255 } else { (addend as u8) + old }
}

#[inline(always)]
fn blend(old_color: BufColor, new_color: Color) -> BufColor {
    let (or, og, ob) = old_color;
    let (nr, ng, nb, a) = new_color;
    (add(or, nr, a), add(og, ng, a), add(ob, nb, a))
}