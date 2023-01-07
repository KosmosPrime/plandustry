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
