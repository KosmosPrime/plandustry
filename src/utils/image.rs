use image::{Rgb, Rgba, RgbaImage};

pub trait ImageUtils {
    fn tint(&mut self, color: Rgb<u8>) -> &mut Self;

    fn repeat(&mut self, with: &RgbaImage) -> &mut Self;

    fn overlay(&mut self, with: &RgbaImage, x: i64, y: i64) -> &mut Self;

    fn scale(&mut self, to: u32) -> &mut Self;
}

impl ImageUtils for RgbaImage {
    fn tint(&mut self, color: Rgb<u8>) -> &mut Self {
        let [tr, tg, tb] = [
            color[0] as f32 / 255.0,
            color[1] as f32 / 255.0,
            color[2] as f32 / 255.0,
        ];
        for Rgba([r, g, b, _]) in self.pixels_mut() {
            *r = (*r as f32 * tr) as u8;
            *g = (*g as f32 * tg) as u8;
            *b = (*b as f32 * tb) as u8;
        }
        self
    }

    fn repeat(&mut self, with: &RgbaImage) -> &mut Self {
        for x in 0..(self.width() / with.width()) {
            for y in 0..(self.height() / with.height()) {
                image::imageops::overlay(
                    self,
                    with,
                    (x * with.width()).into(),
                    (y * with.height()).into(),
                );
            }
        }
        self
    }

    fn overlay(&mut self, with: &RgbaImage, x: i64, y: i64) -> &mut Self {
        image::imageops::overlay(self, with, x, y);
        self
    }

    fn scale(&mut self, to: u32) -> &mut Self {
        *self = image::imageops::resize(self, to, to, image::imageops::FilterType::Nearest);
        self
    }
}
