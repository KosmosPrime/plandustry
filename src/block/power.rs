use std::any::Any;
use std::error::Error;
use std::fmt;

use crate::block::{BlockLogic, DeserializeError, make_register, SerializeError};
use crate::block::simple::{SimpleBlock, state_impl};
use crate::data::GridPos;
use crate::data::dynamic::{DynData, DynType};

make_register!
(
	POWER_NODE: "power-node" => ConnectorBlock::new(1, 10);
	POWER_NODE_LARGE: "power-node-large" => ConnectorBlock::new(2, 15);
	SURGE_TOWER: "surge-tower" => ConnectorBlock::new(2, 2);
	DIODE: "diode" => SimpleBlock::new(1, false);
	BATTERY: "battery" => SimpleBlock::new(1, true);
	BATTERY_LARGE: "battery-large" => SimpleBlock::new(3, true);
	COMBUSTION_GENERATOR: "combustion-generator" => SimpleBlock::new(1, true);
	THERMAL_GENERATOR: "thermal-generator" => SimpleBlock::new(2, true);
	STEAM_GENERATOR: "steam-generator" => SimpleBlock::new(2, true);
	DIFFERENTIAL_GENERATOR: "differential-generator" => SimpleBlock::new(3, true);
	RTG_GENERATOR: "rtg-generator" => SimpleBlock::new(2, true);
	SOLAR_PANEL: "solar-panel" => SimpleBlock::new(1, true);
	SOLAR_PANEL_LARGE: "solar-panel-large" => SimpleBlock::new(3, true);
	THORIUM_REACTOR: "thorium-reactor" => SimpleBlock::new(3, true);
	IMPACT_REACTOR: "impact-reactor" => SimpleBlock::new(4, true);
	POWER_SOURCE: "power-source" => ConnectorBlock::new(1, 100);
	POWER_VOID: "power-void" => SimpleBlock::new(1, true);
);

pub struct ConnectorBlock
{
	size: u8,
	max: u8,
}

impl ConnectorBlock
{
	pub const fn new(size: u8, max: u8) -> Self
	{
		if size == 0
		{
			panic!("invalid size");
		}
		if max == 0 || max > i8::MAX as u8
		{
			panic!("invalid maximum link count");
		}
		Self{size, max}
	}
	
	pub fn get_max_links(&self) -> u8
	{
		self.max
	}
	
	state_impl!(pub Vec<(i16, i16)>);
}

impl BlockLogic for ConnectorBlock
{
	fn get_size(&self) -> u8
	{
		self.size
	}
	
	fn is_symmetric(&self) -> bool
	{
		true
	}
	
	fn data_from_i32(&self, _: i32, _: GridPos) -> DynData
	{
		DynData::Empty
	}
	
	fn deserialize_state(&self, data: DynData) -> Result<Option<Box<dyn Any>>, DeserializeError>
	{
		match data
		{
			DynData::Empty => Ok(Some(Self::create_state(Vec::new()))),
			DynData::Point2Array(s) =>
			{
				Ok(Some(Self::create_state(s)))
			},
			_ => Err(DeserializeError::InvalidType{have: data.get_type(), expect: DynType::Boolean}),
		}
	}
	
	fn clone_state(&self, state: &dyn Any) -> Box<dyn Any>
	{
		Box::new(Self::get_state(state).clone())
	}
	
	fn serialize_state(&self, state: &dyn Any) -> Result<DynData, SerializeError>
	{
		Ok(DynData::Point2Array(Self::get_state(state).clone()))
	}
}

#[derive(Debug)]
pub enum ConnectorDeserializeError
{
	LinkCount{have: usize, max: u8},
}

impl ConnectorDeserializeError
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

impl fmt::Display for ConnectorDeserializeError
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
	{
		match self
		{
			Self::LinkCount{have, max} => write!(f, "Too many links ({have} but only {max} supported)"),
		}
	}
}

impl Error for ConnectorDeserializeError {}
