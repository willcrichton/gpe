extern crate image;

use std::fmt;

#[deriving(Clone)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

pub type Color = image::Rgba<u8>;

#[deriving(Clone)]
pub struct Polygon {
    pub vertices: Vec<Point>,
    pub color: Color,
}

#[deriving(Clone)]
pub struct Encoding {
    pub polygons: Vec<Polygon>,
    pub dimensions: (u32, u32),
}

impl Point {
    pub fn inside_polygon(&self, polygon: &Polygon) -> bool {
        let mut inside = false;
        for &(a, b) in polygon.edges().iter() {
            if ((a.y > self.y) != (b.y > self.y)) &&
                (self.x < (b.x - a.x) * (self.y - a.y) / (b.y - a.y) + a.x)
            {
                inside = !inside;
            }
        }

        inside
    }

    /*fn magnitude(&self) -> f32 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }

    fn dot(&self, other: &Point) -> f32 {
        self.x * other.x + self.y * other.y
    }

    fn normalize(&self) -> Point {
        let mag = self.magnitude();
        Point {x: self.x / mag, y: self.y / mag}
    }

    fn cross_norm(&self, other: &Point) -> f32 {
        self.magnitude() * other.magnitude() *
            (1.0 - self.normalize().dot(&other.normalize()).powi(2)).sqrt()
    }*/
}

impl Add<Point, Point> for Point {
    fn add(&self, other: &Point) -> Point {
        Point {x: self.x + other.x, y: self.y + other.y}
    }
}

impl Sub<Point, Point> for Point {
    fn sub(&self, other: &Point) -> Point {
        Point {x: self.x - other.x, y: self.y - other.y}
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
    pub fn edges(&self) -> Vec<(Point, Point)> {
        let mut edges = Vec::new();
        let len = self.vertices.len();
        for i in range(0, len) {
            edges.push((self.vertices[i], self.vertices[(i + 1) % len]));
        }

        edges
    }
}
