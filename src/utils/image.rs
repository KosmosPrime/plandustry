use fast_image_resize as fr;
use std::{num::NonZeroU32, slice::SliceIndex};

pub trait Overlay<W> {
    /// Overlay with => self at coordinates x, y, without blending
    fn overlay_at(&mut self, with: &W, x: u32, y: u32) -> &mut Self;
}

pub trait RepeatNew {
    type Output;
    /// Repeat self till it fills x, y
    fn repeated(&self, x: u32, y: u32) -> Self::Output;
}

pub trait ImageUtils {
    type With<'a>;
    /// Tint this image with the color
    fn tint(&mut self, color: (u8, u8, u8)) -> &mut Self;
    /// Overlay with => self (does not blend)
    fn overlay(&mut self, with: Self::With<'_>) -> &mut Self;
    /// rotate (squares only)
    fn rotate(&mut self, times: u8) -> &mut Self;
    /// flip along the horizontal axis
    fn flip_h(&mut self) -> &mut Self;
    /// flip along the vertical axis
    fn flip_v(&mut self) -> &mut Self;
    /// shadow
    fn shadow(&mut self) -> &mut Self;
    /// scale a image
    fn scale(self, to: u32) -> Image<Vec<u8>, 4>;
}

macro_rules! unsafe_assert {
    ($cond:expr) => {{
        if !$cond {
            unsafe { std::hint::unreachable_unchecked() }
        }
    }};
}

impl Overlay<Image<&[u8], 3>> for Image<&mut [u8], 3> {
    fn overlay_at(&mut self, with: &Image<&[u8], 3>, x: u32, y: u32) -> &mut Self {
        for j in 0..with.height() {
            for i in 0..with.width() {
                unsafe {
                    let with_index = with.slice(i, j);
                    let their_px = with.buffer.get_unchecked(with_index);
                    let our_index =
                        really_unsafe_index(i.unchecked_add(x), j.unchecked_add(y), self.width())
                            .unchecked_mul(3);
                    let our_px = self
                        .buffer
                        .get_unchecked_mut(our_index..our_index.unchecked_add(3));
                    std::ptr::copy_nonoverlapping(their_px.as_ptr(), our_px.as_mut_ptr(), 3);
                }
            }
        }
        self
    }
}

impl Overlay<Image<&[u8], 4>> for Image<&mut [u8], 3> {
    fn overlay_at(&mut self, with: &Image<&[u8], 4>, x: u32, y: u32) -> &mut Self {
        for j in 0..with.height() {
            for i in 0..with.width() {
                unsafe {
                    let with_index = really_unsafe_index(i, j, with.width()).unchecked_mul(4);
                    // solidity
                    if *with.buffer.get_unchecked(with_index.unchecked_add(3)) > 128 {
                        let their_px = with
                            .buffer
                            .get_unchecked(with_index..with_index.unchecked_add(3));
                        let our_index = really_unsafe_index(
                            i.unchecked_add(x),
                            j.unchecked_add(y),
                            self.width(),
                        )
                        .unchecked_mul(3);
                        let our_px = self
                            .buffer
                            .get_unchecked_mut(our_index..our_index.unchecked_add(3));
                        std::ptr::copy_nonoverlapping(their_px.as_ptr(), our_px.as_mut_ptr(), 3);
                    }
                }
            }
        }
        self
    }
}

impl Overlay<Image<&[u8], 4>> for Image<&mut [u8], 4> {
    fn overlay_at(&mut self, with: &Image<&[u8], 4>, x: u32, y: u32) -> &mut Self {
        for j in 0..with.height() {
            for i in 0..with.width() {
                unsafe {
                    let with_index = really_unsafe_index(i, j, with.width()).unchecked_mul(4);
                    let their_px = with
                        .buffer
                        .get_unchecked(with_index..with_index.unchecked_add(4));
                    if their_px.get_unchecked(3) > &128 {
                        let our_index = really_unsafe_index(
                            i.unchecked_add(x),
                            j.unchecked_add(y),
                            self.width(),
                        )
                        .unchecked_mul(4);
                        let our_px = self
                            .buffer
                            .get_unchecked_mut(our_index..our_index.unchecked_add(4));
                        std::ptr::copy_nonoverlapping(their_px.as_ptr(), our_px.as_mut_ptr(), 4);
                    }
                }
            }
        }
        self
    }
}

impl RepeatNew for Image<&[u8], 4> {
    type Output = Image<Vec<u8>, 4>;
    fn repeated(&self, x: u32, y: u32) -> Self::Output {
        let mut img = Image::alloc(x, y); // could probably optimize this a ton but eh
        for x in 0..(x / self.width()) {
            for y in 0..(y / self.height()) {
                let a: &mut Image<&mut [u8], 4> = &mut img.as_mut();
                a.overlay_at(self, x * self.width(), y * self.height());
            }
        }
        img
    }
}

unsafe fn flip_v<const CHANNELS: usize>(img: &mut Image<&mut [u8], CHANNELS>) {
    for y in 0..img.height() / 2 {
        for x in 0..img.width() {
            let y2 = img.height() - y - 1;
            let p2 = img.pixel(x, y2);
            let p = img.pixel(x, y);
            img.set_pixel(x, y2, p);
            img.set_pixel(x, y, p2);
        }
    }
}

unsafe fn flip_h<const CHANNELS: usize>(img: &mut Image<&mut [u8], CHANNELS>) {
    for y in 0..img.height() {
        for x in 0..img.width() / 2 {
            let x2 = img.width() - x - 1;
            let p2 = img.pixel(x2, y);
            let p = img.pixel(x, y);
            img.set_pixel(x2, y, p);
            img.set_pixel(x, y, p2);
        }
    }
}

unsafe fn rot_180<const CHANNELS: usize>(img: &mut Image<&mut [u8], CHANNELS>) {
    for y in 0..img.height() / 2 {
        for x in 0..img.width() {
            let p = img.pixel(x, y);
            let x2 = img.width() - x - 1;
            let y2 = img.height() - y - 1;
            let p2 = img.pixel(x2, y2);
            img.set_pixel(x, y, p2);
            img.set_pixel(x2, y2, p);
        }
    }

    if img.height() % 2 != 0 {
        let middle = img.height() / 2;

        for x in 0..img.width() / 2 {
            let p = img.pixel(x, middle);
            let x2 = img.width() - x - 1;

            let p2 = img.pixel(x2, middle);
            img.set_pixel(x, middle, p2);
            img.set_pixel(x2, middle, p);
        }
    }
}

/// only works with squares!
unsafe fn rot_90<const CHANNELS: usize>(img: &mut Image<&mut [u8], CHANNELS>) {
    debug_assert_eq!(img.width(), img.height());
    let size = img.width();
    flip_v(img);
    for i in 0..size {
        for j in i..size {
            for c in 0..CHANNELS {
                img.buffer.swap(
                    (i * size + j) as usize * CHANNELS + c,
                    (j * size + i) as usize * CHANNELS + c,
                );
            }
        }
    }
}

/// only works with squares!
unsafe fn rot_270<const CHANNELS: usize>(img: &mut Image<&mut [u8], CHANNELS>) {
    debug_assert_eq!(img.width(), img.height());
    flip_h(img);
    let size = img.width();
    for i in 0..size {
        for j in i..size {
            for c in 0..CHANNELS {
                img.buffer.swap(
                    (i * size + j) as usize * CHANNELS + c,
                    (j * size + i) as usize * CHANNELS + c,
                );
            }
        }
    }
}
impl ImageUtils for Image<&mut [u8], 4> {
    fn rotate(&mut self, times: u8) -> &mut Self {
        unsafe {
            match times {
                2 => rot_180(self),
                1 => rot_90(self),
                3 => rot_270(self),
                _ => {}
            }
        }
        self
    }

