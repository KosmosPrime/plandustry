//! schematic drawing
pub(crate) use super::autotile::*;
use super::schematic::Schematic;
use super::GridPos;
use crate::block::Rotation;
pub(crate) use crate::utils::{Image, ImageHolder, ImageUtils, Overlay, Repeat};
use crate::Map;
include!(concat!(env!("OUT_DIR"), "/full.rs"));
include!(concat!(env!("OUT_DIR"), "/quar.rs"));
include!(concat!(env!("OUT_DIR"), "/eigh.rs"));

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
	("empty", $scale:expr) => {
		 ImageHolder::from(match $scale {
            $crate::data::renderer::Scale::Quarter => &$crate::data::renderer::quar::EMPTY,
            $crate::data::renderer::Scale::Eigth => &$crate::data::renderer::eigh::EMPTY,
            $crate::data::renderer::Scale::Full => &$crate::data::renderer::full::EMPTY,
        }.copy())
	};
    ($name:literal, $scale:expr) => { paste::paste! {
        ImageHolder::from(match $scale {
            $crate::data::renderer::Scale::Quarter => &$crate::data::renderer::quar::[<$name:snake:upper>],
            $crate::data::renderer::Scale::Eigth => &$crate::data::renderer::eigh::[<$name:snake:upper>],
            $crate::data::renderer::Scale::Full => &$crate::data::renderer::full::[<$name:snake:upper>],
        }.copy())
    } };
    ($name: literal) => { paste::paste! {
        [$crate::data::renderer::full::[<$name:snake:upper>].copy(), $crate::data::renderer::quar::[<$name:snake:upper>].copy(), $crate::data::renderer::eigh::[<$name:snake:upper>].copy()]
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
                ImageHolder::from(match $scale {
                    $crate::data::renderer::Scale::Quarter => &$crate::data::renderer::quar::[<$k:snake:upper _ $x:snake:upper>],
                    $crate::data::renderer::Scale::Eigth => &$crate::data::renderer::eigh::[<$k:snake:upper _ $x:snake:upper>],
                    $crate::data::renderer::Scale::Full => &$crate::data::renderer::full::[<$k:snake:upper _ $x:snake:upper>],
                }.copy()),
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
    #[must_use = "i did so much work for you"]
    fn render(&self) -> Image<Vec<u8>, 3>;
}

impl Renderable for Schematic<'_> {
    /// creates a picture of a schematic. Bridges and node connections are not drawn.
    /// ```
    /// use mindus::*;
    /// let mut s = Schematic::new(2, 3);
    /// s.put(0, 0, &block::distribution::DISTRIBUTOR);
    /// s.put(0, 2, &block::distribution::ROUTER);
    /// s.put(1, 2, &block::walls::COPPER_WALL);
    /// let output /*: Image */ = s.render();
    /// ```
    fn render(&self) -> Image<Vec<u8>, 3> {
        // fill background
        let mut bg = load!("metal-floor", Scale::Full).borrow().repeated(
            ((self.width + 2) * 32) as u32,
            ((self.height + 2) * 32) as u32,
        );
        let mut canvas = Image::alloc(
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
            canvas.as_mut().overlay_at(
                &tile
                    .image(
                        ctx.as_ref(),
                        tile.get_rotation().unwrap_or(Rotation::Up),
                        Scale::Full,
                    )
                    .borrow(),
                (x + 1) * 32,
                (y + 1) * 32,
            );
        }
        canvas.as_mut().shadow();
        for x in 0..canvas.width() {
            for y in 0..canvas.height() {
                // canvas has a shadow
                let p2 = unsafe { canvas.pixel(x, y) };
                let p = unsafe { bg.pixel_mut(x, y) };
                crate::utils::image::blend(p.try_into().unwrap(), p2);
            }
        }
        bg.remove_channel()
    }
}

impl Renderable for Map<'_> {
    /// Draws a map
    fn render(&self) -> Image<Vec<u8>, 3> {
        let scale = if self.width + self.height < 2000 {
            Scale::Quarter
        } else {
            Scale::Eigth
        };
        // todo combine these (beware of floor drawing atop buildings) (planned solution:? ptr blocks)
        let mut floor: Image<_, 3> =
            Image::alloc(scale * self.width as u32, scale * self.height as u32);
        let mut top: Image<_, 4> =
            Image::alloc(scale * self.width as u32, scale * self.height as u32);
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
            // println!("draw {tile:?} ({x}, {y}) + {scale:?}");
            floor.as_mut().overlay_at(
                &tile.floor(scale).borrow(),
                scale * x as u32,
                scale * y as u32,
            );
            if tile.has_ore() {
                floor.as_mut().overlay_at(
                    &tile.ore(scale).borrow(),
                    scale * x as u32,
                    scale * y as u32,
                );
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
                top.as_mut().overlay_at(
                    &tile.build_image(ctx.as_ref(), scale).borrow(),
                    scale * x as u32,
                    scale * y as u32,
                );
            }
        }
        floor.as_mut().overlay_at(&top.as_ref(), 0, 0);
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
        let name = dbg!(t.get_name());
        let t = reg.get(name).unwrap();
        let _ = t.image(
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
