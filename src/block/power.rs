//! power connection and generation
use std::any::Any;
use std::error::Error;
use std::fmt;

use crate::block::simple::{cost, state_impl, BuildCost, SimpleBlock};
use crate::block::{
    impl_block, make_register, BlockLogic, DataConvertError, DeserializeError, SerializeError,
};
use crate::data::dynamic::{DynData, DynType};
use crate::data::GridPos;
use crate::item::storage::Storage;

make_register! {
    // illuminator == power ?????
    "illuminator" => LampBlock::new(1, true, cost!(Lead: 8, Graphite: 12, Silicon: 8));
    "power-node" => ConnectorBlock::new(1, true, cost!(Copper: 1, Lead: 3), 10);
    "power-node-large" => ConnectorBlock::new(2, true, cost!(Lead: 10, Titanium: 5, Silicon: 3), 15);
    "surge-tower" => ConnectorBlock::new(2, true, cost!(Lead: 10, Titanium: 7, Silicon: 15, SurgeAlloy: 15), 2);
    "diode" => SimpleBlock::new(1, false, cost!(Metaglass: 10, Silicon: 10, Plastanium: 5));
    "battery" => SimpleBlock::new(1, true, cost!(Copper: 5, Lead: 20));
    "battery-large" => SimpleBlock::new(3, true, cost!(Lead: 50, Titanium: 20, Silicon: 30));
    "combustion-generator" => SimpleBlock::new(1, true, cost!(Copper: 25, Lead: 15));
    "thermal-generator" => SimpleBlock::new(2, true, cost!(Copper: 40, Lead: 50, Metaglass: 40, Graphite: 35, Silicon: 35));
    "steam-generator" => SimpleBlock::new(2, true, cost!(Copper: 35, Lead: 40, Graphite: 25, Silicon: 30));
    "differential-generator" => SimpleBlock::new(3, true, cost!(Copper: 70, Lead: 100, Metaglass: 50, Titanium: 50, Silicon: 65));
    "rtg-generator" => SimpleBlock::new(2, true, cost!(Lead: 100, Thorium: 50, Silicon: 75, Plastanium: 75, PhaseFabric: 25));
    "solar-panel" => SimpleBlock::new(1, true, cost!(Lead: 10, Silicon: 15));
    "solar-panel-large" => SimpleBlock::new(3, true, cost!(Lead: 80, Silicon: 110, PhaseFabric: 15));
    "thorium-reactor" => SimpleBlock::new(3, true, cost!(Lead: 300, Metaglass: 50, Graphite: 150, Thorium: 150, Silicon: 200));
    "impact-reactor" => SimpleBlock::new(4, true,
        cost!(Lead: 500, Metaglass: 250, Graphite: 400, Thorium: 100, Silicon: 300, SurgeAlloy: 250));
    "beam-node" => ConnectorBlock::new(1, true, cost!(Beryllium: 8), 4);
    "beam-tower" => ConnectorBlock::new(3, true, cost!(Beryllium: 30, Oxide: 10, Silicon: 10), 12);
    "turbine-condenser" => SimpleBlock::new(3, true, cost!(Beryllium: 60));
    "chemical-combustion-chamber" => SimpleBlock::new(3, true, cost!(Graphite: 40, Tungsten: 40, Oxide: 40, Silicon: 30));
    "pyrolosis-generator" => SimpleBlock::new(3, true, cost!(Graphite: 50, Carbide: 50, Oxide: 60, Silicon: 50));
    "flux-reactor" => SimpleBlock::new(5, true, cost!(Graphite: 300, Carbide: 200, Oxide: 100, Silicon: 600, SurgeAlloy: 300));
    "neoplasia-reactor" => SimpleBlock::new(5, true, cost!(Tungsten: 1000, Carbide: 300, Oxide: 150, Silicon: 500, PhaseFabric: 300, SurgeAlloy: 200));
    // editor only
    "beam-link" => ConnectorBlock::new(3, true, &[], 12);
    // sandbox only
    "power-source" => ConnectorBlock::new(1, true, &[], 100);
    "power-void" => SimpleBlock::new(1, true, &[]);
}
pub struct ConnectorBlock {
    size: u8,
    symmetric: bool,
    build_cost: BuildCost,
    max: u8,
}