    fn tint(&mut self, (r, g, b): (u8, u8, u8)) -> &mut Self {
        let [tr, tg, tb] = [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0];
        for [r, g, b, _] in self.buffer.array_chunks_mut::<4>() {
            *r = (*r as f32 * tr) as u8;
            *g = (*g as f32 * tg) as u8;
            *b = (*b as f32 * tb) as u8;
        }
        self
    }
    type With<'a> = &'a Image<&'a [u8], 4>;
    fn overlay(&mut self, with: &Image<&[u8], 4>) -> &mut Self {
        debug_assert_eq!(self.width(), with.width());
        debug_assert_eq!(self.height(), with.height());
        unsafe_assert!(self.buffer.len() % 4 == 0);
        unsafe_assert!(with.buffer.len() % 4 == 0);
        for (i, other_pixels) in with.buffer.array_chunks::<4>().enumerate() {
            if other_pixels[3] > 128 {
                unsafe {
                    let own_pixels = self
                        .buffer
                        .get_unchecked_mut(i.unchecked_mul(4)..i.unchecked_mul(4).unchecked_add(4));
                    std::ptr::copy_nonoverlapping(
                        other_pixels.as_ptr(),
                        own_pixels.as_mut_ptr(),
                        4,
                    );
                }
            }
        }
        self
    }

