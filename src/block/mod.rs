use std::any::Any;
use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::error::Error;
use std::fmt;

use crate::access::BoxAccess;
use crate::data::GridPos;
use crate::data::dynamic::{DynData, DynType};

pub mod base;
pub mod defense;
pub mod extraction;
pub mod factory;
pub mod fluid;
pub mod logic;
pub mod payload;
pub mod power;
pub mod simple;
pub mod transport;
pub mod turret;

pub trait BlockLogic
{
	fn get_size(&self) -> u8;
	
	fn is_symmetric(&self) -> bool;
	
	fn data_from_i32(&self, config: i32, pos: GridPos) -> DynData;
	
	fn deserialize_state(&self, data: DynData) -> Result<Option<Box<dyn Any>>, DeserializeError>;
	
	fn clone_state(&self, state: &dyn Any) -> Box<dyn Any>;
	
	fn serialize_state(&self, state: &dyn Any) -> Result<DynData, SerializeError>;
}

#[derive(Debug)]
pub enum DeserializeError
{
	InvalidType{have: DynType, expect: DynType},
	Custom(Box<dyn Error>),
}

impl DeserializeError
{
	pub fn filter<T, E: Error + 'static>(result: Result<T, E>) -> Result<T, Self>
	{
		match result
		{
			Ok(v) => Ok(v),
			Err(e) => Err(Self::Custom(Box::new(e))),
		}
	}
}

impl fmt::Display for DeserializeError
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
	{
		match self
		{
			Self::InvalidType{have, expect} => write!(f, "Expected type {expect:?} but got {have:?}"),
			Self::Custom(e) => e.fmt(f),
		}
	}
}

impl Error for DeserializeError
{
	fn source(&self) -> Option<&(dyn Error + 'static)>
	{
		match self
		{
			Self::Custom(e) => Some(e.as_ref()),
			_ => None,
		}
	}
}

#[derive(Debug)]
pub enum SerializeError
{
	Custom(Box<dyn Error>),
}

impl SerializeError
{
	pub fn filter<T, E: Error + 'static>(result: Result<T, E>) -> Result<T, Self>
	{
		match result
		{
			Ok(v) => Ok(v),
			Err(e) => Err(Self::Custom(Box::new(e))),
		}
	}
}

impl fmt::Display for SerializeError
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
	{
		match self
		{
			Self::Custom(e) => e.fmt(f),
		}
	}
}

impl Error for SerializeError
{
	fn source(&self) -> Option<&(dyn Error + 'static)>
	{
		match self
		{
			Self::Custom(e) => Some(e.as_ref()),
		}
	}
}

pub struct Block
{
	name: Cow<'static, str>,
	logic: BoxAccess<'static, dyn BlockLogic + Sync>,
}

impl Block
{
	pub const fn new(name: Cow<'static, str>, logic: BoxAccess<'static, dyn BlockLogic + Sync>) -> Self
	{
		Self{name, logic}
	}
	
	pub fn get_name(&self) -> &str
	{
		&self.name
	}
	
	pub fn get_size(&self) -> u8
	{
		self.logic.get_size()
	}
	
	pub fn is_symmetric(&self) -> bool
	{
		self.logic.is_symmetric()
	}
	
	pub fn data_from_i32(&self, config: i32, pos: GridPos) -> DynData
	{
		self.logic.data_from_i32(config, pos)
	}
	
	pub fn deserialize_state(&self, data: DynData) -> Result<Option<Box<dyn Any>>, DeserializeError>
	{
		self.logic.deserialize_state(data)
	}
	
	pub fn clone_state(&self, state: &dyn Any) -> Box<dyn Any>
	{
		self.logic.clone_state(state)
	}
	
