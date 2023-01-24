use std::any::Any;
use std::error::Error;
use std::fmt;

use crate::block::{BlockLogic, DataConvertError, DeserializeError, make_register, SerializeError};
use crate::block::simple::{BuildCost, cost, SimpleBlock, state_impl};
use crate::content;
use crate::data::GridPos;
use crate::data::dynamic::{DynData, DynType};
use crate::item;
use crate::item::storage::Storage;

make_register!
(
	CONVEYOR: "conveyor" => SimpleBlock::new(1, false, cost!(Copper: 1));
	TITANIUM_CONVEYOR: "titanium-conveyor" => SimpleBlock::new(1, false, cost!(Copper: 1, Lead: 1, Titanium: 1));
	PLASTANIUM_CONVEYOR: "plastanium-conveyor" => SimpleBlock::new(1, false, cost!(Graphite: 1, Silicon: 1, Plastanium: 1));
	ARMORED_CONVEYOR: "armored-conveyor" => SimpleBlock::new(1, false, cost!(Metaglass: 1, Thorium: 1, Plastanium: 1));
	JUNCTION: "junction" => SimpleBlock::new(1, true, cost!(Copper: 2));
	BRIDGE_CONVEYOR: "bridge-conveyor" => BridgeBlock::new(1, false, cost!(Copper: 6, Lead: 6), 4, true);
	PHASE_CONVEYOR: "phase-conveyor" => BridgeBlock::new(1, false, cost!(Lead: 10, Graphite: 10, Silicon: 7, PhaseFabric: 5), 12, true);
	SORTER: "sorter" => ItemBlock::new(1, true, cost!(Copper: 2, Lead: 2));
	INVERTED_SORTER: "inverted-sorter" => ItemBlock::new(1, true, cost!(Copper: 2, Lead: 2));
	ROUTER: "router" => SimpleBlock::new(1, true, cost!(Copper: 3));
	DISTRIBUTOR: "distributor" => SimpleBlock::new(2, true, cost!(Copper: 4, Lead: 4));
	OVERFLOW_GATE: "overflow-gate" => SimpleBlock::new(1, true, cost!(Copper: 4, Lead: 2));
	UNDERFLOW_GATE: "underflow-gate" => SimpleBlock::new(1, true, cost!(Copper: 4, Lead: 2));
	MASS_DRIVER: "mass-driver" => BridgeBlock::new(3, true, cost!(Lead: 125, Titanium: 125, Thorium: 50, Silicon: 75), 55, false);
	// sandbox only
	ITEM_SOURCE: "item-source" => ItemBlock::new(1, true, &[]);
	ITEM_VOID: "item-void" => SimpleBlock::new(1, true, &[]);
);

pub struct ItemBlock
{
	size: u8,
	symmetric: bool,
	build_cost: BuildCost,
}

impl ItemBlock
{
	pub const fn new(size: u8, symmetric: bool, build_cost: BuildCost) -> Self
	{
		if size == 0
		{
			panic!("invalid size");
		}
		Self{size, symmetric, build_cost}
	}
	
	state_impl!(pub Option<item::Type>);
}

impl BlockLogic for ItemBlock
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
	
	fn data_from_i32(&self, config: i32, _: GridPos) -> Result<DynData, DataConvertError>
	{
		if config < 0 || config > u16::MAX as i32
		{
			return Err(DataConvertError::Custom(Box::new(ItemConvertError(config))));
		}
		Ok(DynData::Content(content::Type::Item, config as u16))
	}
	
	fn deserialize_state(&self, data: DynData) -> Result<Option<Box<dyn Any>>, DeserializeError>
	{
		match data
		{
			DynData::Empty => Ok(Some(Self::create_state(None))),
			DynData::Content(content::Type::Item, id) => Ok(Some(Self::create_state(Some(ItemDeserializeError::forward(item::Type::try_from(id))?)))),
			DynData::Content(have, ..) => Err(DeserializeError::Custom(Box::new(ItemDeserializeError::ContentType(have)))),
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
			Some(item) => Ok(DynData::Content(content::Type::Item, (*item).into())),
		}
	}
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ItemConvertError(pub i32);

impl fmt::Display for ItemConvertError
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
	{
		write!(f, "invalid config ({}) for item", self.0)
	}
}

impl Error for ItemConvertError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ItemDeserializeError
{
	ContentType(content::Type),
	NotFound(item::TryFromU16Error),
}

impl ItemDeserializeError
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

impl From<item::TryFromU16Error> for ItemDeserializeError
{
	fn from(err: item::TryFromU16Error) -> Self
	{
		Self::NotFound(err)
	}
}

impl fmt::Display for ItemDeserializeError
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
	{
		match self
		{
			Self::ContentType(have) => write!(f, "expected content {:?} but got {have:?}", content::Type::Item),
			Self::NotFound(..) => f.write_str("target item not found"),
		}
	}
}

impl Error for ItemDeserializeError
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

pub struct BridgeBlock
{
	size: u8,
	symmetric: bool,
	build_cost: BuildCost,
	range: u16,
	ortho: bool,
}

type Point2 = (i32, i32);

impl BridgeBlock
{
	pub const fn new(size: u8, symmetric: bool, build_cost: BuildCost, range: u16, ortho: bool) -> Self
	{
		if size == 0
		{
			panic!("invalid size");
		}
		if range == 0
		{
			panic!("invalid range");
		}
		Self{size, symmetric, build_cost, range, ortho}
	}
	
	state_impl!(pub Option<Point2>);
}

impl BlockLogic for BridgeBlock
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
	
	fn data_from_i32(&self, config: i32, pos: GridPos) -> Result<DynData, DataConvertError>
	{
		let (x, y) = ((config >> 16) as i16, config as i16);
		if x < 0 || y < 0
		{
			return Err(DataConvertError::Custom(Box::new(BridgeConvertError{x, y})));
		}
		let dx = x as i32 - pos.0 as i32;
		let dy = y as i32 - pos.1 as i32;
		Ok(DynData::Point2(dx, dy))
	}
	
	fn deserialize_state(&self, data: DynData) -> Result<Option<Box<dyn Any>>, DeserializeError>
	{
		match data
		{
			DynData::Empty => Ok(Some(Self::create_state(None))),
			DynData::Point2(dx, dy) =>
			{
				if self.ortho
				{
					// the game uses (-worldX, -worldY) to indicate no target
					// likely because the absolute target being (0, 0) means it's unlinked
					if dx != 0
					{
						if dy != 0
						{
							return Ok(Some(Self::create_state(None)));
						}
						else
						{
							if dx > self.range as i32 || dx < -(self.range as i32)
							{
								return Ok(Some(Self::create_state(None)));
							}
						}
					}
					else
					{
						if dy > self.range as i32 || dy < -(self.range as i32)
						{
							return Ok(Some(Self::create_state(None)));
						}
					}
				}
				// can't check range otherwise, it depends on the target's size
				Ok(Some(Self::create_state(Some((dx, dy)))))
			},
			_ => Err(DeserializeError::InvalidType{have: data.get_type(), expect: DynType::Point2}),
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
			Some((dx, dy)) => Ok(DynData::Point2(*dx, *dy)),
		}
	}
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BridgeConvertError
{
	pub x: i16,
	pub y: i16,
}

impl fmt::Display for BridgeConvertError
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
	{
		write!(f, "invalid coordinate ({} / {}) for bridge", self.x, self.y)
	}
}

impl Error for BridgeConvertError {}