    // this function is very cold but im removing image so might as well use fir
    fn scale(self, to: u32) -> Image<Vec<u8>, 4> {
        let from =
            fr::Image::from_slice_u8(self.width, self.height, self.buffer, fr::PixelType::U8x4)
                .unwrap();
        let mut dst = fr::Image::new(
            to.try_into().unwrap(),
            to.try_into().unwrap(),
            fr::PixelType::U8x4,
        );
        fr::Resizer::new(fr::ResizeAlg::Nearest)
            .resize(&from.view(), &mut dst.view_mut())
            .unwrap();
        Image::new(self.width, self.height, dst.into_vec())
    }

    fn shadow(&mut self) -> &mut Self {
        let mut shadow: Image<Vec<u8>, 4> =
            Image::new(self.width, self.height, self.buffer.to_vec());
        for [r, g, b, a] in shadow.buffer.array_chunks_mut() {
            if *a < 128 {
                *r /= 10;
                *g /= 10;
                *b /= 10;
            }
        }
        blurslice::gaussian_blur_bytes::<4>(
            &mut shadow.buffer,
            self.width() as usize,
            self.height() as usize,
            9.0,
        )
        .unwrap();
        for ([r, g, b, a], &[from_r, from_g, from_b, from_a]) in self
            .buffer
            .array_chunks_mut()
            .zip(shadow.buffer.array_chunks())
        {
            if *a == 0 {
                (*r, *g, *b, *a) = (from_r, from_g, from_b, from_a);
            }
        }
        self
    }

    #[inline]
    fn flip_h(&mut self) -> &mut Self {
        unsafe { flip_h(self) };
        self
    }

    #[inline(always)]
    fn flip_v(&mut self) -> &mut Self {
        unsafe { flip_v(self) };
        self
    }
}

unsafe fn really_unsafe_index(x: u32, y: u32, w: u32) -> usize {
    // y * w + x
    (y as usize)
        .unchecked_mul(w as usize)
        .unchecked_add(x as usize)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Image<T, const CHANNELS: usize> {
    pub buffer: T,
    pub width: NonZeroU32,
    pub height: NonZeroU32,
}

impl<const CHANNELS: usize> Default for Image<&'static [u8], CHANNELS> {
    fn default() -> Self {
        Self {
            buffer: &[0; CHANNELS],
            width: NonZeroU32::new(1).unwrap(),
            height: NonZeroU32::new(1).unwrap(),
        }
    }
}

impl<T, const CHANNELS: usize> Image<T, CHANNELS> {
    #[inline]
    pub fn height(&self) -> u32 {
        self.height.into()
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.width.into()
    }

    #[inline]
    pub const fn new(width: NonZeroU32, height: NonZeroU32, buffer: T) -> Self {
        Image {
            width,
            height,
            buffer,
        }
    }
}

impl<const CHANNELS: usize> Image<&[u8], CHANNELS> {
    #[inline]
    pub const fn copy(&self) -> Self {
        Self {
            width: self.width,
            height: self.height,
            buffer: self.buffer,
        }
    }
}

impl<T: std::ops::Deref<Target = [u8]>, const CHANNELS: usize> Image<T, CHANNELS> {
    /// # Safety
    ///
    /// - UB if x, y is out of bounds
    /// - UB if buffer is too small
    pub unsafe fn slice(&self, x: u32, y: u32) -> impl SliceIndex<[u8], Output = [u8]> {
        debug_assert!(x < self.width(), "x out of bounds");
        debug_assert!(y < self.height(), "y out of bounds");
        let index = really_unsafe_index(x, y, self.width()).unchecked_mul(CHANNELS);
        debug_assert!(self.buffer.len() > index);
        index..index.unchecked_add(CHANNELS)
    }

