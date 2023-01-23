use std::any::Any;
use std::error::Error;
use std::fmt;

use crate::block::{BlockLogic, DataConvertError, DeserializeError, make_register, SerializeError};
use crate::block::simple::{BuildCost, cost, SimpleBlock, state_impl};
use crate::data::GridPos;
use crate::data::dynamic::{DynData, DynType};
use crate::item::storage::Storage;

make_register!
(
	POWER_NODE: "power-node" => ConnectorBlock::new(1, true, cost!(Copper: 1, Lead: 3), 10);
	POWER_NODE_LARGE: "power-node-large" => ConnectorBlock::new(2, true, cost!(Lead: 10, Titanium: 5, Silicon: 3), 15);
	SURGE_TOWER: "surge-tower" => ConnectorBlock::new(2, true, cost!(Lead: 10, Titanium: 7, Silicon: 15, SurgeAlloy: 15), 2);
	DIODE: "diode" => SimpleBlock::new(1, false, cost!(Metaglass: 10, Silicon: 10, Plastanium: 5));
	BATTERY: "battery" => SimpleBlock::new(1, true, cost!(Copper: 5, Lead: 20));
	BATTERY_LARGE: "battery-large" => SimpleBlock::new(3, true, cost!(Lead: 50, Titanium: 20, Silicon: 30));
	COMBUSTION_GENERATOR: "combustion-generator" => SimpleBlock::new(1, true, cost!(Copper: 25, Lead: 15));
	THERMAL_GENERATOR: "thermal-generator" => SimpleBlock::new(2, true, cost!(Copper: 40, Lead: 50, Metaglass: 40, Graphite: 35, Silicon: 35));
	STEAM_GENERATOR: "steam-generator" => SimpleBlock::new(2, true, cost!(Copper: 35, Lead: 40, Graphite: 25, Silicon: 30));
	DIFFERENTIAL_GENERATOR: "differential-generator" => SimpleBlock::new(3, true, cost!(Copper: 70, Lead: 100, Metaglass: 50, Titanium: 50, Silicon: 65));
	RTG_GENERATOR: "rtg-generator" => SimpleBlock::new(2, true, cost!(Lead: 100, Thorium: 50, Silicon: 75, Plastanium: 75, PhaseFabric: 25));
	SOLAR_PANEL: "solar-panel" => SimpleBlock::new(1, true, cost!(Lead: 10, Silicon: 15));
	SOLAR_PANEL_LARGE: "solar-panel-large" => SimpleBlock::new(3, true, cost!(Lead: 80, Silicon: 110, PhaseFabric: 15));
	THORIUM_REACTOR: "thorium-reactor" => SimpleBlock::new(3, true, cost!(Lead: 300, Metaglass: 50, Graphite: 150, Thorium: 150, Silicon: 200));
	IMPACT_REACTOR: "impact-reactor" => SimpleBlock::new(4, true,
		cost!(Lead: 500, Metaglass: 250, Graphite: 400, Thorium: 100, Silicon: 300, SurgeAlloy: 250));
	// sandbox only
	POWER_SOURCE: "power-source" => ConnectorBlock::new(1, true, &[], 100);
	POWER_VOID: "power-void" => SimpleBlock::new(1, true, &[]);
);

pub struct ConnectorBlock
{
	size: u8,
	symmetric: bool,
	build_cost: BuildCost,
	max: u8,
}

impl ConnectorBlock
{
	pub const fn new(size: u8, symmetric: bool, build_cost: BuildCost, max: u8) -> Self
	{
		if size == 0
		{
			panic!("invalid size");
		}
		if max == 0 || max > i8::MAX as u8
		{
			panic!("invalid maximum link count");
		}
		Self{size, symmetric, build_cost, max}
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
			Self::LinkCount{have, max} => write!(f, "too many links ({have} but only {max} supported)"),
		}
	}
}

impl Error for ConnectorDeserializeError {}
