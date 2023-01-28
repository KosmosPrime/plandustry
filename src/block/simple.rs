use std::any::{Any, type_name};

use crate::block::{BlockLogic, DataConvertError, DeserializeError, SerializeError};
use crate::data::GridPos;
use crate::data::dynamic::DynData;
use crate::item;
use crate::item::storage::Storage;

macro_rules!state_impl
{
	($vis:vis $type:ty) =>
	{
		$vis fn get_state<'l>(state: &'l dyn Any) -> &'l $type
			where Self: Sized
		{
			state.downcast_ref::<$type>().unwrap()
		}
		
		$vis fn get_state_mut<'l>(state: &'l mut dyn Any) -> &'l mut $type
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

pub struct SimpleBlock
{
	size: u8,
	symmetric: bool,
	build_cost: BuildCost,
}

impl SimpleBlock
{
	pub const fn new(size: u8, symmetric: bool, build_cost: BuildCost) -> Self
	{
		if size == 0
		{
			panic!("invalid size");
		}
		Self{size, symmetric, build_cost}
	}
}

impl BlockLogic for SimpleBlock
{
	fn get_size(&self) -> u8
	{
		self.size
	}
	
	fn is_symmetric(&self) -> bool
	{
		self.symmetric
	}
	
	fn create_build_cost(&self) -> Option<Storage>
	{
		if !self.build_cost.is_empty()
		{
			let mut storage = Storage::new();
			for (ty, cnt) in self.build_cost
			{
				storage.add(*ty, *cnt, u32::MAX);
			}
			Some(storage)
		}
		else {None}
	}
	
	fn data_from_i32(&self, _: i32, _: GridPos) -> Result<DynData, DataConvertError>
	{
		Ok(DynData::Empty)
	}
	
	fn deserialize_state(&self, _: DynData) -> Result<Option<Box<dyn Any>>, DeserializeError>
	{
		Ok(None)
	}
	
	fn clone_state(&self, _: &dyn Any) -> Box<dyn Any>
	{
		panic!("{} has no custom state", type_name::<Self>())
	}
	
	fn mirror_state(&self, _: &mut dyn Any, _: bool, _: bool)
	{
		panic!("{} has no custom state", type_name::<Self>());
	}
	
	fn rotate_state(&self, _: &mut dyn Any, _: bool)
	{
		panic!("{} has no custom state", type_name::<Self>());
	}
	
	fn serialize_state(&self, _: &dyn Any) -> Result<DynData, SerializeError>
	{
		Ok(DynData::Empty)
	}
}

macro_rules!cost
{
	($($item:ident: $cnt:literal),+) =>
	{
		&[$((crate::item::Type::$item, $cnt)),*]
	};
}
pub(crate) use cost;
