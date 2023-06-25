//! payload related bits and bobs
use std::any::Any;
use std::error::Error;
use std::fmt;

use crate::block::simple::{cost, state_impl, BuildCost, SimpleBlock};
use crate::block::{
    self, distribution::BridgeBlock, impl_block, make_register, BlockLogic, DataConvertError,
    DeserializeError, SerializeError,
};
use crate::content;
use crate::data::dynamic::{DynData, DynType};
use crate::data::GridPos;
use crate::item::storage::Storage;
use crate::unit;

const GROUND_UNITS: &[unit::Type] = &[unit::Type::Dagger, unit::Type::Crawler, unit::Type::Nova];
const AIR_UNITS: &[unit::Type] = &[unit::Type::Flare, unit::Type::Mono];
const NAVAL_UNITS: &[unit::Type] = &[unit::Type::Risso, unit::Type::Retusa];

make_register! {
    "ground-factory" => AssemblerBlock::new(3, false, cost!(Copper: 50, Lead: 120, Silicon: 80), GROUND_UNITS);
    "air-factory" => AssemblerBlock::new(3, false, cost!(Copper: 60, Lead: 70), AIR_UNITS);
    "naval-factory" => AssemblerBlock::new(3, false, cost!(Copper: 150, Lead: 130, Metaglass: 120), NAVAL_UNITS);
    "additive-reconstructor" => SimpleBlock::new(3, false, cost!(Copper: 200, Lead: 120, Silicon: 90));
    "multiplicative-reconstructor" => SimpleBlock::new(5, false, cost!(Lead: 650, Titanium: 350, Thorium: 650, Silicon: 450));
    "exponential-reconstructor" => SimpleBlock::new(7, false,
        cost!(Lead: 2000, Titanium: 2000, Thorium: 750, Silicon: 1000, Plastanium: 450, PhaseFabric: 600));
    "tetrative-reconstructor" => SimpleBlock::new(9, false,
        cost!(Lead: 4000, Thorium: 1000, Silicon: 3000, Plastanium: 600, PhaseFabric: 600, SurgeAlloy: 800));
    "repair-point" => SimpleBlock::new(1, true, cost!(Copper: 30, Lead: 30, Silicon: 20));
    "repair-turret" => SimpleBlock::new(2, true, cost!(Thorium: 80, Silicon: 90, Plastanium: 60));
    "tank-fabricator" => SimpleBlock::new(3, true, cost!(Silicon: 200, Beryllium: 150));
    "ship-fabricator" => SimpleBlock::new(3, true, cost!(Silicon: 250, Beryllium: 200));
    "mech-fabricator" => SimpleBlock::new(3, true, cost!(Silicon: 200, Graphite: 300, Tungsten: 60));
    "tank-refabricator" => SimpleBlock::new(3, true, cost!(Beryllium: 200, Tungsten: 80, Silicon: 100));
    "mech-refabricator" => SimpleBlock::new(3, true, cost!(Beryllium: 250, Tungsten: 120, Silicon: 150));
    "ship-refabricator" => SimpleBlock::new(3, true, cost!(Beryllium: 200, Tungsten: 100, Silicon: 150, Oxide: 40));
    "prime-refabricator" => SimpleBlock::new(5, true, cost!(Thorium: 250, Oxide: 200, Tungsten: 200, Silicon: 400));
    "tank-assembler" => SimpleBlock::new(5, true, cost!(Thorium: 500, Oxide: 150, Carbide: 80, Silicon: 500));
    "ship-assembler" => SimpleBlock::new(5, true, cost!(Carbide: 100, Oxide: 200, Tungsten: 500, Silicon: 800, Thorium: 400));
    "mech-assembler" => SimpleBlock::new(5, true, cost!(Carbide: 200, Thorium: 600, Oxide: 200, Tungsten: 500, Silicon: 900)); // smh collaris
    "basic-assembler-module" => SimpleBlock::new(5, true, cost!(Carbide: 300, Thorium: 500, Oxide: 200, PhaseFabric: 400)); // the dummy block
    // payload
    "payload-conveyor" => SimpleBlock::new(3, false, cost!(Copper: 10, Graphite: 10));
    "payload-router" => PayloadBlock::new(3, false, cost!(Copper: 10, Graphite: 15));
    "reinforced-payload-conveyor" => SimpleBlock::new(3, false, cost!(Tungsten: 10));
    "reinforced-payload-router" => SimpleBlock::new(3, false, cost!(Tungsten: 15));
    "payload-mass-driver" => BridgeBlock::new(3, true, cost!(Tungsten: 120, Silicon: 120, Graphite: 50), 700, false);
    "large-payload-mass-driver" => BridgeBlock::new(5, true, cost!(Thorium: 200, Tungsten: 200, Silicon: 200, Graphite: 100, Oxide: 30), 1100, false);
    "small-deconstructor" => SimpleBlock::new(3, true, cost!(Beryllium: 100, Silicon: 100, Oxide: 40, Graphite: 80));
    "deconstructor" => SimpleBlock::new(5, true, cost!(Beryllium: 250, Oxide: 100, Silicon: 250, Carbide: 250));
    "constructor" => PayloadBlock::new(3, true, cost!(Silicon: 100, Beryllium: 150, Tungsten: 80));
    "large-constructor" => PayloadBlock::new(3, true, cost!(Silicon: 150, Oxide: 150, Tungsten: 200, PhaseFabric: 40));
    "payload-loader" => SimpleBlock::new(3, false, cost!(Graphite: 50, Silicon: 50, Tungsten: 80));
    "payload-unloader" => SimpleBlock::new(3, false, cost!(Graphite: 50, Silicon: 50, Tungsten: 30));
    // sandbox only
    "payload-source" => PayloadBlock::new(5, false, &[]);
    "payload-void" => SimpleBlock::new(5, true, &[]);
}

