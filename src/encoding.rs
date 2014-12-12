use std::fmt;
use std::num::Float;
use std::rand::random;
use std::cmp::{min, max};
use std::iter::range_inclusive;

use constants::*;
use compress::Compressor;

#[deriving(Clone)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

pub type Color = (u8, u8, u8, u8);

#[deriving(Clone)]
pub struct Pixel {
    pub pos: Point,
    pub color: (u8, u8, u8),
}

#[deriving(Clone)]
pub struct Polygon {
    pub vertices: Vec<Point>,
    pub color: Color,
    pub blur: f32,
    edges: Vec<(Point, Point)>,
    center: Point,
    max_dist: f32,
    pub bounding_box: (Point, Point),
}

#[deriving(Clone)]
pub struct Encoding {
    pub polygons: Vec<Polygon>,
    pub dimensions: (u32, u32),
    pub pixels: Vec<Pixel>,
}

#[inline(always)]
pub fn fmin(a: f32, b: f32) -> f32 {
    if a < b { a } else { b }
}

#[inline(always)]
pub fn fmax(a: f32, b: f32) -> f32 {
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

impl Div<f32, Point> for Point {
    #[inline(always)]
    fn div(&self, constant: &f32) -> Point {
        Point {x: self.x / *constant, y: self.y / *constant}
    }
}

impl Equiv<Point> for Point {
    fn equiv(&self, other: &Point) -> bool {
        (self.x - other.x).abs() < 0.001 && (self.y - other.y).abs() < 0.001
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
            blur: 0.0,
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
        let origin = if compressor.error.iter().fold(0, |b, a| b + *a) > 0 {
            let mut regions: Vec<(uint, &uint)> = compressor.error.iter().enumerate().collect();
            regions.sort_by(|&(_, a), &(_, b)| b.cmp(a));
            let (region, _) = regions[random::<uint>() % 4];

            let (x, y) = (region % 8, region / 8);

            Point {x: (x as f32) * 25.0 + random::<f32>() * 25.0,
                   y: (y as f32) * 25.0 + random::<f32>() * 25.0}
        } else {
            Point {x: random::<f32>() * (w as f32),
                   y: random::<f32>() * (h as f32)}
        };


        let mut vertices = vec![origin];
        for _ in range(0, VERTICES - 1) {
            let mut vtx = origin + Point{x: (random::<f32>() - 0.5) * POLY_SIZE_INIT,
                                     y: (random::<f32>() - 0.5) * POLY_SIZE_INIT};
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
                         random::<u8>() % 130 + 125);

        polygon.blur = 0.5 + random::<f32>() * 0.5;

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
        }

        center = center / self.vertices.len() as f32;

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

    pub fn mutate(&mut self, compressor: &Compressor) {
        let (mut r, mut g, mut b, mut a) = self.color;
        r = self.rand_color(r);
        g = self.rand_color(g);
        b = self.rand_color(b);
        a = if should_mutate(CHANGE_COLOR_RATE) { random::<u8>() % 130 + 125 } else { a };
        self.color = (r, g, b, a);

        if self.vertices.iter_mut().all(|v| v.mutate(compressor.dimensions)) {
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

        if should_mutate(CHANGE_BLUR_RATE) {
            self.blur = 0.5 + random::<f32>() * 0.5;
        }
    }
}

fn order_points(mut vertices: Vec<Point>) -> Vec<Point> {
    vertices.sort_by(|u, v| {
        if u.x < v.x { Less }
        else { Greater }
    });

    let mut hull = vec![];
    let mut point_on_hull = vertices[0];

    loop {
        hull.push(point_on_hull);

        let mut endpoint = vertices[0];
        for vertex in vertices.iter() {
            // A = endpoint, B = point_on_hull, S[j] = vertex
            let line_side = (point_on_hull.x - endpoint.x) * (vertex.y - endpoint.y) -
                (point_on_hull.y - endpoint.y) * (vertex.x - endpoint.x);
            if endpoint.equiv(&point_on_hull) || line_side > 0.0 {
                endpoint = *vertex;
            }
        }

        point_on_hull = endpoint;

        if endpoint.equiv(&hull[0]) { break; }
    }

    hull
}

impl fmt::Show for Encoding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.polygons)
    }
}

impl Encoding {
    pub fn size(&self) -> uint {
        let mut size = 16; // width + height + num_pixels + num_polygons
        size += self.pixels.len() * (3 + 2); // color + position

        for polygon in self.polygons.iter() {
            size += 3 + 2 * polygon.vertices.len();
        }

        size
    }
}