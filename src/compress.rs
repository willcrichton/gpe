extern crate image;

use image::GenericImage;
use encoding::{Encoding, Polygon, Point};
use render::{render, Image};

static ITERATIONS: int = 50;
static MUTATIONS: int = 10;

pub fn compress(img: Image) -> Encoding {
    let (w, h) = img.dimensions();
    let mut population = create_population(&img);

    for _ in range(0, ITERATIONS) {
        population = mutate(population);
    }

    /*let poly = Polygon {
        vertices: vec![Point {x: 0.0, y: 0.0}, Point {x: 0.0, y: 10.9}, Point {x: 10.0, y: 0.0}],
        color: image::Rgba(0, 255, 0, 255)
    };

    let poly2 = Polygon {
        vertices: vec![Point {x: 20.0, y: 20.0}, Point {x: 20.0, y: 40.0}, Point{x: 40.0, y: 20.0}],
        color: image::Rgba(255, 0, 0, 255)
    };

    let encoding = Encoding { dimensions: (w, h), polygons: vec![poly, poly2] };
    let output = render(&encoding);
    println!("{}", fitness(&img, &output));*/

    population[0].clone()
}

fn create_population(base: &Image) -> Vec<Encoding> {
    Vec::new()
}

fn mutate(population: Vec<Encoding>) -> Vec<Encoding> {
    let mut new_population = Vec::new();
    for candidate in population.into_iter() {
        for _ in range(0, MUTATIONS) {

        }
    }

    new_population
}

fn diff(a: u8, b: u8) -> uint {
    (if a > b { a - b } else { b - a }) as uint
}

fn fitness(base: &Image, new: &Image) -> uint {
    let (w, h) = base.dimensions();
    let mut score = 0;

    for y in range(0, h) {
        for x in range(0, w) {
            let (br, bg, bb) = base.get_pixel(x, y).channels();
            let (nr, ng, nb) = new.get_pixel(x, y).channels();

            score += diff(br, nr) + diff(bg, ng) + diff(bb, nb);
        }
    }

    score
}