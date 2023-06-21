use std::any::Any;

use crate::block::simple::{cost, state_impl, BuildCost, SimpleBlock};
use crate::block::{
    impl_block, make_register, BlockLogic, DataConvertError, DeserializeError, SerializeError,
};
use crate::data::dynamic::{DynData, DynType};
use crate::data::GridPos;
use crate::item::storage::Storage;

make_register!
(
    "copper-wall" => SimpleBlock::new(1, true, cost!(Copper: 6));
    "copper-wall-large" => SimpleBlock::new(2, true, cost!(Copper: 24));
    "titanium-wall" => SimpleBlock::new(1, true, cost!(Titanium: 6));
    "titanium-wall-large" => SimpleBlock::new(2, true, cost!(Titanium: 24));
    "plastanium-wall" => SimpleBlock::new(1, true, cost!(Metaglass: 2, Plastanium: 5));
    "plastanium-wall-large" => SimpleBlock::new(2, true, cost!(Metaglass: 8, Plastanium: 20));
    "thorium-wall" => SimpleBlock::new(1, true, cost!(Thorium: 6));
    "thorium-wall-large" => SimpleBlock::new(2, true, cost!(Thorium: 24));
    "phase-wall" => SimpleBlock::new(1, true, cost!(PhaseFabric: 6));
    "phase-wall-large" => SimpleBlock::new(2, true, cost!(PhaseFabric: 24));
    "surge-wall" => SimpleBlock::new(1, true, cost!(SurgeAlloy: 6));
    "surge-wall-large" => SimpleBlock::new(2, true, cost!(SurgeAlloy: 24));
    "door" => DoorBlock::new(1, true, cost!(Titanium: 6, Silicon: 4));
    "door-large" => DoorBlock::new(2, true, cost!(Titanium: 24, Silicon: 16));
    // sandbox only
    "scrap-wall" => SimpleBlock::new(1, true, cost!(Scrap: 6));
    "scrap-wall-large" => SimpleBlock::new(2, true, cost!(Scrap: 24));
    "scrap-wall-huge" => SimpleBlock::new(3, true, cost!(Scrap: 54));
    "scrap-wall-gigantic" => SimpleBlock::new(4, true, cost!(Scrap: 96));
    "thruster" => SimpleBlock::new(4, false, cost!(Scrap: 96));
);

pub struct DoorBlock {
    size: u8,
    symmetric: bool,
    build_cost: BuildCost,
}

impl DoorBlock {
    #[must_use]
    pub const fn new(size: u8, symmetric: bool, build_cost: BuildCost) -> Self {
        assert!(size != 0, "invalid size");
        Self {
            size,
            symmetric,
            build_cost,
        }
    }

    state_impl!(pub bool);
}

impl BlockLogic for DoorBlock {
    impl_block!();

    fn data_from_i32(&self, _: i32, _: GridPos) -> Result<DynData, DataConvertError> {
        Ok(DynData::Boolean(false))
    }

    fn deserialize_state(&self, data: DynData) -> Result<Option<Box<dyn Any>>, DeserializeError> {
        match data {
            DynData::Boolean(opened) => Ok(Some(Self::create_state(opened))),
            _ => Err(DeserializeError::InvalidType {
                have: data.get_type(),
                expect: DynType::Boolean,
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
        Ok(DynData::Boolean(*state))
    }
}
