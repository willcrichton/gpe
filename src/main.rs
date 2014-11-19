#![feature(globs)]

extern crate getopts;
extern crate image;

use std::os;
use std::io::File;

mod compress;
mod render;
mod encoding;

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

    let compressed = match image::open(&Path::new(matches.free[0].clone())).unwrap() {
        image::ImageRgb8(buf) => compress::compress(buf),
        _ => panic!("image must be RGB")
    };
    let output = render::render(&compressed);

    let save_file = File::create(&Path::new("out.png")).unwrap();
    let _ = image::ImageRgb8(output).save(save_file, image::PNG);
}

fn opts() -> Vec<getopts::OptGroup> {
    use getopts::optflag;
    vec![
        optflag("h", "help", "show this help message"),
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
