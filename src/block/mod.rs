use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

pub trait BlockLogic
{
	fn get_size(&self) -> u8;
	
	fn is_symmetric(&self) -> bool
	{
		true
	}
}

pub struct Block
{
	name: Cow<'static, str>,
	logic: Box<dyn BlockLogic>,
}

impl Block
{
	pub fn new(name: Cow<'static, str>, logic: Box<dyn BlockLogic>) -> Self
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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Rotation
{
	Right, Up, Left, Down
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
	
	pub fn register(&mut self, block: &'l Block) -> Result<(), &'l Block>
	{
		let key = match block.name
		{
			Cow::Borrowed(r) => r,
			Cow::Owned(ref s) => s,
		};
		match self.blocks.entry(key)
		{
			Entry::Occupied(e) => Err(e.get()),
			Entry::Vacant(e) =>
			{
				e.insert(block);
				Ok(())
			},
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
