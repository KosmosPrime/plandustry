use std::any::Any;

use crate::block::{BlockLogic, DataConvertError, DeserializeError, make_register, SerializeError};
use crate::block::simple::{SimpleBlock, state_impl};
use crate::data::GridPos;
use crate::data::dynamic::{DynData, DynType};

make_register!
(
	COPPER_WALL: "copper-wall" => SimpleBlock::new(1, true);
	COPPER_WALL_LARGE: "copper-wall-large" => SimpleBlock::new(2, true);
	TITANIUM_WALL: "titanium-wall" => SimpleBlock::new(1, true);
	TITANIUM_WALL_LARGE: "titanium-wall-large" => SimpleBlock::new(2, true);
	PLASTANIUM_WALL: "plastanium-wall" => SimpleBlock::new(1, true);
	PLASTANIUM_WALL_LARGE: "plastanium-wall-large" => SimpleBlock::new(2, true);
	THORIUM_WALL: "thorium-wall" => SimpleBlock::new(1, true);
	THORIUM_WALL_LARGE: "thorium-wall-large" => SimpleBlock::new(2, true);
	PHASE_WALL: "phase-wall" => SimpleBlock::new(1, true);
	PHASE_WALL_LARGE: "phase-wall-large" => SimpleBlock::new(2, true);
	SURGE_WALL: "surge-wall" => SimpleBlock::new(1, true);
	SURGE_WALL_LARGE: "surge-wall-large" => SimpleBlock::new(2, true);
	DOOR: "door" => DoorBlock::new(1, true);
	DOOR_LARGE: "door-large" => DoorBlock::new(2, true);
	// sandbox only
	SCRAP_WALL: "scrap-wall" => SimpleBlock::new(1, true);
	SCRAP_WALL_LARGE: "scrap-wall-large" => SimpleBlock::new(2, true);
	SCRAP_WALL_HUGE: "scrap-wall-huge" => SimpleBlock::new(3, true);
	SCRAP_WALL_GIGANTIC: "scrap-wall-gigantic" => SimpleBlock::new(4, true);
	THRUSTER: "thruster" => SimpleBlock::new(4, false);
);

pub struct DoorBlock
{
	size: u8,
	symmetric: bool,
}

impl DoorBlock
{
	pub const fn new(size: u8, symmetric: bool) -> Self
	{
		if size == 0
		{
			panic!("invalid size");
		}
		Self{size, symmetric}
	}
	
	state_impl!(pub bool);
}

impl BlockLogic for DoorBlock
{
	fn get_size(&self) -> u8
	{
		self.size
	}
	
	fn is_symmetric(&self) -> bool
	{
		self.symmetric
	}
	
	fn data_from_i32(&self, _: i32, _: GridPos) -> Result<DynData, DataConvertError>
	{
		Ok(DynData::Boolean(false))
	}
	
	fn deserialize_state(&self, data: DynData) -> Result<Option<Box<dyn Any>>, DeserializeError>
	{
		match data
		{
			DynData::Boolean(opened) => Ok(Some(Self::create_state(opened))),
			_ => Err(DeserializeError::InvalidType{have: data.get_type(), expect: DynType::Boolean}),
		}
	}
	
	fn clone_state(&self, state: &dyn Any) -> Box<dyn Any>
	{
		let state = Self::get_state(state);
		Box::new(Self::create_state(*state))
	}
	
	fn serialize_state(&self, state: &dyn Any) -> Result<DynData, SerializeError>
	{
		let state = Self::get_state(state);
		Ok(DynData::Boolean(*state))
	}
}
