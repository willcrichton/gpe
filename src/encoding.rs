pub struct Polygon {
    pub vertices: Box<[[f64, ..2]]>,
    pub color: (f32, f32, f32, f32),
}

pub struct Encoding {
    polygons: Vec<Polygon>,
    dimensions: (u32, u32),
}

impl Encoding {
    pub fn new(width: u32, height: u32, polygons: Vec<Polygon>) -> Encoding {
        Encoding {
            polygons: polygons,
            dimensions: (width, height)
        }
    }

    pub fn dimensions(&self) -> (u32, u32) {
        self.dimensions
    }

    pub fn polygons(&self) -> &Vec<Polygon> {
        &self.polygons
    }
}