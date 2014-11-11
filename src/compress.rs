extern crate image;

use image::GenericImage;
use encoding::{Encoding, Polygon};

pub fn compress(img: image::DynamicImage) -> Encoding {
    let (w, h) = img.dimensions();

    let poly = Polygon {
        vertices: box [[0.0, 0.0], [10.0, 0.0], [0.0, 10.0]],
        color: (1.0, 0.0, 0.0)
    };

    let poly2 = Polygon {
        vertices: box [[20.0, 20.0], [20.0, 40.0], [40.0, 20.0]],
        color: (0.0, 1.0, 0.0)
    };

    Encoding::new(w, h, vec![poly, poly2])
}