use crate::block::BlockLogic;

pub struct SimpleBlock
{
	size: u8,
	symmetric: bool,
}

impl SimpleBlock
{
	pub const fn new(size: u8, symmetric: bool) -> Self
	{
		Self{size, symmetric}
	}
}

impl BlockLogic for SimpleBlock
{
	fn get_size(&self) -> u8
	{
		self.size
	}
	
	fn is_symmetric(&self) -> bool
	{
		self.symmetric
	}
}
