use encoding::{Encoding, CEncoding, RGB};

type BufColor = (u8, u8, u8);
pub type Image = Vec<BufColor>;

extern {
    fn cuda_render(img: CEncoding, output: *mut RGB, antialias: bool);
}

pub fn render(img: &Encoding, antialias: bool) -> Image {
    let (w, h) = img.dimensions;
    let mut imgbuf = Vec::from_fn((w * h) as uint, |_| RGB {r: 0, g: 0, b: 0});

    unsafe {
        cuda_render(img.clone().raw(), imgbuf.as_mut_ptr(), antialias);
    }

    return imgbuf.into_iter().map(|color| (color.r, color.g, color.b)).collect();
}