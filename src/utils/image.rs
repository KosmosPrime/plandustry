use image::*;

pub trait Overlay<W> {
    /// Overlay with => self at coordinates x, y, without blending
    fn overlay_at(&mut self, with: &W, x: u32, y: u32) -> &mut Self;
}

pub trait RepeatNew {
    /// Repeat with over self
    fn repeated(with: &Self, x: u32, y: u32) -> Self;
}

pub trait ImageUtils {
    /// Tint this image with the color
    fn tint(&mut self, color: Rgb<u8>) -> &mut Self;
    /// Overlay with => self (does not blend)
    fn overlay(&mut self, with: &Self) -> &mut Self;
    /// rotate
    fn rotate(&mut self, times: u8) -> &mut Self;
    /// flip along the horizontal axis
    fn flip_h(&mut self) -> &mut Self;
    /// flip along the vertical axis
    fn flip_v(&mut self) -> &mut Self;
    /// shadow
    fn shadow(&mut self) -> &mut Self;
    /// silhouette
    fn silhouette(&mut self) -> &mut Self;
    /// scale a image
    fn scale(&self, to: u32) -> Self;
}

impl Overlay<RgbImage> for RgbImage {
    fn overlay_at(&mut self, with: &RgbImage, x: u32, y: u32) -> &mut Self {
        for j in 0..with.height() {
            for i in 0..with.width() {
                let get = unsafe { with.unsafe_get_pixel(i, j) };
                unsafe { self.unsafe_put_pixel(i + x, j + y, get) };
            }
        }
        self
    }
}

impl Overlay<RgbaImage> for RgbImage {
    fn overlay_at(&mut self, with: &RgbaImage, x: u32, y: u32) -> &mut Self {
        for j in 0..with.height() {
            for i in 0..with.width() {
                let get = unsafe { with.unsafe_get_pixel(i, j) };
                // solidity
                if get[3] > 128 {
                    unsafe { self.unsafe_put_pixel(i + x, j + y, Rgb([get[0], get[1], get[2]])) };
                }
            }
        }
        self
    }
}

impl Overlay<RgbaImage> for RgbaImage {
    fn overlay_at(&mut self, with: &RgbaImage, x: u32, y: u32) -> &mut Self {
        for j in 0..with.height() {
            for i in 0..with.width() {
                let get = unsafe { with.unsafe_get_pixel(i, j) };
                if get[3] > 128 {
                    unsafe { self.unsafe_put_pixel(i + x, j + y, get) };
                }
            }
        }
        self
    }
}

impl RepeatNew for RgbImage {
    fn repeated(with: &Self, x: u32, y: u32) -> Self {
        let mut img = RgbImage::new(x, y); // could probably optimize this a ton but eh
        for x in 0..(x / with.width()) {
            for y in 0..(y / with.height()) {
                img.overlay_at(with, x * with.width(), y * with.height());
            }
        }
        img
    }
}

impl ImageUtils for RgbaImage {
    fn rotate(&mut self, times: u8) -> &mut Self {
        use image::imageops::{rotate180, rotate270, rotate90};
        match times {
            2 => *self = rotate180(self),
            1 => *self = rotate90(self),
            3 => *self = rotate270(self),
            _ => {}
        }
        self
    }

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

    fn overlay(&mut self, with: &RgbaImage) -> &mut Self {
        debug_assert_eq!(self.width(), with.width());
        debug_assert_eq!(self.height(), with.height());
        if self.len() % 4 != 0 || with.len() % 4 != 0 {
            unsafe { std::hint::unreachable_unchecked() };
        }
        for (i, other_pixels) in with.array_chunks::<4>().enumerate() {
            if other_pixels[3] > 128 {
                let own_pixels = unsafe { self.get_unchecked_mut(i * 4..i * 4 + 4) };
                own_pixels.copy_from_slice(other_pixels);
            }
        }
        self
    }

    fn scale(&self, to: u32) -> Self {
        imageops::resize(self, to, to, imageops::Nearest)
    }

    fn silhouette(&mut self) -> &mut Self {
        for pixel in self.pixels_mut() {
            if pixel[3] < 128 {
                pixel[2] /= 10;
                pixel[1] /= 10;
                pixel[0] /= 10;
            }
        }
        self
    }

    fn shadow(&mut self) -> &mut Self {
        let mut shadow = self.clone();
        shadow.silhouette();
        let samples = shadow.as_flat_samples_mut();
        blurslice::gaussian_blur_bytes::<4>(
            samples.samples,
            self.width() as usize,
            self.height() as usize,
            9.0,
        )
        .unwrap();
        for x in 0..shadow.width() {
            for y in 0..shadow.height() {
                let Rgba([r, g, b, a]) = self.get_pixel_mut(x, y);
                if *a == 0 {
                    let p = unsafe { shadow.unsafe_get_pixel(x, y) };
                    *r = p[0];
                    *g = p[0];
                    *b = p[0];
                    *a = p[1];
                }
            }
        }
        self
    }

    #[inline(always)]
    fn flip_h(&mut self) -> &mut Self {
        imageops::flip_horizontal_in_place(self);
        self
    }

    #[inline(always)]
    fn flip_v(&mut self) -> &mut Self {
        imageops::flip_vertical_in_place(self);
        self
    }
}