pub struct AssemblerBlock {
    size: u8,
    symmetric: bool,
    build_cost: BuildCost,
    valid: &'static [unit::Type],
}

impl AssemblerBlock {
    #[must_use]
    pub const fn new(
        size: u8,
        symmetric: bool,
        build_cost: BuildCost,
        valid: &'static [unit::Type],
    ) -> Self {
        assert!(size != 0, "invalid size");
        assert!(!valid.is_empty(), "no valid units");
        assert!(valid.len() <= i32::MAX as usize, "too many valid units");
        Self {
            size,
            symmetric,
            build_cost,
            valid,
        }
    }

    state_impl!(pub Option<unit::Type>);
}

impl BlockLogic for AssemblerBlock {
    impl_block!();

    fn data_from_i32(&self, _: i32, _: GridPos) -> Result<DynData, DataConvertError> {
        Ok(DynData::Int(-1))
    }

    fn deserialize_state(&self, data: DynData) -> Result<Option<Box<dyn Any>>, DeserializeError> {
        match data {
            DynData::Empty => Ok(Some(Self::create_state(None))),
            DynData::Int(idx) => {
                if idx == -1 {
                    Ok(Some(Self::create_state(None)))
                } else if idx >= 0 && idx < self.valid.len() as i32 {
                    Ok(Some(Self::create_state(Some(self.valid[idx as usize]))))
                } else {
                    Err(DeserializeError::Custom(Box::new(
                        AssemblerDeserializeError {
                            idx,
                            count: self.valid.len() as i32,
                        },
                    )))
                }
            }
            _ => Err(DeserializeError::InvalidType {
                have: data.get_type(),
                expect: DynType::Int,
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
        if let Some(state) = Self::get_state(state) {
            for (i, curr) in self.valid.iter().enumerate() {
                if curr == state {
                    return Ok(DynData::Int(i as i32));
                }
            }
            Err(SerializeError::Custom(Box::new(AssemblerSerializeError(
                *state,
            ))))
        } else {
            Ok(DynData::Int(-1))
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AssemblerDeserializeError {
    pub idx: i32,
    pub count: i32,
}

impl fmt::Display for AssemblerDeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid unit index ({}, #valid: {})",
            self.idx, self.count
        )
    }
}

impl Error for AssemblerDeserializeError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AssemblerSerializeError(unit::Type);

impl fmt::Display for AssemblerSerializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid unit ({:?}) is not valid", self.0)
    }
}

impl Error for AssemblerSerializeError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Payload {
    Empty,
    Block(block::content::Type),
    Unit(unit::Type),
}

pub struct PayloadBlock {
    size: u8,
    symmetric: bool,
    build_cost: BuildCost,
}

impl PayloadBlock {
    #[must_use]
    pub const fn new(size: u8, symmetric: bool, build_cost: BuildCost) -> Self {
        assert!(size != 0, "invalid size");
        Self {
            size,
            symmetric,
            build_cost,
        }
    }

    state_impl!(pub Payload);
}

impl BlockLogic for PayloadBlock {
    impl_block!();

    fn data_from_i32(&self, _: i32, _: GridPos) -> Result<DynData, DataConvertError> {
        Ok(DynData::Empty)
    }

    fn deserialize_state(&self, data: DynData) -> Result<Option<Box<dyn Any>>, DeserializeError> {
        match data {
            DynData::Empty => Ok(Some(Self::create_state(Payload::Empty))),
            DynData::Content(content::Type::Block, id) => {
                let block = PayloadDeserializeError::forward(block::content::Type::try_from(id))?;
                Ok(Some(Self::create_state(Payload::Block(block))))
            }
            DynData::Content(content::Type::Unit, id) => {
                let unit = PayloadDeserializeError::forward(unit::Type::try_from(id))?;
                Ok(Some(Self::create_state(Payload::Unit(unit))))
            }
            DynData::Content(have, ..) => Err(DeserializeError::Custom(Box::new(
                PayloadDeserializeError::ContentType(have),
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
            Payload::Empty => Ok(DynData::Empty),
            Payload::Block(block) => Ok(DynData::Content(content::Type::Block, (*block).into())),
            Payload::Unit(unit) => Ok(DynData::Content(content::Type::Unit, (*unit).into())),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PayloadDeserializeError {
    ContentType(content::Type),
    BlockNotFound(block::content::TryFromU16Error),
    UnitNotFound(unit::TryFromU16Error),
}

impl PayloadDeserializeError {
    pub fn forward<T, E: Into<Self>>(result: Result<T, E>) -> Result<T, DeserializeError> {
        match result {
            Ok(v) => Ok(v),
            Err(e) => Err(DeserializeError::Custom(Box::new(e.into()))),
        }
    }
}

impl From<block::content::TryFromU16Error> for PayloadDeserializeError {
    fn from(err: block::content::TryFromU16Error) -> Self {
        Self::BlockNotFound(err)
    }
}

impl From<unit::TryFromU16Error> for PayloadDeserializeError {
    fn from(err: unit::TryFromU16Error) -> Self {
        Self::UnitNotFound(err)
    }
}

impl fmt::Display for PayloadDeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ContentType(have) => write!(
                f,
                "expected content {:?} or {:?} but got {have:?}",
                content::Type::Block,
                content::Type::Unit
            ),
            Self::BlockNotFound(..) => f.write_str("payload block not found"),
            Self::UnitNotFound(..) => f.write_str("payload unit not found"),
        }
    }
}

impl Error for PayloadDeserializeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::BlockNotFound(e) => Some(e),
            Self::UnitNotFound(e) => Some(e),
            _ => None,
        }
    }
}
