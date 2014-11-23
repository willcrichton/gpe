use std::rand::random;

pub static FITNESS_THRESHOLD: f32 = 0.65;
pub static INITIAL_POLYGONS: uint = 1;
pub static WORKERS: uint = 16;
pub static MUTATIONS: uint = 5;
pub static POPULATION_SIZE: uint = 10;
pub static VERTICES: uint = 8;

pub static ADD_POLYGON_RATE: uint = 400;
pub static REMOVE_POLYGON_RATE: uint = 1200;

pub static CHANGE_COLOR_RATE: uint = 1200;
pub static CHANGE_COLOR_MAX: f32 = 60.0;

pub static MOVE_VERTEX_RATE: uint = 1000;
pub static MOVE_VERTEX_MAX: f32 = 60.0;

pub static ADD_VERTEX_RATE: uint = 1000;
pub static REMOVE_VERTEX_RATE: uint = 1200;

#[inline(always)]
pub fn should_mutate(max: uint) -> bool {
    (random::<uint>() % max) == 1
}
