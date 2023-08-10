//! schematic drawing
pub(crate) use super::autotile::*;
use super::schematic::Schematic;
use super::GridPos;
use crate::block::environment::METAL_FLOOR;
use crate::block::Rotation;
pub(crate) use crate::utils::{ImageUtils, Overlay, Repeat};
use crate::Map;
pub(crate) use image::{
    DynamicImage, GenericImage, GenericImageView, Pixel, Rgb, RgbImage, Rgba, RgbaImage,
};
pub(crate) use std::borrow::{Borrow, BorrowMut};
use std::ops::{Deref, DerefMut};
include!(concat!(env!("OUT_DIR"), "/full.rs"));
include!(concat!(env!("OUT_DIR"), "/quar.rs"));
include!(concat!(env!("OUT_DIR"), "/eigh.rs"));

pub enum ImageHolder {
    Borrow(&'static RgbaImage),
    Own(RgbaImage),
}

impl ImageHolder {
    #[must_use]
    pub fn own(self) -> RgbaImage {
        match self {
            Self::Own(x) => x,
            Self::Borrow(x) => x.clone(),
        }
    }

    pub fn rotate(&mut self, times: u8) -> &mut Self {
        if times == 0 {
            return self;
        }
        let p: &mut RgbaImage = self.borrow_mut();
        p.rotate(times);
        self
    }
}

impl Borrow<RgbaImage> for ImageHolder {
    fn borrow(&self) -> &RgbaImage {
        match self {
            Self::Own(x) => x,
            Self::Borrow(x) => x,
        }
    }
}

impl BorrowMut<RgbaImage> for ImageHolder {
    fn borrow_mut(&mut self) -> &mut RgbaImage {
        match self {
            Self::Own(x) => x,
            Self::Borrow(_) => {
                *self = Self::from(std::mem::replace(self, Self::from(RgbaImage::new(0, 0))).own());
                self.borrow_mut()
            }
        }
    }
}

impl Deref for ImageHolder {
    type Target = RgbaImage;
    fn deref(&self) -> &Self::Target {
        self.borrow()
    }
}

impl DerefMut for ImageHolder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.borrow_mut()
    }
}

impl From<&'static RgbaImage> for ImageHolder {
    fn from(value: &'static RgbaImage) -> Self {
        debug_assert_ne!(value.width(), 0);
        debug_assert_ne!(value.height(), 0);
        Self::Borrow(value)
    }
}

