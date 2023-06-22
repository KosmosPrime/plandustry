use std::any::{type_name, Any};

use crate::block::{impl_block, BlockLogic, DataConvertError, DeserializeError, SerializeError};
use crate::data::dynamic::DynData;
use crate::data::GridPos;
use crate::item;
use crate::item::storage::Storage;

macro_rules!state_impl
{
	($vis:vis $type:ty) =>
	{
		$vis fn get_state(state: &dyn Any) -> &$type
			where Self: Sized
		{
			state.downcast_ref::<$type>().unwrap()
		}

		$vis fn get_state_mut(state: &mut dyn Any) -> &mut $type
			where Self: Sized
		{
			state.downcast_mut::<$type>().unwrap()
		}

		fn create_state(val: $type) -> Box<dyn Any>
			where Self: Sized
		{
			Box::new(val)
		}
	};
}
pub(crate) use state_impl;

pub type BuildCost = &'static [(item::Type, u32)];

pub struct SimpleBlock {
    size: u8,
    symmetric: bool,
    build_cost: BuildCost,
}

impl SimpleBlock {
    #[must_use]
    pub const fn new(size: u8, symmetric: bool, build_cost: BuildCost) -> Self {
        assert!(size != 0, "invalid size");
        Self {
            size,
            symmetric,
            build_cost,
        }
    }
}

impl BlockLogic for SimpleBlock {
    impl_block!();

    fn data_from_i32(&self, _: i32, _: GridPos) -> Result<DynData, DataConvertError> {
        Ok(DynData::Empty)
    }

    fn deserialize_state(&self, _: DynData) -> Result<Option<Box<dyn Any>>, DeserializeError> {
        Ok(None)
    }

    fn clone_state(&self, _: &dyn Any) -> Box<dyn Any> {
        panic!("{} has no custom state", type_name::<Self>())
    }

    fn mirror_state(&self, _: &mut dyn Any, _: bool, _: bool) {
        panic!("{} has no custom state", type_name::<Self>());
    }

    fn rotate_state(&self, _: &mut dyn Any, _: bool) {
        panic!("{} has no custom state", type_name::<Self>());
    }

    fn serialize_state(&self, _: &dyn Any) -> Result<DynData, SerializeError> {
        Ok(DynData::Empty)
    }
}

macro_rules! cost {
	($($item:ident: $cnt:expr),+) => {
		&[$((crate::item::Type::$item, $cnt)),*]
	};
}
pub(crate) use cost;