    #[inline]
    /// Return a pixel at (x, y).
    /// # Safety
    ///
    /// Refer to [`slice`]
    pub unsafe fn pixel(&self, x: u32, y: u32) -> [u8; CHANNELS] {
        *(self.buffer.get_unchecked(self.slice(x, y)).as_ptr().cast())
    }
}
impl<T: std::ops::DerefMut<Target = [u8]>, const CHANNELS: usize> Image<T, CHANNELS> {
    #[inline]
    /// Return a mutable reference to a pixel at (x, y).
    /// # Safety
    ///
    /// Refer to [`slice`]
    pub unsafe fn pixel_mut(&mut self, x: u32, y: u32) -> &mut [u8] {
        let idx = self.slice(x, y);
        self.buffer.get_unchecked_mut(idx)
    }
    #[inline]
    pub unsafe fn set_pixel(&mut self, x: u32, y: u32, px: [u8; CHANNELS]) {
        std::ptr::copy_nonoverlapping(px.as_ptr(), self.pixel_mut(x, y).as_mut_ptr(), CHANNELS);
    }
}

impl<const CHANNELS: usize> Image<Vec<u8>, CHANNELS> {
    pub fn alloc(width: u32, height: u32) -> Self {
        Image {
            width: width.try_into().unwrap(),
            height: height.try_into().unwrap(),
            buffer: vec![0; CHANNELS * width as usize * height as usize],
        }
    }

    pub fn as_ref(&self) -> Image<&[u8], CHANNELS> {
        Image::new(self.width, self.height, &self.buffer)
    }

    pub fn as_mut(&mut self) -> Image<&mut [u8], CHANNELS> {
        Image::new(self.width, self.height, &mut self.buffer)
    }
}

impl Image<Vec<u8>, 4> {
    pub fn remove_channel(&mut self) -> Image<Vec<u8>, 3> {
        let mut new = vec![0; self.width() as usize * self.height() as usize * 3];
        for (&[r, g, b, _], [nr, ng, nb]) in self
            .buffer
            .array_chunks::<4>()
            .zip(new.array_chunks_mut::<3>())
        {
            (*nr, *ng, *nb) = (r, g, b);
        }
        Image::new(self.width, self.height, new)
    }
}

impl Image<Vec<u8>, 3> {
    #[cfg(feature = "bin")]
    pub fn save(&self, f: impl AsRef<std::path::Path>) {
        image::save_buffer(
            f,
            &self.buffer,
            self.width(),
            self.height(),
            image::ColorType::Rgb8,
        )
        .unwrap();
    }
}

#[derive(Clone, Debug)]
pub enum ImageHolder<const CHANNELS: usize> {
    Borrow(Image<&'static [u8], CHANNELS>),
    Own(Image<Vec<u8>, CHANNELS>),
}

impl<const CHANNELS: usize> ImageHolder<CHANNELS> {
    #[must_use]
    pub fn own(self) -> Image<Vec<u8>, CHANNELS> {
        match self {
            Self::Own(x) => x,
            Self::Borrow(x) => Image::new(x.width, x.height, x.buffer.to_vec()),
        }
    }
}

impl<const CHANNELS: usize> ImageHolder<CHANNELS> {
    #[must_use]
    #[inline]
    pub fn borrow(&self) -> Image<&[u8], CHANNELS> {
        match self {
            Self::Own(x) => x.as_ref(),
            Self::Borrow(x) => x.clone(),
        }
    }

    #[must_use]
    #[inline]
    pub fn borrow_mut(&mut self) -> Image<&mut [u8], CHANNELS> {
        match self {
            Self::Own(x) => Image::new(x.width, x.height, &mut x.buffer),
            Self::Borrow(_) => {
                *self = Self::from(std::mem::replace(self, Self::from(Image::default())).own());
                self.borrow_mut()
            }
        }
    }
}

impl Overlay<ImageHolder<4>> for ImageHolder<4> {
    fn overlay_at(&mut self, with: &ImageHolder<4>, x: u32, y: u32) -> &mut Self {
        self.borrow_mut().overlay_at(&with.borrow(), x, y);
        self
    }
}

impl ImageUtils for ImageHolder<4> {
    fn tint(&mut self, color: (u8, u8, u8)) -> &mut Self {
        self.borrow_mut().tint(color);
        self
    }
    type With<'a> = &'a Self;
    fn overlay(&mut self, with: &Self) -> &mut Self {
        self.borrow_mut().overlay(&with.borrow());
        self
    }

    fn rotate(&mut self, times: u8) -> &mut Self {
        if times == 0 {
            return self;
        }
        // borrow mut may clone, so try to avoid
        self.borrow_mut().rotate(times);
        self
    }

