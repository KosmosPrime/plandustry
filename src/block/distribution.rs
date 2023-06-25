use std::any::Any;
use std::error::Error;
use std::fmt;

use image::{Rgba, RgbaImage};

use crate::block::simple::{cost, state_impl, BuildCost, SimpleBlock};
use crate::block::{
    impl_block, make_register, BlockLogic, DataConvertError, DeserializeError, SerializeError,
};
use crate::content;
use crate::data::dynamic::{DynData, DynType};
use crate::data::renderer::load;
use crate::data::GridPos;
use crate::item;
use crate::item::storage::Storage;

make_register! {
    "conveyor" => SimpleBlock::new(1, false, cost!(Copper: 1));
    "titanium-conveyor" => SimpleBlock::new(1, false, cost!(Copper: 1, Lead: 1, Titanium: 1));
    "plastanium-conveyor" => SimpleBlock::new(1, false, cost!(Graphite: 1, Silicon: 1, Plastanium: 1));
    "armored-conveyor" => SimpleBlock::new(1, false, cost!(Metaglass: 1, Thorium: 1, Plastanium: 1));
    "junction" => SimpleBlock::new(1, true, cost!(Copper: 2));
    "bridge-conveyor" => BridgeBlock::new(1, false, cost!(Copper: 6, Lead: 6), 4, true);
    "phase-conveyor" => BridgeBlock::new(1, false, cost!(Lead: 10, Graphite: 10, Silicon: 7, PhaseFabric: 5), 12, true);
    "sorter" => ItemBlock::new(1, true, cost!(Copper: 2, Lead: 2));
    "inverted-sorter" => ItemBlock::new(1, true, cost!(Copper: 2, Lead: 2));
    "router" => SimpleBlock::new(1, true, cost!(Copper: 3));
    "distributor" => SimpleBlock::new(2, true, cost!(Copper: 4, Lead: 4));
    "overflow-gate" => SimpleBlock::new(1, true, cost!(Copper: 4, Lead: 2));
    "underflow-gate" => SimpleBlock::new(1, true, cost!(Copper: 4, Lead: 2));
    "mass-driver" => BridgeBlock::new(3, true, cost!(Lead: 125, Titanium: 125, Thorium: 50, Silicon: 75), 55, false);
    "duct" => SimpleBlock::new(1, false, cost!(Beryllium: 1));
    "armored-duct" => SimpleBlock::new(1, false, cost!(Beryllium: 2, Tungsten: 1));
    "duct-router" => ItemBlock::new(1, true, cost!(Beryllium: 10));
    "overflow-duct" => SimpleBlock::new(1, true, cost!(Graphite: 8, Beryllium: 8));
    "underflow-duct" => SimpleBlock::new(1, true, cost!(Graphite: 8, Beryllium: 8));
    "duct-bridge" => BridgeBlock::new(1, true, cost!(Beryllium: 20), 3, true);
    "duct-unloader" => ItemBlock::new(1, true, cost!(Graphite: 20, Silicon: 20, Tungsten: 10));
    "surge-conveyor" => SimpleBlock::new(1, false, cost!(SurgeAlloy: 1, Tungsten: 1));
    "surge-router" => SimpleBlock::new(1, false, cost!(SurgeAlloy: 5, Tungsten: 1)); // not symmetric
    "unit-cargo-loader" => SimpleBlock::new(3, true, cost!(Silicon: 80, SurgeAlloy: 50, Oxide: 20));
    "unit-cargo-unload-point" => ItemBlock::new(2, true, cost!(Silicon: 60, Tungsten: 60));
    // sandbox only
    "item-source" => ItemBlock::new(1, true, &[]);
    "item-void" => SimpleBlock::new(1, true, &[]);
}

pub struct ItemBlock {
    size: u8,
    symmetric: bool,
    build_cost: BuildCost,
}

impl ItemBlock {
    #[must_use]
    pub const fn new(size: u8, symmetric: bool, build_cost: BuildCost) -> Self {
        assert!(size != 0, "invalid size");
        Self {
            size,
            symmetric,
            build_cost,
        }
    }

    state_impl!(pub Option<item::Type>);
}

impl BlockLogic for ItemBlock {
    impl_block!();

    fn data_from_i32(&self, config: i32, _: GridPos) -> Result<DynData, DataConvertError> {
        if config < 0 || config > i32::from(u16::MAX) {
            return Err(DataConvertError::Custom(Box::new(ItemConvertError(config))));
        }
        Ok(DynData::Content(content::Type::Item, config as u16))
    }

    fn deserialize_state(&self, data: DynData) -> Result<Option<Box<dyn Any>>, DeserializeError> {
        match data {
            DynData::Empty => Ok(Some(Self::create_state(None))),
            DynData::Content(content::Type::Item, id) => Ok(Some(Self::create_state(Some(
                ItemDeserializeError::forward(item::Type::try_from(id))?,
            )))),
            DynData::Content(have, ..) => Err(DeserializeError::Custom(Box::new(
                ItemDeserializeError::ContentType(have),
            ))),
            _ => Err(DeserializeError::InvalidType {
                have: data.get_type(),
                expect: DynType::Content,
            }),
        }
    }

    fn clone_state(&self, state: &dyn Any) -> Box<dyn Any> {
        let state = Self::get_state(state);
        Box::new(Self::create_state(*state))
    }

    fn mirror_state(&self, _: &mut dyn Any, _: bool, _: bool) {}

    fn rotate_state(&self, _: &mut dyn Any, _: bool) {}

    fn serialize_state(&self, state: &dyn Any) -> Result<DynData, SerializeError> {
        match Self::get_state(state) {
            None => Ok(DynData::Empty),
            Some(item) => Ok(DynData::Content(content::Type::Item, (*item).into())),
        }
    }

