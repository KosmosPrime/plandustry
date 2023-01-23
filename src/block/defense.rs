use std::any::Any;

use crate::block::{BlockLogic, DataConvertError, DeserializeError, make_register, SerializeError};
use crate::block::simple::{BuildCost, cost, SimpleBlock, state_impl};
use crate::data::GridPos;
use crate::data::dynamic::{DynData, DynType};
use crate::item::storage::Storage;

make_register!
(
	COPPER_WALL: "copper-wall" => SimpleBlock::new(1, true, cost!(Copper: 6));
	COPPER_WALL_LARGE: "copper-wall-large" => SimpleBlock::new(2, true, cost!(Copper: 24));
	TITANIUM_WALL: "titanium-wall" => SimpleBlock::new(1, true, cost!(Titanium: 6));
	TITANIUM_WALL_LARGE: "titanium-wall-large" => SimpleBlock::new(2, true, cost!(Titanium: 24));
	PLASTANIUM_WALL: "plastanium-wall" => SimpleBlock::new(1, true, cost!(Metaglass: 2, Plastanium: 5));
	PLASTANIUM_WALL_LARGE: "plastanium-wall-large" => SimpleBlock::new(2, true, cost!(Metaglass: 8, Plastanium: 20));
	THORIUM_WALL: "thorium-wall" => SimpleBlock::new(1, true, cost!(Thorium: 6));
	THORIUM_WALL_LARGE: "thorium-wall-large" => SimpleBlock::new(2, true, cost!(Thorium: 24));
	PHASE_WALL: "phase-wall" => SimpleBlock::new(1, true, cost!(PhaseFabric: 6));
	PHASE_WALL_LARGE: "phase-wall-large" => SimpleBlock::new(2, true, cost!(PhaseFabric: 24));
	SURGE_WALL: "surge-wall" => SimpleBlock::new(1, true, cost!(SurgeAlloy: 6));
	SURGE_WALL_LARGE: "surge-wall-large" => SimpleBlock::new(2, true, cost!(SurgeAlloy: 24));
	DOOR: "door" => DoorBlock::new(1, true, cost!(Titanium: 6, Silicon: 4));
	DOOR_LARGE: "door-large" => DoorBlock::new(2, true, cost!(Titanium: 24, Silicon: 16));
	// sandbox only
	SCRAP_WALL: "scrap-wall" => SimpleBlock::new(1, true, cost!(Scrap: 6));
	SCRAP_WALL_LARGE: "scrap-wall-large" => SimpleBlock::new(2, true, cost!(Scrap: 24));
	SCRAP_WALL_HUGE: "scrap-wall-huge" => SimpleBlock::new(3, true, cost!(Scrap: 54));
	SCRAP_WALL_GIGANTIC: "scrap-wall-gigantic" => SimpleBlock::new(4, true, cost!(Scrap: 96));
	THRUSTER: "thruster" => SimpleBlock::new(4, false, cost!(Scrap: 96));
);

pub struct DoorBlock
{
	size: u8,
	symmetric: bool,
	build_cost: BuildCost,
}

impl DoorBlock
{
	pub const fn new(size: u8, symmetric: bool, build_cost: BuildCost) -> Self
	{
		if size == 0
		{
			panic!("invalid size");
		}
		Self{size, symmetric, build_cost}
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