	pub fn serialize_state(&self, state: &dyn Any) -> Result<DynData, SerializeError>
	{
		self.logic.serialize_state(state)
	}
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Rotation
{
	Right, Up, Left, Down
}

impl Rotation
{
	pub fn mirrored(self, horizontally: bool, vertically: bool) -> Self
	{
		match self
		{
			Rotation::Right => if horizontally {Rotation::Left} else {Rotation::Right},
			Rotation::Up => if vertically {Rotation::Down} else {Rotation::Up},
			Rotation::Left => if horizontally {Rotation::Right} else {Rotation::Left},
			Rotation::Down => if vertically {Rotation::Up} else {Rotation::Down},
		}
	}
	
	pub fn mirror(&mut self, horizontally: bool, vertically: bool)
	{
		*self = self.mirrored(horizontally, vertically);
	}
	
	pub fn rotated(self, clockwise: bool) -> Self
	{
		match self
		{
			Rotation::Right => if clockwise {Rotation::Up} else {Rotation::Down},
			Rotation::Up => if clockwise {Rotation::Left} else {Rotation::Right},
			Rotation::Left => if clockwise {Rotation::Down} else {Rotation::Up},
			Rotation::Down => if clockwise {Rotation::Right} else {Rotation::Left},
		}
	}
	
	pub fn rotate(&mut self, clockwise: bool)
	{
		*self = self.rotated(clockwise);
	}
	
	pub fn rotated_180(self) -> Self
	{
		match self
		{
			Rotation::Right => Rotation::Left,
			Rotation::Up => Rotation::Down,
			Rotation::Left => Rotation::Right,
			Rotation::Down => Rotation::Up,
		}
	}
	
	pub fn rotate_180(&mut self)
	{
		*self = self.rotated_180();
	}
}

impl From<u8> for Rotation
{
	fn from(val: u8) -> Self
	{
		match val & 3
		{
			0 => Rotation::Right,
			1 => Rotation::Up,
			2 => Rotation::Left,
			_ => Rotation::Down,
		}
	}
}

impl From<Rotation> for u8
{
	fn from(rot: Rotation) -> Self
	{
		match rot
		{
			Rotation::Right => 0,
			Rotation::Up => 1,
			Rotation::Left => 2,
			Rotation::Down => 3,
		}
	}
}

pub struct BlockRegistry<'l>
{
	blocks: HashMap<&'l str, &'l Block>,
}

impl<'l> BlockRegistry<'l>
{
	pub fn new() -> Self
	{
		Self{blocks: HashMap::new()}
	}
	
	pub fn register(&mut self, block: &'l Block) -> Result<&'l Block, &'l Block>
	{
		match self.blocks.entry(&block.name)
		{
			Entry::Occupied(e) => Err(e.get()),
			Entry::Vacant(e) => Ok(*e.insert(block)),
		}
	}
	
	pub fn get(&self, name: &str) -> Option<&'l Block>
	{
		self.blocks.get(name).map(|&r| r)
	}
}

impl<'l> AsRef<HashMap<&'l str, &'l Block>> for BlockRegistry<'l>
{
	fn as_ref(&self) -> &HashMap<&'l str, &'l Block>
	{
		&self.blocks
	}
}

macro_rules!make_register
{
	($($field:ident: $name:literal => $logic:expr;)+) =>
	{
		$(
			pub static $field: $crate::block::Block = $crate::block::Block::new(
				std::borrow::Cow::Borrowed($name), $crate::access::Access::Borrowed(&$logic));
		)+
		
		pub fn register<'l>(reg: &mut $crate::block::BlockRegistry<'l>)
		{
			$(assert!(reg.register(&$field).is_ok(), "duplicate block {:?}", $name);)+
		}
	};
}
pub(crate) use make_register;

pub fn build_registry() -> BlockRegistry<'static>
{
	let mut reg = BlockRegistry::new();
	register(&mut reg);
	reg
}

pub fn register<'l>(reg: &mut BlockRegistry<'l>)
{
	turret::register(reg);
	extraction::register(reg);
	transport::register(reg);
	fluid::register(reg);
	power::register(reg);
	defense::register(reg);
	factory::register(reg);
	payload::register(reg);
	base::register(reg);
	logic::register(reg);
}
