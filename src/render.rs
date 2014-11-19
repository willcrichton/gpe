extern crate image;

use std::collections::HashSet;

use std::cmp::{min, max};
use std::iter::range_inclusive;

use image::GenericImage;
use encoding::{Encoding, Point, Color};

type BufColor = image::Rgb<u8>;
pub type Image = image::ImageBuf<BufColor>;

pub fn render(img: &Encoding) -> Image {
    let (w, h) = img.dimensions;
    let mut imgbuf = image::ImageBuf::new(w, h);

    let mut modified = HashSet::new();
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
                if pt.inside_polygon(polygon) {
                    let old_color = imgbuf.get_pixel(x, y);
                    imgbuf.put_pixel(x, y, blend(old_color, polygon.color));
                    modified.insert((x, y));
                }
            }
        }
    }

    for y in range(0, h) {
        for x in range(0, w) {
            if !modified.contains(&(x, y)) {
                imgbuf.put_pixel(x, y, image::Rgb(255, 255, 255));
            }
        }
    }

    imgbuf
}

fn blend(old_color: BufColor, new_color: Color) -> BufColor {
    let (or, og, ob) = old_color.channels();
    let (nr, ng, nb, a) = new_color.channels();
    let a = a as u32;
    image::Rgb(or + (nr as u32 * a / 255) as u8, og + (ng as u32 * a / 255) as u8, ob + (nb as u32 * a / 255) as u8)
}