use std::any::Any;
use std::error::Error;
use std::fmt;

use crate::block::{BlockLogic, DataConvertError, DeserializeError, make_register, SerializeError};
use crate::block::simple::{SimpleBlock, state_impl};
use crate::data::GridPos;
use crate::data::dynamic::{DynData, DynType};
use crate::unit;

const GROUND_UNITS: &[unit::Type] = &[unit::Type::Dagger, unit::Type::Crawler, unit::Type::Nova];
const AIR_UNITS: &[unit::Type] = &[unit::Type::Flare, unit::Type::Mono];
const NAVAL_UNITS: &[unit::Type] = &[unit::Type::Risso, unit::Type::Retusa];

make_register!
(
	GROUND_FACTORY: "ground-factory" => AssemblerBlock::new(3, false, GROUND_UNITS);
	AIR_FACTORY: "air-factory" => AssemblerBlock::new(3, false, AIR_UNITS);
	NAVAL_FACTORY: "naval-factory" => AssemblerBlock::new(3, false, NAVAL_UNITS);
	ADDITIVE_RECONSTRUCTOR: "additive-reconstructor" => SimpleBlock::new(3, false);
	MULTIPLICATIVE_RECONSTRUCTOR: "multiplicative-reconstructor" => SimpleBlock::new(5, false);
	EXPONENTIAL_RECONSTRUCTOR: "exponential-reconstructor" => SimpleBlock::new(7, false);
	TETRATIVE_RECONSTRUCTOR: "tetrative-reconstructor" => SimpleBlock::new(9, false);
	REPAIR_POINT: "repair-point" => SimpleBlock::new(1, true);
	REPAIR_TURRET: "repair-turret" => SimpleBlock::new(2, true);
	PAYLOAD_CONVEYOR: "payload-conveyor" => SimpleBlock::new(3, false);
	PAYLOAD_ROUTER: "payload-router" => SimpleBlock::new(3, false);
	// sandbox only
	PAYLOAD_SOURCE: "payload-source" => SimpleBlock::new(5, false); // TODO config: block/unit
	PAYLOAD_VOID: "payload-void" => SimpleBlock::new(5, true);
);

pub struct AssemblerBlock
{
	size: u8,
	symmetric: bool,
	valid: &'static [unit::Type],
}

impl AssemblerBlock
{
	pub const fn new(size: u8, symmetric: bool, valid: &'static [unit::Type]) -> Self
	{
		if size == 0
		{
			panic!("invalid size");
		}
		if valid.is_empty()
		{
			panic!("no valid units");
		}
		if valid.len() > i32::MAX as usize
		{
			panic!("too many valid units");
		}
		Self{size, symmetric, valid}
	}
	
	state_impl!(pub Option<unit::Type>);
}

impl BlockLogic for AssemblerBlock
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
		Ok(DynData::Int(-1))
	}
	
	fn deserialize_state(&self, data: DynData) -> Result<Option<Box<dyn Any>>, DeserializeError>
	{
		match data
		{
			DynData::Empty => Ok(Some(Self::create_state(None))),
			DynData::Int(idx) =>
			{
				if idx == -1
				{
					Ok(Some(Self::create_state(None)))
				}
				else if idx >= 0 && idx < self.valid.len() as i32
				{
					Ok(Some(Self::create_state(Some(self.valid[idx as usize]))))
				}
				else
				{
					Err(DeserializeError::Custom(Box::new(AssemblerDeserializeError{idx, count: self.valid.len() as i32})))
				}
			},
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
		if let Some(state) = Self::get_state(state)
		{
			for (i, curr) in self.valid.iter().enumerate()
			{
				if curr == state
				{
					return Ok(DynData::Int(i as i32));
				}
			}
			Err(SerializeError::Custom(Box::new(AssemblerSerializeError(*state))))
		}
		else
		{
			Ok(DynData::Int(-1))
		}
	}
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AssemblerDeserializeError
{
	pub idx: i32,
	pub count: i32,
}

impl fmt::Display for AssemblerDeserializeError
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
	{
		write!(f, "invalid unit index ({}, #valid: {})", self.idx, self.count)
	}
}

impl Error for AssemblerDeserializeError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AssemblerSerializeError(unit::Type);

impl fmt::Display for AssemblerSerializeError
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
	{
		write!(f, "invalid unit ({:?}) is not valid", self.0)
	}
}

impl Error for AssemblerSerializeError {}
