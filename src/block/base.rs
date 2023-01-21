use std::any::Any;

use crate::block::{BlockLogic, DataConvertError, DeserializeError, make_register, SerializeError};
use crate::block::simple::{SimpleBlock, state_impl};
use crate::block::transport::ItemBlock;
use crate::data::GridPos;
use crate::data::dynamic::{DynData, DynType};

make_register!
(
	MENDER: "mender" => SimpleBlock::new(1, true);
	MEND_PROJECTOR: "mend-projector" => SimpleBlock::new(2, true);
	OVERDRIVE_PROJECTOR: "overdrive-projector" => SimpleBlock::new(2, true);
	OVERDRIVE_DOME: "overdrive-dome" => SimpleBlock::new(3, true);
	FORCE_PROJECTOR: "force-projector" => SimpleBlock::new(3, true);
	SHOCK_MINE: "shock-mine" => SimpleBlock::new(1, true);
	CORE_SHARD: "core-shard" => SimpleBlock::new(3, true);
	CORE_FOUNDATION: "core-foundation" => SimpleBlock::new(4, true);
	CORE_NUCLEUS: "core-nucleus" => SimpleBlock::new(5, true);
	CONTAINER: "container" => SimpleBlock::new(2, true);
	VAULT: "vault" => SimpleBlock::new(3, true);
	UNLOADER: "unloader" => ItemBlock::new(1, true);
	ILLUMINATOR: "illuminator" => LampBlock::new(1, true);
	LAUNCH_PAD: "launch-pad" => SimpleBlock::new(3, true);
);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RGBA(u8, u8, u8, u8);

impl From<u32> for RGBA
{
	fn from(value: u32) -> Self
	{
		Self((value >> 24) as u8, (value >> 16) as u8, (value >> 8) as u8, value as u8)
	}
}

impl From<RGBA> for u32
{
	fn from(value: RGBA) -> Self
	{
		((value.0 as u32) << 24) | ((value.1 as u32) << 16) | ((value.2 as u32) << 8) | (value.3 as u32)
	}
}

pub struct LampBlock
{
	size: u8,
	symmetric: bool,
}

impl LampBlock
{
	pub const fn new(size: u8, symmetric: bool) -> Self
	{
		if size == 0
		{
			panic!("invalid size");
		}
		Self{size, symmetric}
	}
	
	state_impl!(pub RGBA);
}

impl BlockLogic for LampBlock
{
	fn get_size(&self) -> u8
	{
		self.size
	}
	
	fn is_symmetric(&self) -> bool
	{
		self.symmetric
	}
	
	fn data_from_i32(&self, config: i32, _: GridPos) -> Result<DynData, DataConvertError>
	{
		Ok(DynData::Int(config))
	}
	
	fn deserialize_state(&self, data: DynData) -> Result<Option<Box<dyn Any>>, DeserializeError>
	{
		match data
		{
			DynData::Int(rgba) => Ok(Some(Self::create_state(RGBA::from(rgba as u32)))),
			_ => Err(DeserializeError::InvalidType{have: data.get_type(), expect: DynType::Int}),
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
		Ok(DynData::Int(u32::from(*state) as i32))
	}
}
