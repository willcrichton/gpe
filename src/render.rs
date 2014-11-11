extern crate shader_version;
extern crate event;
extern crate graphics;
extern crate sdl2_window;
extern crate opengl_graphics;

use std::cell::RefCell;

use self::graphics::*;
use self::opengl_graphics::{Gl};
use self::sdl2_window::Sdl2Window;
use self::event::{Events, WindowSettings};
use encoding::Encoding;

fn render_image(c: &Context, gl: &mut Gl, img: &Encoding) {
    for polygon in img.polygons().iter() {
        let (r, g, b, a) = polygon.color;
        c.polygon(&*polygon.vertices).rgba(r, g, b, a).draw(gl);
    }
}

pub fn render(img: Encoding) {
    let opengl = shader_version::opengl::OpenGL_3_2;
    let (width, height) = img.dimensions();
    let window = Sdl2Window::new(
        opengl,
        WindowSettings {
            title: "GPE".to_string(),
            size: [width, height],
            fullscreen: false,
            exit_on_esc: true,
            samples: 0
        });

    let ref mut gl = Gl::new(opengl);
    let window = RefCell::new(window);
    for e in Events::new(&window) {
        use self::event::RenderEvent;
        e.render(|_| {
            gl.viewport(0, 0, width as i32, height as i32);
            let c = Context::abs(width as f64, height as f64);
            c.rgb(1.0, 1.0, 1.0).draw(gl);
            render_image(&c, gl, &img);
        });
    }
}