use std::fmt;
use std::num::Float;
use std::rand::random;
use std::cmp::{min, max};
use std::iter::range_inclusive;

use constants::*;
use compress::Compressor;

#[deriving(Clone)]
#[repr(C)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

pub type Color = (u8, u8, u8, u8);

#[deriving(Clone)]
#[repr(C)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[deriving(Clone)]
#[repr(C)]
pub struct Pixel {
    pub pos: Point,
    pub color: RGB,
}

#[deriving(Clone)]
pub struct Polygon {
    pub vertices: Vec<Point>,
    pub color: Color,
    edges: Vec<(Point, Point)>,
    center: Point,
    max_dist: f32,
    pub bounding_box: (Point, Point),
}

#[repr(C)]
pub struct CPolygon {
    vertices: *mut Point,
    num_vertices: u32,
    r: u8, g: u8, b: u8, a: u8,
    center: Point,
    max_dist: f32,
}

#[deriving(Clone)]
pub struct Encoding {
    pub polygons: Vec<Polygon>,
    pub dimensions: (u32, u32),
    pub pixels: Vec<Pixel>,
}

#[repr(C)]
pub struct CEncoding {
    polygons: *mut CPolygon,
    num_polygons: u32,
    pixels: *mut Pixel,
    num_pixels: u32,
    width: u32,
    height: u32,
}

#[inline(always)]
fn fmin(a: f32, b: f32) -> f32 {
    if a < b { a } else { b }
}

#[inline(always)]
fn fmax(a: f32, b: f32) -> f32 {
    if a < b { b } else { a }
}

#[inline(always)]
fn clamp(p: &mut Point, (w, h): (u32, u32)) {
    p.x = fmax(fmin(p.x, (w - 1) as f32), 0.0);
    p.y = fmax(fmin(p.y, (h - 1) as f32), 0.0);
}

impl Point {
    #[inline(always)]
    pub fn distance_squared(&self, other: &Point) -> f32 {
        let (x, y) = (self.x - other.x, self.y - other.y);
        x * x + y * y
    }

    #[inline(always)]
    pub fn dot(&self, other: &Point) -> f32 {
        self.x * other.x + self.y * other.y
    }

    #[inline]
    pub fn mutate(&mut self, (w, h): (u32, u32)) -> bool {
        let mutated = should_mutate(MOVE_VERTEX_RATE);
        if mutated {
            self.x += (random::<f32>() - 0.5) * MOVE_VERTEX_MAX;
            self.y += (random::<f32>() - 0.5) * MOVE_VERTEX_MAX;
            clamp(self, (w, h));
        }

        mutated
    }
}

impl Add<Point, Point> for Point {
    #[inline(always)]
    fn add(&self, other: &Point) -> Point {
        Point {x: self.x + other.x, y: self.y + other.y}
    }
}

impl Sub<Point, Point> for Point {
    #[inline(always)]
    fn sub(&self, other: &Point) -> Point {
        Point {x: self.x - other.x, y: self.y - other.y}
    }
}

impl Mul<f32, Point> for Point {
    #[inline(always)]
    fn mul(&self, constant: &f32) -> Point {
        Point {x: self.x * *constant, y: self.y * *constant}
    }
}

impl fmt::Show for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl fmt::Show for Polygon {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{Color: {}, Vertices: {}}}\n", self.color, self.vertices)
    }
}

impl Polygon {
    pub fn new(vertices: Vec<Point>, color: Color) -> Polygon {
        let mut polygon = Polygon {
            vertices: vertices,
            color: color,
            edges: Vec::new(),
            center: Point {x: 0.0, y: 0.0},
            max_dist: 0.0,
            bounding_box: (Point {x: 0.0, y: 0.0}, Point {x: 0.0, y: 0.0}),
        };

        polygon.update_data();
        polygon
    }

    pub fn random(compressor: &Compressor) -> Option<Polygon> {
        let (w, h) = compressor.dimensions;
        let origin = Point {x: random::<f32>() * (w as f32),
                            y: random::<f32>() * (h as f32)};

        let mut vertices = vec![origin];
        for _ in range(0, VERTICES - 1) {
            let mut vtx = origin + Point{x: (random::<f32>() - 0.5) * (w as f32),
                                     y: (random::<f32>() - 0.5) * (h as f32)};
            clamp(&mut vtx, (w, h));
            vertices.push(vtx);
        }

        let mut polygon = Polygon::new(order_points(vertices), (0, 0, 0, 0));
        let (mut r, mut g, mut b) = (0, 0, 0);
        let mut count = 0u;
        let (bbmin, bbmax) = polygon.bounding_box;

        // weight color towards expected color in base
        for y in range_inclusive(bbmin.y as u32, bbmax.y as u32) {
            for x in range_inclusive(bbmin.x as u32, bbmax.x as u32) {
                let pt = Point {x: x as f32, y: y as f32};
                let (contains, _) = polygon.query(&pt, false);
                let (br, bg, bb) = compressor.base[((y * w) + x) as uint];
                if contains {
                    count += 1;
                    r += br as uint;
                    g += bg as uint;
                    b += bb as uint;
                }
            }
        }

        // polygon is invalid
        if count == 0 {
            return None;
        }

        polygon.color = (polygon.rand_color((r / count) as u8),
                         polygon.rand_color((g / count) as u8),
                         polygon.rand_color((b / count) as u8),
                         random::<u8>() % 60 + 30);

        Some(polygon)
    }

