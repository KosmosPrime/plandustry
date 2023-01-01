use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

use crate::access::BoxAccess;
use crate::data::dynamic::DynData;

pub trait BlockLogic
{
	fn get_size(&self) -> u8;
	
	fn is_symmetric(&self) -> bool
	{
		true
	}
	
	fn state_from_i32(&self, _config: i32) -> DynData
	{
		DynData::Empty
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
	
	pub fn state_from_i32(&self, config: i32) -> DynData
	{
		self.logic.state_from_i32(config)
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
