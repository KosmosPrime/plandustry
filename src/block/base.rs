use std::any::Any;

use crate::block::simple::{cost, state_impl, BuildCost, SimpleBlock};
use crate::block::transport::ItemBlock;
use crate::block::{make_register, BlockLogic, DataConvertError, DeserializeError, SerializeError};
use crate::data::dynamic::{DynData, DynType};
use crate::data::GridPos;
use crate::item::storage::Storage;

make_register! {
    "mender" => SimpleBlock::new(1, true, cost!(Copper: 25, Lead: 30));
    "mend-projector" => SimpleBlock::new(2, true, cost!(Copper: 50, Lead: 100, Titanium: 25, Silicon: 40));
    "overdrive-projector" => SimpleBlock::new(2, true, cost!(Lead: 100, Titanium: 75, Silicon: 75, Plastanium: 30));
    "overdrive-dome" => SimpleBlock::new(3, true, cost!(Lead: 200, Titanium: 130, Silicon: 130, Plastanium: 80, SurgeAlloy: 120));
    "force-projector" => SimpleBlock::new(3, true, cost!(Lead: 100, Titanium: 75, Silicon: 125));
    "shock-mine" => SimpleBlock::new(1, true, cost!(Lead: 25, Silicon: 12));
    "core-shard" => SimpleBlock::new(3, true, cost!(Copper: 1000, Lead: 800));
    "core-foundation" => SimpleBlock::new(4, true, cost!(Copper: 3000, Lead: 3000, Silicon: 2000));
    "core-nucleus" => SimpleBlock::new(5, true, cost!(Copper: 8000, Lead: 8000, Thorium: 4000, Silicon: 5000));
    "container" => SimpleBlock::new(2, true, cost!(Titanium: 100));
    "vault" => SimpleBlock::new(3, true, cost!(Titanium: 250, Thorium: 125));
    "unloader" => ItemBlock::new(1, true, cost!(Titanium: 25, Silicon: 30));
    "illuminator" => LampBlock::new(1, true, cost!(Lead: 8, Graphite: 12, Silicon: 8));
    "launch-pad" => SimpleBlock::new(3, true, cost!(Copper: 350, Lead: 200, Titanium: 150, Silicon: 140));
}

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
        ((value.0 as u32) << 24)
            | ((value.1 as u32) << 16)
            | ((value.2 as u32) << 8)
            | (value.3 as u32)
    }
}

pub struct LampBlock {
    size: u8,
    symmetric: bool,
    build_cost: BuildCost,
}

impl LampBlock {
    pub const fn new(size: u8, symmetric: bool, build_cost: BuildCost) -> Self {
        if size == 0 {
            panic!("invalid size");
        }
        Self {
            size,
            symmetric,
            build_cost,
        }
    }

    state_impl!(pub RGBA);
}

impl BlockLogic for LampBlock {
    fn get_size(&self) -> u8 {
        self.size
    }

    fn is_symmetric(&self) -> bool {
        self.symmetric
    }

    fn create_build_cost(&self) -> Option<Storage> {
        if !self.build_cost.is_empty() {
            let mut storage = Storage::new();
            for (ty, cnt) in self.build_cost {
                storage.add(*ty, *cnt, u32::MAX);
            }
            Some(storage)
        } else {
            None
        }
    }

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