    fn update_data(&mut self) {
        let mut edges = Vec::new();
        let len = self.vertices.len();
        for i in range(0, len) {
            edges.push((self.vertices[i], self.vertices[(i + 1) % len]));
        }

        let mut center = self.vertices[0];
        for vertex in self.vertices.iter() {
            center = center + *vertex;
            center.x /= 2.0;
            center.y /= 2.0;
        }

        let mut max_dist = 0.0;
        for vertex in self.vertices.iter() {
            let vdist = vertex.distance_squared(&center);
            max_dist = if vdist > max_dist { vdist } else { max_dist };
        }

        // TODO: actual w, h
        let (mut minx, mut miny, mut maxx, mut maxy) =
            (200 - 1, 200 - 1, 0, 0);

        for vertex in self.vertices.iter() {
            minx = min(minx, vertex.x as u32);
            miny = min(miny, vertex.y as u32);
            maxx = max(maxx, vertex.x as u32);
            maxy = max(maxy, vertex.y as u32);
        }

        self.edges = edges;
        self.center = center;
        self.max_dist = max_dist;
        self.bounding_box = (Point {x: minx as f32, y: miny as f32},
                             Point {x: maxx as f32, y: maxy as f32});
    }

    #[inline]
    pub fn query(&self, pt: &Point, antialias: bool) -> (bool, f32) {
        let mut inside = false;
        let mut min_dist = 100000.0;
        for &(a, b) in self.edges.iter() {
            if ((a.y > pt.y) != (b.y > pt.y)) &&
                (pt.x < (b.x - a.x) * (pt.y - a.y) / (b.y - a.y) + a.x)
            {
                inside = !inside;
            }

            if antialias {
                let ba = b - a;
                let mag = a.distance_squared(&b);
                let t = (*pt - a).dot(&ba) / mag;
                let dist = if t < 0.0 { pt.distance_squared(&a) }
                else if t > 1.0 { pt.distance_squared(&b) }
                else { pt.distance_squared(&(a + ba * t)) };

                min_dist = if dist < min_dist { dist } else { min_dist };
            }
        }

        (inside, if antialias { min_dist.sqrt() } else { 0.0 })
    }

    #[inline]
    fn rand_color(&self, base: u8) -> u8 {
        if should_mutate(CHANGE_COLOR_RATE) {
            min(max((base as f32 + (random::<f32>() - 0.5) * CHANGE_COLOR_MAX) as uint, 0), 255) as u8
        } else {
            base
        }
    }

    pub fn mutate(&mut self, dimensions: (u32, u32)) {
        let (mut r, mut g, mut b, mut a) = self.color;
        r = self.rand_color(r);
        g = self.rand_color(g);
        b = self.rand_color(b);
        a = if should_mutate(CHANGE_COLOR_RATE) { random::<u8>() % 60 + 30 } else { a };
        self.color = (r, g, b, a);

        if self.vertices.iter_mut().all(|v| v.mutate(dimensions)) {
            self.vertices = order_points(self.vertices.clone());
            self.update_data();
        }

        if should_mutate(ADD_VERTEX_RATE) {
            let index = random::<uint>() % (self.vertices.len() - 1);
            let (u, v) = (self.vertices[index], self.vertices[index + 1]);
            self.vertices.insert(index + 1, (u + v) * 0.5);
            self.update_data();
        }

        if should_mutate(REMOVE_VERTEX_RATE) && self.vertices.len() > 3 {
            let index = random::<uint>() % self.vertices.len();
            self.vertices.remove(index);
            self.update_data();
        }
    }

    pub fn raw(mut self) -> CPolygon {
        let (r, g, b, a) = self.color;
        CPolygon {
            num_vertices: self.vertices.len() as u32,
            vertices: self.vertices.as_mut_ptr(),
            r: r, g: g, b: b, a: a,
            center: self.center,
            max_dist: self.max_dist,
        }
    }
}

fn order_points(mut vertices: Vec<Point>) -> Vec<Point> {
    let (mut cx, mut cy) = (vertices[0].x, vertices[0].y);

    for vertex in vertices.iter() {
        cx = (cx + vertex.x) / 2.0;
        cy = (cy + vertex.y) / 2.0;
    }

    vertices.sort_by(|u, v| {
        let det = (u.x - cx) * (v.y - cy) - (v.x - cx) * (u.y - cy);
        if det < 0.0 { Less }
        else if det > 0.0 { Greater }
        else { Equal }
    });

    vertices
}

impl fmt::Show for Encoding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.polygons)
    }
}

impl Encoding {
    pub fn raw(mut self) -> CEncoding {
        let (width, height) = self.dimensions;
        let (poly_len, pixel_len) = (self.polygons.len() as u32, self.pixels.len() as u32);
        let mut polygons: Vec<CPolygon> = self.polygons.into_iter().map(|p| p.raw()).collect();

        CEncoding {
            polygons: polygons.as_mut_ptr(),
            num_polygons: poly_len,
            pixels: self.pixels.as_mut_ptr(),
            num_pixels: pixel_len,
            width: width,
            height: height,
        }
    }

    pub fn size(&self) -> uint {
        let mut size = 16; // width + height + num_pixels + num_polygons
        size += self.pixels.len() * (3 + 2); // color + position

        for polygon in self.polygons.iter() {
            size += 3 + 2 * polygon.vertices.len();
        }

        size
    }
}

impl fmt::Show for RGB {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", (self.r, self.g, self.b))
    }
}