impl From<RgbaImage> for ImageHolder {
    fn from(value: RgbaImage) -> Self {
        Self::Own(value)
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum Scale {
    Full,
    // Half,
    Quarter,
    Eigth,
}

impl Scale {
    #[must_use]
    pub const fn px(self) -> u8 {
        match self {
            Self::Full => 32,
            Self::Quarter => 32 / 4,
            Self::Eigth => 32 / 8,
        }
    }
}

impl std::ops::Mul<u32> for Scale {
    type Output = u32;
    fn mul(self, rhs: u32) -> u32 {
        self.px() as u32 * rhs
    }
}

#[macro_export]
macro_rules! load {
	("empty", $scale:ident) => {
		 ImageHolder::from(unsafe { $crate::utils::Lock::get(match $scale {
            $crate::data::renderer::Scale::Quarter => &$crate::data::renderer::quar::EMPTY,
            $crate::data::renderer::Scale::Eigth => &$crate::data::renderer::eigh::EMPTY,
            $crate::data::renderer::Scale::Full => &$crate::data::renderer::full::EMPTY,
        })})
	};
    ($name:literal, $scale:ident) => { paste::paste! {
        ImageHolder::from(unsafe { $crate::utils::Lock::get(match $scale {
            $crate::data::renderer::Scale::Quarter => $crate::data::renderer::quar::[<$name:snake:upper>],
            $crate::data::renderer::Scale::Eigth => $crate::data::renderer::eigh::[<$name:snake:upper>],
            $crate::data::renderer::Scale::Full => $crate::data::renderer::full::[<$name:snake:upper>],
        })})
    } };
    ($name: literal) => { paste::paste! {
        [$crate::data::renderer::full::[<$name:snake:upper>], $crate::data::renderer::quar::[<$name:snake:upper>], $crate::data::renderer::eigh::[<$name:snake:upper>]]
    } };
    (from $v:ident which is [$($k:literal $(|)?)+], $scale: ident) => {
        $crate::data::renderer::load!($scale -> match $v {
            $($k => $k,)+
        })
    };
    // turn load!(s -> match x { "v" => "y" }) into match x { "v" => load!("y", s) }
    ($scale:ident -> match $v:ident { $($k:pat => $nam:literal $(,)?)+ }) => {
        match $v {
            $($k => $crate::data::renderer::load!($nam, $scale),)+
            #[allow(unreachable_patterns)]
            n => unreachable!("{n:?}"),
        }
    };
    (concat $x:expr => $v:ident which is [$($k:literal $(|)?)+], $scale: ident) => { paste::paste! {
        match $v {
            $($k =>
                ImageHolder::from(unsafe { $crate::utils::Lock::get(match $scale {
                    $crate::data::renderer::Scale::Quarter => $crate::data::renderer::quar::[<$k:snake:upper _ $x:snake:upper>],
                    $crate::data::renderer::Scale::Eigth => $crate::data::renderer::eigh::[<$k:snake:upper _ $x:snake:upper>],
                    $crate::data::renderer::Scale::Full => $crate::data::renderer::full::[<$k:snake:upper _ $x:snake:upper>],
                }) }),
            )+
            #[allow(unreachable_patterns)]
            n => unreachable!("{n:?}"),
        }
    } };
}
pub(crate) use load;

/// trait for renderable objects
pub trait Renderable {
    /// create a picture
    ///
    /// # Safety
    ///
    /// UB if called before [`warmup`](crate::warmup)
    unsafe fn render(&self) -> RgbImage;
}

impl Renderable for Schematic<'_> {
    /// creates a picture of a schematic. Bridges and node connections are not drawn.
    /// ```
    /// use mindus::*;
    /// let mut s = Schematic::new(2, 3);
    /// s.put(0, 0, &block::distribution::DISTRIBUTOR);
    /// s.put(0, 2, &block::distribution::ROUTER);
    /// s.put(1, 2, &block::walls::COPPER_WALL);
    /// // warm up the images for the first time
    /// unsafe { warmup(); }
    /// // this is now safe, because we have warmed up
    /// let output /*: RgbImage */ = unsafe { s.render() };
    /// ```
    ///
    /// # Safety
    ///
    /// UB if called before [`warmup`](crate::warmup)
    unsafe fn render(&self) -> RgbImage {
        // fill background
        let mut bg = RgbImage::repeated(
            &DynamicImage::from(
                METAL_FLOOR
                    .image(None, None, Rotation::Up, Scale::Full)
                    .own(),
            )
            .into_rgb8(),
            ((self.width + 2) * 32) as u32,
            ((self.height + 2) * 32) as u32,
        );
        let mut canvas = RgbaImage::new(
            ((self.width + 2) * 32) as u32,
            ((self.height + 2) * 32) as u32,
        );
        for (GridPos(x, y), tile) in self.block_iter() {
            let ctx = if tile.block.wants_context() {
                let pctx = PositionContext {
                    position: GridPos(x, y),
                    width: self.width,
                    height: self.height,
                };
                Some(RenderingContext {
                    cross: self.cross(&pctx),
                    position: pctx,
                })
            } else {
                None
            };
            let x = x as u32 - ((tile.block.get_size() - 1) / 2) as u32;
            let y = self.height as u32 - y as u32 - ((tile.block.get_size() / 2) + 1) as u32;
            canvas.overlay_at(
                tile.image(
                    ctx.as_ref(),
                    tile.get_rotation().unwrap_or(Rotation::Up),
                    Scale::Full,
                )
                .borrow(),
                (x + 1) * 32,
                (y + 1) * 32,
            );
        }
        canvas.shadow();
        for x in 0..canvas.width() {
            for y in 0..canvas.height() {
                let p2 = unsafe { canvas.unsafe_get_pixel(x, y) };
                let Rgb([r2, g2, b2]) = unsafe { bg.unsafe_get_pixel(x, y) };
                let mut p = Rgba([r2, g2, b2, u8::MAX]);
                p.blend(&p2);
                let Rgba([r, g, b, a]) = p;
                let a = a as f32 / 255.;
                let p = Rgb([
                    (((r as f32 / 255.) * a) * 255.) as u8,
                    (((g as f32 / 255.) * a) * 255.) as u8,
                    (((b as f32 / 255.) * a) * 255.) as u8,
                ]);
                unsafe { bg.unsafe_put_pixel(x, y, p) };
            }
        }
        bg
    }
}

impl Renderable for Map<'_> {
    /// Draws a map
    ///
    /// # Safety
    /// UB if called before [`warmup`](crate::warmup)
    unsafe fn render(&self) -> RgbImage {
        let scale = if self.width + self.height < 2000 {
            Scale::Quarter
        } else {
            Scale::Eigth
        };
        // todo combine these (beware of floor drawing atop buildings) (planned solution:? ptr blocks)
        let mut floor = RgbImage::new(scale * self.width as u32, scale * self.height as u32);
        let mut top = RgbaImage::new(scale * self.width as u32, scale * self.height as u32);
        for (x, y, j, tile) in self.tiles.iter().enumerate().map(|(j, t)| {
            (
                (j % self.width),
                // flip y
                (self.height - (j / self.width)) - 1,
                j,
                t,
            )
        }) {
            // draw the floor first.
            let flo: &RgbaImage = &tile.floor(scale);
            // println!("draw {tile:?} ({x}, {y}) + {scale:?}");
            // debug_assert_eq!(floor.width(), scale.px() as u32);
            // debug_assert_eq!(floor.height(), scale.px() as u32);
            floor.overlay_at(flo, scale * x as u32, scale * y as u32);
            if tile.has_ore() {
                let ore: &RgbaImage = &tile.ore(scale);
                // debug_assert_eq!(ore.width(), scale.px() as u32);
                // debug_assert_eq!(ore.height(), scale.px() as u32);
                floor.overlay_at(ore, scale * x as u32, scale * y as u32);
            }

            if let Some(build) = tile.build() {
                let s = build.block.get_size();
                let x = x - ((s - 1) / 2) as usize;
                let y = y - (s / 2) as usize;
                let ctx = if build.block.wants_context() {
                    let pctx = PositionContext {
                        position: GridPos(x, y),
                        width: self.width,
                        height: self.height,
                    };
                    let rctx = RenderingContext {
                        cross: self.cross(j, &pctx),
                        position: pctx,
                    };
                    Some(rctx)
                } else {
                    None
                };
                let img: &RgbaImage = &tile.build_image(ctx.as_ref(), scale);
                // debug_assert_eq!(img.width(), scale * build.block.get_size() as u32);
                // debug_assert_eq!(img.height(), scale * build.block.get_size() as u32);
                top.overlay_at(img, scale * x as u32, scale * y as u32);
            }
        }
        floor.overlay_at(&top, 0, 0);
        floor
    }
}

