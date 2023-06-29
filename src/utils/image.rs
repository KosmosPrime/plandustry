use image::{Rgb, Rgba, RgbaImage};

pub fn tint(image: &mut RgbaImage, color: Rgb<u8>) {
    let [tr, tg, tb] = [
        color[0] as f32 / 255.0,
        color[1] as f32 / 255.0,
        color[2] as f32 / 255.0,
    ];
    for Rgba([r, g, b, _]) in image.pixels_mut() {
        *r = (*r as f32 * tr) as u8;
        *g = (*g as f32 * tg) as u8;
        *b = (*b as f32 * tb) as u8;
    }
}
