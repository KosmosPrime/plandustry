use std::any::Any;
use std::error::Error;
use std::fmt;

use crate::block::{BlockLogic, DataConvertError, DeserializeError, make_register, SerializeError};
use crate::block::simple::{SimpleBlock, state_impl};
use crate::content;
use crate::data::GridPos;
use crate::data::dynamic::{DynData, DynType};
use crate::fluid;

make_register!
(
	MECHANICAL_PUMP: "mechanical-pump" => SimpleBlock::new(1, true);
	ROTARY_PUMP: "rotary-pump" => SimpleBlock::new(2, true);
	IMPULSE_PUMP: "impulse-pump" => SimpleBlock::new(3, true);
	CONDUIT: "conduit" => SimpleBlock::new(1, false);
	PULSE_CONDUIT: "pulse-conduit" => SimpleBlock::new(1, false);
	PLATED_CONDUIT: "plated-conduit" => SimpleBlock::new(1, false);
	LIQUID_ROUTER: "liquid-router" => SimpleBlock::new(1, true);
	LIQUID_CONTAINER: "liquid-container" => SimpleBlock::new(2, true);
	LIQUID_TANK: "liquid-tank" => SimpleBlock::new(3, true);
	LIQUID_JUNCTION: "liquid-junction" => SimpleBlock::new(1, true);
	BRIDGE_CONDUIT: "bridge-conduit" => SimpleBlock::new(1, true); // TODO config: destination
	PHASE_CONDUIT: "phase-conduit" => SimpleBlock::new(1, true); // TODO config: destination
	// sandbox only
	LIQUID_SOURCE: "liquid-source" => FluidBlock::new(1, true);
	LIQUID_VOID: "liquid-void" => SimpleBlock::new(1, true);
);

pub struct FluidBlock
{
	size: u8,
	symmetric: bool,
}

impl FluidBlock
{
	pub const fn new(size: u8, symmetric: bool) -> Self
	{
		if size == 0
		{
			panic!("invalid size");
		}
		Self{size, symmetric}
	}
	
	state_impl!(pub Option<fluid::Type>);
}

impl BlockLogic for FluidBlock
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
		if config < 0 || config > u16::MAX as i32
		{
			return Err(DataConvertError(Box::new(FluidConvertError(config))));
		}
		Ok(DynData::Content(content::Type::Fluid, config as u16))
	}
	
	fn deserialize_state(&self, data: DynData) -> Result<Option<Box<dyn Any>>, DeserializeError>
	{
		match data
		{
			DynData::Empty => Ok(Some(Self::create_state(None))),
			DynData::Content(content::Type::Fluid, id) => Ok(Some(Self::create_state(Some(FluidDeserializeError::forward(fluid::Type::try_from(id))?)))),
			DynData::Content(have, ..) => Err(DeserializeError::Custom(Box::new(FluidDeserializeError::ContentType(have)))),
			_ => Err(DeserializeError::InvalidType{have: data.get_type(), expect: DynType::Content}),
		}
	}
	
	fn clone_state(&self, state: &dyn Any) -> Box<dyn Any>
	{
		let state = Self::get_state(state);
		Box::new(Self::create_state(*state))
	}
	
	fn serialize_state(&self, state: &dyn Any) -> Result<DynData, SerializeError>
	{
		match Self::get_state(state)
		{
			None => Ok(DynData::Empty),
			Some(fluid) => Ok(DynData::Content(content::Type::Fluid, (*fluid).into())),
		}
	}
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FluidConvertError(pub i32);

impl fmt::Display for FluidConvertError
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
	{
		write!(f, "invalid config ({}) for fluid", self.0)
	}
}

impl Error for FluidConvertError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FluidDeserializeError
{
	ContentType(content::Type),
	NotFound(fluid::TryFromU16Error),
}

impl FluidDeserializeError
{
	pub fn forward<T, E: Into<Self>>(result: Result<T, E>) -> Result<T, DeserializeError>
	{
		match result
		{
			Ok(v) => Ok(v),
			Err(e) => Err(DeserializeError::Custom(Box::new(e.into()))),
		}
	}
}

impl From<fluid::TryFromU16Error> for FluidDeserializeError
{
	fn from(err: fluid::TryFromU16Error) -> Self
	{
		Self::NotFound(err)
	}
}

impl fmt::Display for FluidDeserializeError
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
	{
		match self
		{
			Self::ContentType(have) => write!(f, "expected content {:?} but got {have:?}", content::Type::Fluid),
			Self::NotFound(e) => e.fmt(f),
		}
	}
}

impl Error for FluidDeserializeError
{
	fn source(&self) -> Option<&(dyn Error + 'static)>
	{
		match self
		{
			Self::NotFound(e) => Some(e),
			_ => None,
		}
	}
}
