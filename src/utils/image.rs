use fast_image_resize as fr;
use image::{Rgb, Rgba, RgbaImage};
use std::num::NonZeroU32;
pub trait ImageUtils {
    fn tint(&mut self, color: Rgb<u8>) -> &mut Self;

    fn repeat(&mut self, with: &RgbaImage) -> &mut Self;

    fn overlay(&mut self, with: &RgbaImage, x: u32, y: u32) -> &mut Self;

    unsafe fn scale(self, to: u32) -> Self;
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
                self.overlay(with, x * with.width(), y * with.height());
            }
        }
        self
    }

    fn overlay(&mut self, with: &RgbaImage, x: u32, y: u32) -> &mut Self {
        for j in 0..with.height() {
            for i in 0..with.width() {
                let get = with.get_pixel(i, j);
                if get[3] > 5 {
                    self.put_pixel(i + x, j + y, *get);
                }
            }
        }
        self
    }

    /// scales a image
    ///
    /// SAFETY: to and width and height cannot be 0.
    unsafe fn scale(self, to: u32) -> Self {
        debug_assert_ne!(to, 0);
        debug_assert_ne!(self.width(), 0);
        debug_assert_ne!(self.height(), 0);
        let to = NonZeroU32::new_unchecked(to);
        let src = fr::Image::from_vec_u8(
            NonZeroU32::new_unchecked(self.width()),
            NonZeroU32::new_unchecked(self.height()),
            self.into_vec(),
            fr::PixelType::U8x4,
        )
        .unwrap();
        let mut dst = fr::Image::new(to, to, fr::PixelType::U8x4);
        fr::Resizer::new(fr::ResizeAlg::Nearest)
            .resize(&src.view(), &mut dst.view_mut())
            .unwrap();
        RgbaImage::from_raw(to.get(), to.get(), dst.into_vec()).unwrap()
    }
}