#[allow(clippy::needless_doctest_main)]
/// Loads all the images into memory (about 300mb)
/// This is a necessary function. Call it once in main.
///
/// ```
/// fn main() {
///     unsafe { mindus::warmup(); }
/// }
/// ```
///
/// # Safety
///
/// only call once, else UB
pub unsafe fn warmup() {
    full::warmup();
    quar::warmup();
    eigh::warmup();
}

#[test]
fn all_blocks() {
    use crate::block::content::Type;
    use crate::content::Content;
    unsafe { warmup() };
    let reg = crate::block::build_registry();
    for t in 19..Type::WorldMessage as u16 {
        let t = Type::try_from(t).unwrap();
        if matches!(t, |Type::Empty| Type::SlagCentrifuge
            | Type::HeatReactor
            | Type::LegacyMechPad
            | Type::LegacyUnitFactory
            | Type::LegacyUnitFactoryAir
            | Type::LegacyUnitFactoryGround
            | Type::CommandCenter)
        {
            continue;
        }
        let name = dbg!(t.get_name());
        let t = reg.get(name).unwrap();
        let _ = unsafe {
            t.image(
                None,
                Some(&RenderingContext {
                    cross: [None; 4],
                    position: PositionContext {
                        position: GridPos(0, 0),
                        width: 5,
                        height: 5,
                    },
                }),
                Rotation::Up,
                Scale::Quarter,
            )
        };
    }
}
