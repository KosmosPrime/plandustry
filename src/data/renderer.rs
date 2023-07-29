//! schematic drawing
pub(crate) use super::autotile::*;
use crate::block::environment::METAL_FLOOR;
use crate::block::Rotation;
use crate::team::SHARDED;
pub(crate) use crate::utils::ImageUtils;
use crate::Map;
pub(crate) use image::{DynamicImage, RgbaImage};
pub(crate) use std::borrow::{Borrow, BorrowMut};
use std::ops::{Deref, DerefMut};
use std::sync::LazyLock;

use super::schematic::Schematic;
use super::GridPos;

macro_rules! r {
    ($v:expr) => {{
        static TMP: LazyLock<RgbaImage> = $v;
        &TMP
    }};
}

type Images = phf::Map<&'static str, &'static LazyLock<RgbaImage>>;
static FULL: Images = include!(concat!(env!("OUT_DIR"), "/full.rs"));
// static HALF: Images = include!(concat!(env!("OUT_DIR"), "/half.rs"));
static QUAR: Images = include!(concat!(env!("OUT_DIR"), "/quar.rs"));
static EIGH: Images = include!(concat!(env!("OUT_DIR"), "/eigh.rs"));

pub enum ImageHolder {
    Borrow(&'static RgbaImage),
    Own(RgbaImage),
}

impl ImageHolder {
    pub fn own(self) -> RgbaImage {
        match self {
            Self::Own(x) => x,
            Self::Borrow(x) => x.clone(),
        }
    }

    pub fn rotate(&mut self, times: u8) {
        if times == 0 {
            return;
        }
        let p: &mut RgbaImage = self.borrow_mut();
        p.rotate(times);
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
        Self::Borrow(value)
    }
}

impl From<RgbaImage> for ImageHolder {
    fn from(value: RgbaImage) -> Self {
        Self::Own(value)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Scale {
    Full,
    // Half,
    Quarter,
    Eigth,
}

impl Scale {
    fn px(self) -> u8 {
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

pub(crate) fn try_load(name: &str, scale: Scale) -> Option<&'static RgbaImage> {
    let key = name.to_string();
    match scale {
        Scale::Quarter => QUAR.get(&key).map(|v| LazyLock::force(v)),
        Scale::Eigth => EIGH.get(&key).map(|v| LazyLock::force(v)),
        Scale::Full => FULL.get(&key).map(|v| LazyLock::force(v)),
        // Scale::Half => HALF.get(&key).map(|v| LazyLock::force(v)),
    }
}

pub(crate) fn load(name: &str, scale: Scale) -> ImageHolder {
    ImageHolder::from(
        try_load(name, scale)
            .ok_or_else(|| format!("failed to load {name}"))
            .unwrap(),
    )
}

const SUFFIXES: &[&str; 9] = &[
    "-bottom", "-mid", "-base", "", "-left", "-right", "-top", "-over", "-team",
];
pub(crate) fn read<S>(name: &str, size: S, scale: Scale) -> ImageHolder
where
    S: Into<u32> + Copy,
{
    read_with(name, SUFFIXES, size, scale)
}

pub(crate) fn read_with<S>(
    name: &str,
    suffixes: &'static [&'static str],
    size: S,
    scale: Scale,
) -> ImageHolder
where
    S: Into<u32> + Copy,
{
    let mut c = RgbaImage::new(
        size.into() * scale.px() as u32,
        size.into() * scale.px() as u32,
    );
    for suffix in suffixes {
        if let Some(p) = try_load(&format!("{name}{suffix}"), scale) {
            if suffix == &"-team" {
                c.overlay(p.clone().tint(SHARDED.color()));
                continue;
            }
            c.overlay(&p);
        }
    }
    ImageHolder::from(c)
}

/// trait for renderable objects
pub trait Renderable {
    /// creates a picture of a schematic. Bridges and node connections are not drawn.
    fn render(&self) -> RgbaImage;
}

impl Renderable for Schematic<'_> {
    /// ```
    /// use mindus::*;
    /// let mut s = Schematic::new(2, 3);
    /// s.put(0, 0, &block::distribution::DISTRIBUTOR);
    /// s.put(0, 2, &block::distribution::ROUTER);
    /// s.put(1, 2, &block::walls::COPPER_WALL);
    /// let output /*: RgbaImage */ = s.render();
    /// ```
    fn render(&self) -> RgbaImage {
        // fill background
        let mut bg = RgbaImage::new(
            ((self.width + 2) * 32) as u32,
            ((self.height + 2) * 32) as u32,
        );
        bg.repeat(
            METAL_FLOOR
                .image(None, None, Rotation::Up, Scale::Full)
                .borrow(),
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

        #[cfg(feature = "schem_shadow")]
        image::imageops::overlay(&mut bg, canvas.shadow(), 0, 0);
        #[cfg(not(feature = "schem_shadow"))]
        bg.overlay(&canvas);
        bg
    }
}

impl Renderable for Map<'_> {
    fn render(&self) -> RgbaImage {
        let scale = if self.width + self.height < 2000 {
            Scale::Quarter
        } else {
            Scale::Eigth
        };
        let mut floor = RgbaImage::new(scale * self.width as u32, scale * self.height as u32);
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
            let img = tile.floor_image(None, scale);
            // println!("draw {tile:?} ({x}, {y}) + {scale:?}");
            // assert_eq!(img.width(), scale.px() as u32);
            // assert_eq!(img.height(), scale.px() as u32);
            floor.overlay_at(&img, scale * x as u32, scale * y as u32);
            if let Some(build) = tile.build() {
                let s = build.block.get_size();
                let x = x - ((s - 1) / 2) as usize;
                let y = y - (s / 2) as usize;
                let ctx = (|| {
                    if !build.block.wants_context() {
                        return None;
                    }
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
                })();
                let img = tile.build_image(ctx.as_ref(), scale);
                // assert_eq!(img.width(), scale * build.block.get_size() as u32);
                // assert_eq!(img.height(), scale * build.block.get_size() as u32);
                top.overlay_at(&img, scale * x as u32, scale * y as u32);
            }
        }
        #[cfg(feature = "map_shadow")]
        image::imageops::overlay(&mut floor, top.shadow(), 0, 0);
        #[cfg(not(feature = "map_shadow"))]
        floor.overlay(&top);
        floor
    }
}

#[test]
fn all_blocks() {
    use crate::block::content::Type;
    use crate::content::Content;
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

        let t = reg.get(t.get_name()).unwrap();
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
        );
    }
}
