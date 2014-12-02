#![feature(globs)]
#![feature(phase)]
#![feature(default_type_params)]
#[phase(plugin, link)] extern crate log;

extern crate getopts;
extern crate image;

use std::os;
use std::io::File;
use image::GenericImage;

mod compress;
mod render;
mod encoding;
mod constants;
mod fnvhasher;

local_data_key!(THRESHOLD: f32)

fn main() {
    let args = os::args();
    let matches = match getopts::getopts(args.tail(), opts().as_slice()) {
        Ok(m) => m,
        Err(err) => return println!("{}", err),
    };

    if matches.opt_present("h") {
        return usage(args[0].as_slice(), None);
    }

    if matches.free.len() == 0 {
        return usage(args[0].as_slice(), Some("expected an input image to compress"));
    } if matches.free.len() > 1 {
        return usage(args[0].as_slice(), Some("can only compress one file at a time"));
    }

    let threshold = match matches.opt_str("t") {
        Some(s) => from_str(s.as_slice()).unwrap(),
        None => constants::FITNESS_THRESHOLD
    };
    THRESHOLD.replace(Some(threshold));

    let (compressed, w, h) = match image::open(&Path::new(matches.free[0].clone())).unwrap() {
        image::ImageRgb8(buf) => {
            let (w, h) = buf.dimensions();
            (compress::compress(buf), w, h)
        },
        _ => panic!("image must be RGB")
    };

    println!("{}", compressed);
    let output = render::render(&compressed, true);

    let save_file = File::create(&Path::new("out.png")).unwrap();
    let pixels = output.into_iter().map(|(r, g, b)| image::Rgb(r, g, b)).collect();
    let buf = image::ImageBuf::from_pixels(pixels, w, h);
    let _ = image::ImageRgb8(buf).save(save_file, image::PNG);
}

fn opts() -> Vec<getopts::OptGroup> {
    use getopts::{optflag, optopt};
    vec![
        optflag("h", "help", "show this help message"),
        optopt("t", "threshold", "terminate after reaching this fitness threshold", "0.75"),
        ]
}

fn usage(argv0: &str, err: Option<&str>) {
    match err {
        Some(e) => println!("error: {}", e),
        None => {}
    }
    println!("{}", getopts::usage(format!("{} [options] <input>", argv0).as_slice(),
                                  opts().as_slice()));
}

pub fn threshold() -> f32 {
    *THRESHOLD.get().unwrap()
}
