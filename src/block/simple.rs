use std::any::{Any, type_name};

use crate::block::BlockLogic;
use crate::data::dynamic::DynData;

macro_rules!gen_state_empty
{
	() =>
	{
		fn data_from_i32(&self, _: i32) -> DynData
		{
			DynData::Empty
		}
		
		fn deserialize_state(&self, _: DynData) -> Option<Box<dyn Any>>
		{
			None
		}
		
		fn clone_state(&self, _: &dyn Any) -> Box<dyn Any>
		{
			panic!("{} has no custom state", type_name::<Self>())
		}
		
		fn serialize_state(&self, _: &dyn Any) -> DynData
		{
			DynData::Empty
		}
	};
}

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

pub struct SimpleBlock
{
	size: u8,
	symmetric: bool,
}

impl SimpleBlock
{
	pub const fn new(size: u8, symmetric: bool) -> Self
	{
		Self{size, symmetric}
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
	
	gen_state_empty!();
}