    fn flip_h(&mut self) -> &mut Self {
        self.borrow_mut().flip_h();
        self
    }

    fn flip_v(&mut self) -> &mut Self {
        self.borrow_mut().flip_v();
        self
    }

    fn shadow(&mut self) -> &mut Self {
        self.borrow_mut().shadow();
        self
    }

    fn scale(mut self, to: u32) -> Image<Vec<u8>, 4> {
        self.borrow_mut().scale(to)
    }
}

impl<const CHANNELS: usize> From<Image<&'static [u8], CHANNELS>> for ImageHolder<CHANNELS> {
    fn from(value: Image<&'static [u8], CHANNELS>) -> Self {
        Self::Borrow(value)
    }
}

impl<const CHANNELS: usize> From<Image<Vec<u8>, CHANNELS>> for ImageHolder<CHANNELS> {
    fn from(value: Image<Vec<u8>, CHANNELS>) -> Self {
        Self::Own(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    macro_rules! img {
        [[$($v:literal),+] [$($v2:literal),+]] => {{
            let from: Image<Vec<u8>, 1> = Image::new(
                2.try_into().unwrap(),
                2.try_into().unwrap(),
                vec![$($v,)+ $($v2,)+]
            );
            from
        }}
    }

    #[test]
    fn rem_chnl_test() {
        let mut img: Image<_, 4> = Image::alloc(2, 1);
        unsafe { img.set_pixel(1, 0, [255, 165, 0, 241]) };
        assert_eq!(unsafe { img.pixel(1, 0) }, [255, 165, 0, 241]);
        assert_eq!(unsafe { img.pixel(0, 0) }, [0, 0, 0, 0]);
        let img = img.remove_channel();
        assert_eq!(unsafe { img.pixel(1, 0) }, [255, 165, 0]);
    }

    #[test]
    fn rotate_90() {
        let mut from = img![
            [00, 01]
            [02, 10]
        ];
        unsafe { rot_90(&mut from.as_mut()) };
        assert_eq!(
            from,
            img![
                [02, 00]
                [10, 01]
            ]
        );
    }

    #[test]
    fn rotate_180() {
        let mut from = img![
            [00, 01]
            [02, 10]
        ];
        unsafe { rot_180(&mut from.as_mut()) };
        assert_eq!(
            from,
            img![
                [10, 02]
                [01, 00]
            ]
        );
    }

    #[test]
    fn rotate_270() {
        let mut from = img![
            [00, 01]
            [20, 10]
        ];
        unsafe { rot_270(&mut from.as_mut()) };
        assert_eq!(
            from,
            img![
                [01, 10]
                [00, 20]
            ]
        );
    }

    #[test]
    fn flip_vertical() {
        let mut from = img![
            [90, 01]
            [21, 42]
        ];
        unsafe { flip_v(&mut from.as_mut()) };
        assert_eq!(
            from,
            img![
                [21, 42]
                [90, 01]
            ]
        )
    }
    #[test]
    fn flip_horizontal() {
        let mut from = img![
            [90, 01]
            [21, 42]
        ];
        unsafe { flip_h(&mut from.as_mut()) };
        assert_eq!(
            from,
            img![
                [01, 90]
                [42, 21]
            ]
        )
    }
}

pub fn blend(bg: &mut [u8; 4], fg: [u8; 4]) {
    if fg[3] == 0 {
        return;
    }
    if fg[3] == 255 {
        *bg = fg;
        return;
    }
    let bg_a = bg[3] as f32 / 255.0;
    let fg_a = fg[3] as f32 / 255.0;
    let a = bg_a + fg_a - bg_a * fg_a;
    if a == 0.0 {
        return;
    };
    *bg = [
        (255.0
            * ((((fg[0] as f32 / 255.0) * fg_a) + ((bg[0] as f32 / 255.0) * bg_a) * (1.0 - fg_a))
                / a)) as u8,
        (255.0
            * ((((fg[1] as f32 / 255.0) * fg_a) + ((bg[1] as f32 / 255.0) * bg_a) * (1.0 - fg_a))
                / a)) as u8,
        (255.0
            * ((((fg[2] as f32 / 255.0) * fg_a) + ((bg[2] as f32 / 255.0) * bg_a) * (1.0 - fg_a))
                / a)) as u8,
        (255.0 * a) as u8,
    ]
}
