use std::fmt;
use std::num::Float;
use std::rand::random;

use render::Image;
use constants::*;

#[deriving(Clone)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

pub type Color = (u8, u8, u8, u8);

#[deriving(Clone)]
pub struct Polygon {
    pub vertices: Vec<Point>,
    pub color: Color,
    edges: Vec<(Point, Point)>,
    center: Point,
    max_dist: f32,
}

#[deriving(Clone)]
pub struct Encoding {
    pub polygons: Vec<Polygon>,
    pub dimensions: (u32, u32),
    pub render: Option<Image>,
}

impl Point {
    #[inline(always)]
    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    #[inline(always)]
    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    #[inline(always)]
    pub fn distance(&self, other: &Point) -> f32 {
        (*self - *other).length()
    }

    #[inline(always)]
    pub fn distance_squared(&self, other: &Point) -> f32 {
        let (x, y) = (self.x - other.x, self.y - other.y);
        x * x + y * y
    }

    #[inline(always)]
    pub fn dot(&self, other: &Point) -> f32 {
        self.x * other.x + self.y * other.y
    }

    pub fn mutate(&mut self, (w, h): (u32, u32)) {
        if should_mutate(MOVE_VERTEX_RATE) {
            self.x += (random::<f32>() - 0.5) * MOVE_VERTEX_MAX;
            self.y += (random::<f32>() - 0.5) * MOVE_VERTEX_MAX;
            clamp(self, (w, h));
        }
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
        write!(f, "{}", self.vertices)
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
        };

        polygon.update_data();
        polygon
    }

    pub fn random((w, h): (u32, u32)) -> Polygon {
        let origin = Point {x: random::<f32>() * (w as f32), y: random::<f32>() * (h as f32)};

        let mut vertices = vec![origin];
        for _ in range(0, VERTICES - 1) {
            let mut vtx = origin + Point{x: (random::<f32>() - 0.5) * (w as f32),
                                     y: (random::<f32>() - 0.5) * (h as f32)};
            clamp(&mut vtx, (w, h));
            vertices.push(vtx);
        }

        Polygon::new(order_points(vertices),
                     (random::<u8>(), random::<u8>(), random::<u8>(), 30 + random::<u8>() % 60))
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
            let vdist = vertex.distance(&center);
            max_dist = if vdist > max_dist { vdist } else { max_dist };
        }

        self.edges = edges;
        self.center = center;
        self.max_dist = max_dist;
    }

    #[inline(always)]
    pub fn query(&self, pt: &Point, antialias: bool) -> (bool, f32) {
        let mut inside = false;
        let mut min_dist = 100000.0;
        for &(a, b) in self.edges.iter() {
            let ba = b - a;
            if ((a.y > pt.y) != (b.y > pt.y)) &&
                (pt.x < (ba.x) * (pt.y - a.y) / (ba.y) + a.x)
            {
                inside = !inside;
            }

            if antialias {
                let mag = a.distance_squared(&b);
                let t = (*pt - a).dot(&ba) / mag;
                let dist = if t < 0.0 { pt.distance_squared(&a) }
                else if t > 1.0 { pt.distance_squared(&b) }
                else { pt.distance_squared(&(a + ba * t)) };

                min_dist = if dist < min_dist { dist } else { min_dist };
            }
        }

        (inside, min_dist.sqrt())
    }

    pub fn get_color(&self, p: Point) -> Color {
        let scale = p.distance(&self.center) / self.max_dist;
        let (r, g, b, a) = self.color;
        let convert = |n: u8| { ((n as f32) * (1.0 - scale)) as u8 };
        (convert(r), convert(g), convert(b), a)
    }

    pub fn mutate(&mut self, dimensions: (u32, u32)) {
        let (mut r, mut g, mut b, mut a) = self.color;
        r = if should_mutate(CHANGE_COLOR_RATE) { random::<u8>() } else { r };
        g = if should_mutate(CHANGE_COLOR_RATE) { random::<u8>() } else { g };
        b = if should_mutate(CHANGE_COLOR_RATE) { random::<u8>() } else { b };
        a = if should_mutate(CHANGE_COLOR_RATE) { random::<u8>() % 60 + 30 } else { a };
        self.color = (r, g, b, a);

        self.vertices.iter_mut().map(|v| v.mutate(dimensions)).count();

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
}

fn clamp(p: &mut Point, (w, h): (u32, u32)) {
    p.x = max(min(p.x, (w - 1) as f32), 0.0);
    p.y = max(min(p.y, (h - 1) as f32), 0.0);
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

#[inline(always)]
fn min(a: f32, b: f32) -> f32 {
    if a < b { a } else { b }
}

#[inline(always)]
fn max(a: f32, b: f32) -> f32 {
    if a < b { b } else { a }
}

impl fmt::Show for Encoding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.polygons)
    }
}
