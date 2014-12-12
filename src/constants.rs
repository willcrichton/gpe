use std::rand::random;

pub static FITNESS_THRESHOLD: f32 = 0.95;
pub static PIXEL_FIX_THRESHOLD: f32 = 50.0;
pub static INITIAL_POLYGONS: uint = 0;
pub static WORKERS: uint = 16;
pub static MUTATIONS: uint = 1;
pub static POPULATION_SIZE: uint = 16;
pub static VERTICES: uint = 5;
pub static POLY_SIZE_INIT: f32 = 50.0;

pub static ADD_POLYGON_RATE: uint = 15;
pub static MAX_POLYGONS: uint = 100;
pub static REMOVE_POLYGON_RATE: uint = 80;

pub static CHANGE_COLOR_RATE: uint = 40;
pub static CHANGE_COLOR_MAX: f32 = 150.0;

pub static MOVE_VERTEX_RATE: uint = 40;
pub static MOVE_VERTEX_MAX: f32 = 60.0;

pub static ADD_VERTEX_RATE: uint = 40;
pub static REMOVE_VERTEX_RATE: uint = 80;

pub static CHANGE_BLUR_RATE: uint = 60;

#[inline(always)]
pub fn should_mutate(max: uint) -> bool {
    (random::<uint>() % max) == 1
}
