use std::rand::random;

pub static FITNESS_THRESHOLD: f32 = 0.95;
pub static PIXEL_FIX_THRESHOLD: uint = 30;
pub static INITIAL_POLYGONS: uint = 0;
pub static WORKERS: uint = 16;
pub static MUTATIONS: uint = 2;
pub static POPULATION_SIZE: uint = 16;
pub static VERTICES: uint = 6;
pub static POLY_SIZE_INIT: f32 = 50.0;

pub static ADD_POLYGON_RATE: uint = 30;
pub static MAX_POLYGONS: uint = 100;
pub static REMOVE_POLYGON_RATE: uint = 100;

pub static CHANGE_COLOR_RATE: uint = 50;
pub static CHANGE_COLOR_MAX: f32 = 150.0;

pub static MOVE_VERTEX_RATE: uint = 50;
pub static MOVE_VERTEX_MAX: f32 = 60.0;

pub static ADD_VERTEX_RATE: uint = 50;
pub static REMOVE_VERTEX_RATE: uint = 100;

pub static CHANGE_BLUR_RATE: uint = 60;

#[inline(always)]
pub fn should_mutate(max: uint) -> bool {
    (random::<uint>() % max) == 1
}