impl ConnectorBlock {
    #[must_use]
    pub const fn new(size: u8, symmetric: bool, build_cost: BuildCost, max: u8) -> Self {
        assert!(size != 0, "invalid size");
        assert!(
            !(max == 0 || max > i8::MAX as u8),
            "invalid maximum link count"
        );
        Self {
            size,
            symmetric,
            build_cost,
            max,
        }
    }

    #[must_use]
    pub fn get_max_links(&self) -> u8 {
        self.max
    }

    state_impl!(pub Vec<(i16, i16)>);
}

impl BlockLogic for ConnectorBlock {
    impl_block!();

    fn data_from_i32(&self, _: i32, _: GridPos) -> Result<DynData, DataConvertError> {
        Ok(DynData::Empty)
    }

    fn deserialize_state(&self, data: DynData) -> Result<Option<Box<dyn Any>>, DeserializeError> {
        match data {
            DynData::Empty => Ok(Some(Self::create_state(Vec::new()))),
            DynData::Point2Array(s) => Ok(Some(Self::create_state(s))),
            _ => Err(DeserializeError::InvalidType {
                have: data.get_type(),
                expect: DynType::Boolean,
            }),
        }
    }

    fn clone_state(&self, state: &dyn Any) -> Box<dyn Any> {
        Box::new(Self::get_state(state).clone())
    }

    fn mirror_state(&self, state: &mut dyn Any, horizontally: bool, vertically: bool) {
        for (dx, dy) in Self::get_state_mut(state).iter_mut() {
            if horizontally {
                *dx = -*dx;
            }
            if vertically {
                *dy = -*dy;
            }
        }
    }

    fn rotate_state(&self, state: &mut dyn Any, clockwise: bool) {
        for (dx, dy) in Self::get_state_mut(state).iter_mut() {
            let (cdx, cdy) = (*dx, *dy);
            *dx = if clockwise { cdy } else { -cdy };
            *dy = if clockwise { -cdx } else { cdx };
        }
    }

    fn serialize_state(&self, state: &dyn Any) -> Result<DynData, SerializeError> {
        Ok(DynData::Point2Array(Self::get_state(state).clone()))
    }
}

#[derive(Debug)]
pub enum ConnectorDeserializeError {
    LinkCount { have: usize, max: u8 },
}

impl ConnectorDeserializeError {
    pub fn forward<T, E: Into<Self>>(result: Result<T, E>) -> Result<T, DeserializeError> {
        match result {
            Ok(v) => Ok(v),
            Err(e) => Err(DeserializeError::Custom(Box::new(e.into()))),
        }
    }
}

impl fmt::Display for ConnectorDeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LinkCount { have, max } => {
                write!(f, "too many links ({have} but only {max} supported)")
            }
        }
    }
}

impl Error for ConnectorDeserializeError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RGBA(u8, u8, u8, u8);

impl From<u32> for RGBA {
    fn from(value: u32) -> Self {
        Self(
            (value >> 24) as u8,
            (value >> 16) as u8,
            (value >> 8) as u8,
            value as u8,
        )
    }
}

impl From<RGBA> for u32 {
    fn from(value: RGBA) -> Self {
        (u32::from(value.0) << 24)
            | (u32::from(value.1) << 16)
            | (u32::from(value.2) << 8)
            | u32::from(value.3)
    }
}

pub struct LampBlock {
    size: u8,
    symmetric: bool,
    build_cost: BuildCost,
}

impl LampBlock {
    #[must_use]
    pub const fn new(size: u8, symmetric: bool, build_cost: BuildCost) -> Self {
        assert!(size != 0, "invalid size");
        Self {
            size,
            symmetric,
            build_cost,
        }
    }

    state_impl!(pub RGBA);
}

impl BlockLogic for LampBlock {
    impl_block!();

    fn data_from_i32(&self, config: i32, _: GridPos) -> Result<DynData, DataConvertError> {
        Ok(DynData::Int(config))
    }

    fn deserialize_state(&self, data: DynData) -> Result<Option<Box<dyn Any>>, DeserializeError> {
        match data {
            DynData::Int(rgba) => Ok(Some(Self::create_state(RGBA::from(rgba as u32)))),
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
        let state = Self::get_state(state);
        Ok(DynData::Int(u32::from(*state) as i32))
    }
}
