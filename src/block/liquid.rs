//! liquid related things
use std::any::Any;
use std::error::Error;
use std::fmt;

use crate::block::distribution::BridgeBlock;
use crate::block::simple::{cost, state_impl, BuildCost, SimpleBlock};
use crate::block::{
    impl_block, make_register, BlockLogic, DataConvertError, DeserializeError, SerializeError,
};
use crate::content;
use crate::data::dynamic::{DynData, DynType};
use crate::data::GridPos;
use crate::fluid;
use crate::item::storage::Storage;

make_register! {
    "reinforced-pump" => SimpleBlock::new(2, true, cost!(Beryllium: 40, Tungsten: 30, Silicon: 20));
    "mechanical-pump" => SimpleBlock::new(1, true, cost!(Copper: 15, Metaglass: 10));
    "rotary-pump" => SimpleBlock::new(2, true, cost!(Copper: 70, Metaglass: 50, Titanium: 35, Silicon: 20));
    "impulse-pump" => SimpleBlock::new(3, true, cost!(Copper: 80, Metaglass: 90, Titanium: 40, Thorium: 35, Silicon: 30));
    "conduit" => SimpleBlock::new(1, false, cost!(Metaglass: 1));
    "pulse-conduit" => SimpleBlock::new(1, false, cost!(Metaglass: 1, Titanium: 2));
    "plated-conduit" => SimpleBlock::new(1, false, cost!(Metaglass: 1, Thorium: 2, Plastanium: 1));
    "liquid-router" => SimpleBlock::new(1, true, cost!(Metaglass: 2, Graphite: 4));
    "liquid-container" => SimpleBlock::new(2, true, cost!(Metaglass: 15, Titanium: 10));
    "liquid-tank" => SimpleBlock::new(3, true, cost!(Metaglass: 40, Titanium: 30));
    "liquid-junction" => SimpleBlock::new(1, true, cost!(Metaglass: 8, Graphite: 4));
    "bridge-conduit" => BridgeBlock::new(1, true, cost!(Metaglass: 8, Graphite: 4), 4, true);
    "phase-conduit" => BridgeBlock::new(1, true, cost!(Metaglass: 20, Titanium: 10, Silicon: 7, PhaseFabric: 5), 12, true);
    "reinforced-conduit" => SimpleBlock::new(1, false, cost!(Beryllium: 2));
    "reinforced-liquid-junction" => SimpleBlock::new(1, true, cost!(Graphite: 4, Beryllium: 8));
    "reinforced-bridge-conduit" => BridgeBlock::new(1, true, cost!(Graphite: 8, Beryllium: 20), 4, true);
    "reinforced-liquid-router" => SimpleBlock::new(1, true, cost!(Graphite: 8, Beryllium: 4));
    "reinforced-liquid-container" => SimpleBlock::new(2, true, cost!(Tungsten: 10, Beryllium: 16));
    "reinforced-liquid-tank" => SimpleBlock::new(3, true, cost!(Tungsten: 40, Beryllium: 50));
    // sandbox only
    "liquid-source" => FluidBlock::new(1, true, &[]);
    "liquid-void" => SimpleBlock::new(1, true, &[]);
}

pub struct FluidBlock {
    size: u8,
    symmetric: bool,
    build_cost: BuildCost,
}

impl FluidBlock {
    #[must_use]
    pub const fn new(size: u8, symmetric: bool, build_cost: BuildCost) -> Self {
        assert!(size != 0, "invalid size");
        Self {
            size,
            symmetric,
            build_cost,
        }
    }

    state_impl!(pub Option<fluid::Type>);
}

impl BlockLogic for FluidBlock {
    impl_block!();

    fn data_from_i32(&self, config: i32, _: GridPos) -> Result<DynData, DataConvertError> {
        if config < 0 || config > i32::from(u16::MAX) {
            return Err(DataConvertError::Custom(Box::new(FluidConvertError(
                config,
            ))));
        }
        Ok(DynData::Content(content::Type::Fluid, config as u16))
    }

    fn deserialize_state(&self, data: DynData) -> Result<Option<Box<dyn Any>>, DeserializeError> {
        match data {
            DynData::Empty => Ok(Some(Self::create_state(None))),
            DynData::Content(content::Type::Fluid, id) => Ok(Some(Self::create_state(Some(
                FluidDeserializeError::forward(fluid::Type::try_from(id))?,
            )))),
            DynData::Content(have, ..) => Err(DeserializeError::Custom(Box::new(
                FluidDeserializeError::ContentType(have),
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
            Some(fluid) => Ok(DynData::Content(content::Type::Fluid, (*fluid).into())),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FluidConvertError(pub i32);

impl fmt::Display for FluidConvertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid config ({}) for fluid", self.0)
    }
}

impl Error for FluidConvertError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FluidDeserializeError {
    ContentType(content::Type),
    NotFound(fluid::TryFromU16Error),
}

impl FluidDeserializeError {
    pub fn forward<T, E: Into<Self>>(result: Result<T, E>) -> Result<T, DeserializeError> {
        match result {
            Ok(v) => Ok(v),
            Err(e) => Err(DeserializeError::Custom(Box::new(e.into()))),
        }
    }
}

impl From<fluid::TryFromU16Error> for FluidDeserializeError {
    fn from(err: fluid::TryFromU16Error) -> Self {
        Self::NotFound(err)
    }
}

impl fmt::Display for FluidDeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ContentType(have) => write!(
                f,
                "expected content {:?} but got {have:?}",
                content::Type::Fluid
            ),
            Self::NotFound(..) => f.write_str("fluid not found"),
        }
    }
}

impl Error for FluidDeserializeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::NotFound(e) => Some(e),
            _ => None,
        }
    }
}
