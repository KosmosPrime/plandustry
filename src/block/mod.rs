use std::borrow::Cow;

pub trait BlockLogic
{
	fn get_size(&self) -> u8;
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
}
