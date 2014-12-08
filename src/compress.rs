extern crate image;
extern crate time;

use std::sync::{Arc, TaskPool};
use std::rand::random;

use image::GenericImage;
use encoding::{Encoding, Polygon, Pixel, Point, RGB};
use render::{render, Image};
use constants::*;

pub struct Compressor {
    pub dimensions: (u32, u32),
    pub base: Arc<Image>,
}

pub fn compress(img: image::ImageBuf<image::Rgb<u8>>) -> Encoding {
    let dimensions = img.dimensions();
    let buf = Arc::new(img.into_vec().into_iter().map(|p| p.channels()).collect());
    let compressor = Compressor { dimensions: dimensions, base: buf };
    let mut population = compressor.create_population();
    let max_score = compressor.max_score();

    let mut iteration = 0u;
    let mut cur_time = time::get_time();
    let mut avg_time = 0.0;
    loop {
        let (new_population, min_fitness, index) = compressor.mutate(population);

        iteration += 1;
        let new_time = time::get_time();
        let diff = (new_time.nsec - cur_time.nsec) / 1000000;

        let iter_f = iteration as f32;
        avg_time = avg_time * (iter_f - 1.0) / iter_f +
            (if diff < 0 { avg_time } else { diff as f32 }) / iter_f;
        cur_time = new_time;

        population = new_population;
        let current_score = 1.0 - (min_fitness as f32 / max_score as f32);
        info!("Iteration {} (size {}, score {}, time {}ms)", iteration,
              population[0].polygons.len(), current_score, diff);
        if current_score >= ::threshold() {
            info!("Average time: {}ms", avg_time);
            return if !::should_fix() { population[index].clone() }
            else { compressor.fix_pixels(population[index].clone()) }
        }
    }
}

#[inline(always)]
fn diff(a: u8, b: u8) -> uint {
    let diff = if a > b { (a - b) as uint } else { (b - a) as uint };
    diff * diff
}

fn fitness((w, h): (u32, u32), base: Arc<Image>, individual: Arc<Option<Encoding>>) -> uint {
    let mut score = 0;
    let individual = individual.as_ref().unwrap();
    let new_render = render(individual, false);

    for i in range(0, w * h) {
        let (br, bg, bb) = base[i as uint];
        let (nr, ng, nb) = new_render[i as uint];

        score += 2 * diff(br, nr) + 3 * diff(bg, ng) + diff(bb, nb);
    }

    score
}
impl Compressor {
    fn create_population(&self) -> Vec<Encoding> {
        let mut population = vec![];
        for _ in range(0, POPULATION_SIZE) {
            let mut polygons = vec![];
            for _ in range(0, INITIAL_POLYGONS) {
                match Polygon::random(self) {
                    Some(p) => { polygons.push(p); },
                    None => {}
                };
            }

            population.push(Encoding { dimensions: self.dimensions,
                                       polygons: polygons,
                                       pixels: vec![] });
        }

        population
    }

    fn mutate(&self, population: Vec<Encoding>) -> (Vec<Encoding>, uint, uint) {
        let mut new_population = vec![];
        for candidate in population.into_iter() {
            for _ in range(0, MUTATIONS) {
                let mut candidate = candidate.clone();

                let mut new_polygons = vec![];
                for mut polygon in candidate.polygons.into_iter() {
                    if should_mutate(REMOVE_POLYGON_RATE) { continue; }
                    polygon.mutate(self.dimensions);
                    new_polygons.push(polygon);
                }

                candidate.polygons = new_polygons;

                if should_mutate(ADD_POLYGON_RATE) {
                    match Polygon::random(self) {
                        Some(p) => { candidate.polygons.push(p); }
                        None => {}
                    }
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

        let mut population_fitness = vec![];
        for _ in new_population.iter() {
            population_fitness.push(rmaster.recv());
        }

        population_fitness.sort_by(|&(_, a), &(_, b)| a.cmp(&b));

        let mut filtered_population = vec![];
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
        fitness(
            (w, h),
            self.base.clone(),
            Arc::new(Some(Encoding {
                dimensions: (w, h),
                polygons: vec![],
                pixels: vec![] })))
    }


    pub fn fix_pixels(&self, mut img: Encoding) -> Encoding {
        let (w, h) = img.dimensions;
        let new_render = render(&img, true);
        for y in range(0, h) {
            for x in range(0, w) {
                let i = (y * w + x) as uint;
                let (br, bg, bb) = self.base[i];
                let (nr, ng, nb) = new_render[i];
                let score = 2 * diff(br, nr) + 3 * diff(bg, ng) + diff(bb, nb);

                if score > PIXEL_FIX_THRESHOLD {
                    img.pixels.push(Pixel {
                        pos: Point { x: x as f32, y: y as f32 },
                        color: RGB { r: br, g: bg, b: bb }
                    });
                }
            }
        }

        info!("Fixed {} pixels", img.pixels.len());

        img
    }
}