    fn draw(&self, category: &str, name: &str, state: Option<&dyn Any>) -> Option<RgbaImage> {
        let mut p = load(category, name).unwrap();
        if let Some(state) = state {
            if let Some(s) = Self::get_state(state) {
                let item_c = s.color();
                let [tr, tg, tb] = [
                    item_c[0] as f32 / 255.0,
                    item_c[1] as f32 / 255.0,
                    item_c[2] as f32 / 255.0,
                ];
                let mut top = load(category, "center").unwrap();
                for Rgba([r, g, b, ref a]) in top.pixels_mut() {
                    if a > &254 {
                        *r = (*r as f32 * tr) as u8;
                        *g = (*g as f32 * tg) as u8;
                        *b = (*b as f32 * tb) as u8;
                    }
                }

                image::imageops::overlay(&mut p, &top, 0, 0);
                return Some(p);
            }
        }
        if name == "unloader" {
            return Some(p);
        }
        let mut null = load("distribution", "cross-full").unwrap();
        image::imageops::overlay(&mut null, &p, 0, 0);
        Some(null)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ItemConvertError(pub i32);

impl fmt::Display for ItemConvertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid config ({}) for item", self.0)
    }
}

impl Error for ItemConvertError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ItemDeserializeError {
    ContentType(content::Type),
    NotFound(item::TryFromU16Error),
}

impl ItemDeserializeError {
    pub fn forward<T, E: Into<Self>>(result: Result<T, E>) -> Result<T, DeserializeError> {
        match result {
            Ok(v) => Ok(v),
            Err(e) => Err(DeserializeError::Custom(Box::new(e.into()))),
        }
    }
}

impl From<item::TryFromU16Error> for ItemDeserializeError {
    fn from(err: item::TryFromU16Error) -> Self {
        Self::NotFound(err)
    }
}

impl fmt::Display for ItemDeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ContentType(have) => write!(
                f,
                "expected content {:?} but got {have:?}",
                content::Type::Item
            ),
            Self::NotFound(..) => f.write_str("target item not found"),
        }
    }
}

impl Error for ItemDeserializeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::NotFound(e) => Some(e),
            _ => None,
        }
    }
}

pub struct BridgeBlock {
    size: u8,
    symmetric: bool,
    build_cost: BuildCost,
    range: u16,
    ortho: bool,
}

type Point2 = (i32, i32);

impl BridgeBlock {
    #[must_use]
    pub const fn new(
        size: u8,
        symmetric: bool,
        build_cost: BuildCost,
        range: u16,
        ortho: bool,
    ) -> Self {
        assert!(size != 0, "invalid size");
        assert!(range != 0, "invalid range");
        Self {
            size,
            symmetric,
            build_cost,
            range,
            ortho,
        }
    }

    state_impl!(pub Option<Point2>);
}

impl BlockLogic for BridgeBlock {
    impl_block!();

    fn data_from_i32(&self, config: i32, pos: GridPos) -> Result<DynData, DataConvertError> {
        let (x, y) = ((config >> 16) as i16, config as i16);
        if x < 0 || y < 0 {
            return Err(DataConvertError::Custom(Box::new(BridgeConvertError {
                x,
                y,
            })));
        }
        let dx = i32::from(x) - i32::from(pos.0);
        let dy = i32::from(y) - i32::from(pos.1);
        Ok(DynData::Point2(dx, dy))
    }

    fn deserialize_state(&self, data: DynData) -> Result<Option<Box<dyn Any>>, DeserializeError> {
        match data {
            DynData::Empty => Ok(Some(Self::create_state(None))),
            DynData::Point2(dx, dy) => {
                if self.ortho {
                    // the game uses (-worldX, -worldY) to indicate no target
                    // likely because the absolute target being (0, 0) means it's unlinked
                    if dx != 0 && dy != 0 {
                        return Ok(Some(Self::create_state(None)));
                    }
                    if dx > i32::from(self.range) || dx < -i32::from(self.range) {
                        return Ok(Some(Self::create_state(None)));
                    }
                }
                // can't check range otherwise, it depends on the target's size
                Ok(Some(Self::create_state(Some((dx, dy)))))
            }
            _ => Err(DeserializeError::InvalidType {
                have: data.get_type(),
                expect: DynType::Point2,
            }),
        }
    }

    fn clone_state(&self, state: &dyn Any) -> Box<dyn Any> {
        let state = Self::get_state(state);
        Box::new(Self::create_state(*state))
    }

    fn mirror_state(&self, state: &mut dyn Any, horizontally: bool, vertically: bool) {
        match Self::get_state_mut(state) {
            None => (),
            Some((dx, dy)) => {
                if horizontally {
                    *dx = -*dx;
                }
                if vertically {
                    *dy = -*dy;
                }
            }
        }
    }

    fn rotate_state(&self, state: &mut dyn Any, clockwise: bool) {
        match Self::get_state_mut(state) {
            None => (),
            Some((dx, dy)) => {
                let (cdx, cdy) = (*dx, *dy);
                *dx = if clockwise { cdy } else { -cdy };
                *dy = if clockwise { -cdx } else { cdx };
            }
        }
    }

    fn serialize_state(&self, state: &dyn Any) -> Result<DynData, SerializeError> {
        match Self::get_state(state) {
            None => Ok(DynData::Empty),
            Some((dx, dy)) => Ok(DynData::Point2(*dx, *dy)),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BridgeConvertError {
    pub x: i16,
    pub y: i16,
}

impl fmt::Display for BridgeConvertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid coordinate ({} / {}) for bridge", self.x, self.y)
    }
}

impl Error for BridgeConvertError {}
