extern crate image;

use std::sync::{Arc, TaskPool};
use std::rand::random;

use image::GenericImage;
use encoding::{Encoding, Polygon};
use render::{render, Image};
use constants::*;

struct Compressor {
    dimensions: (u32, u32),
    base: Arc<Image>
}

pub fn compress(img: image::ImageBuf<image::Rgb<u8>>) -> Encoding {
    let dimensions = img.dimensions();
    let buf = Arc::new(img.into_vec().into_iter().map(|p| p.channels()).collect());
    let compressor = Compressor { dimensions: dimensions, base: buf };
    let mut population = compressor.create_population();
    let max_score = compressor.max_score();

    let mut iteration = 0u;
    loop {
        let (new_population, min_fitness, index) = compressor.mutate(population);
        population = new_population;
        let current_score = 1.0 - (min_fitness as f32 / max_score as f32);
        info!("Iteration {} (size {}, score {})", iteration,
              population[0].polygons.len(), current_score);
        if current_score >= ::threshold() { return population[index].clone(); }
        iteration += 1;
    }
}

#[inline(always)]
fn diff(a: u8, b: u8) -> uint {
    (if a > b { a - b } else { 2 * (b - a) }) as uint
}

fn fitness((w, h): (u32, u32), base: Arc<Image>, individual: Arc<Option<Encoding>>) -> uint {
    let mut score = 0;
    let individual = individual.as_ref().unwrap();
    let new_render = render(individual, false);

    for i in range(0, w * h) {
        let (br, bg, bb) = base[i as uint];
        let (nr, ng, nb) = new_render[i as uint];

        score += diff(br, nr) + diff(bg, ng) + diff(bb, nb);
    }

    score
}
impl Compressor {
    fn create_population(&self) -> Vec<Encoding> {
        let mut population = Vec::new();
        for _ in range(0, POPULATION_SIZE) {
            let mut polygons = Vec::new();
            for _ in range(0, INITIAL_POLYGONS) {
                let polygon = Polygon::random(self.dimensions);
                polygons.push(polygon);
            }

            population.push(Encoding { dimensions: self.dimensions,
                                       polygons: polygons,
                                       render: None });
        }

        population
    }

    fn mutate(&self, population: Vec<Encoding>) -> (Vec<Encoding>, uint, uint) {
        let mut new_population = Vec::new();
        for candidate in population.into_iter() {
            for _ in range(0, MUTATIONS) {
                let mut candidate = candidate.clone();

                let mut new_polygons = Vec::new();
                for mut polygon in candidate.polygons.into_iter() {
                    if should_mutate(REMOVE_POLYGON_RATE) { continue; }
                    polygon.mutate(self.dimensions);
                    new_polygons.push(polygon);
                }

                candidate.polygons = new_polygons;

                if should_mutate(ADD_POLYGON_RATE) {
                    candidate.polygons.push(Polygon::random(self.dimensions));
                }

                new_population.push(Arc::new(Some(candidate)));
            }

            new_population.push(Arc::new(Some(candidate)));
        }

        let pool = TaskPool::new(WORKERS);
        let (tmaster, rmaster) = channel();
        for (i, individual) in new_population.iter().enumerate() {
            let (tx, rx) = channel();
            let tmaster = tmaster.clone();
            pool.execute(proc() {
                let (dimensions, base, i, individual) = rx.recv();
                tmaster.send((i, fitness(dimensions, base, individual)));
            });

            tx.send((self.dimensions, self.base.clone(), i, individual.clone()));
        }

        let mut population_fitness = Vec::new();
        for _ in new_population.iter() {
            population_fitness.push(rmaster.recv());
        }

        population_fitness.sort_by(|&(_, a), &(_, b)| a.cmp(&b));

        let mut filtered_population = Vec::new();
        let (mut min_fitness, mut min_individual) = (population_fitness[0].val1(), 0);
        for i in range(0, POPULATION_SIZE) {
            let (index, fitvalue) = population_fitness[i as uint];
            if fitvalue < min_fitness {
                min_fitness = fitvalue;
                min_individual = index;
            }

            filtered_population.push(new_population[index].make_unique().take().unwrap());
        }

        (filtered_population, min_fitness, min_individual)
    }

    pub fn max_score(&self) -> uint {
        let (w, h) = self.dimensions;
        let mut max_score = 0u;
        for i in range(0, w * h) {
            let (r, g, b) = self.base[i as uint];
            max_score += (if r > 128 { r } else { 255 - r }) as uint;
            max_score += (if g > 128 { g } else { 255 - g }) as uint;
            max_score += (if b > 128 { b } else { 255 - b }) as uint;
        }

        max_score
    }
}