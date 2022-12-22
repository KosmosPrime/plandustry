use std::collections::HashMap;

use crate::block::{Block, Rotation};

pub const MAX_DIMENSION: u16 = 128;
pub const MAX_BLOCKS: u32 = 128 * 128;

#[derive(Clone, Copy)]
struct Storage(&'static Block, Rotation);

#[derive(Clone)]
pub struct Schematic
{
	width: u16,
	height: u16,
	tags: HashMap<String, String>,
	blocks: Vec<Option<Storage>>,
	block_cnt: u32,
}

impl Schematic
{
	pub fn new(width: u16, height: u16) -> Self
	{
		if width > MAX_DIMENSION
		{
			panic!("invalid schematic width ({width})");
		}
		if height > MAX_DIMENSION
		{
			panic!("invalid schematic width ({height})");
		}
		let mut tags = HashMap::<String, String>::new();
		tags.insert("name".to_string(), String::new());
		tags.insert("description".to_string(), String::new());
		tags.insert("labels".to_string(), "[]".to_string());
		Self{width, height, tags, blocks: Vec::new(), block_cnt: 0}
	}
	
	pub fn get_width(&self) -> u16
	{
		self.width
	}
	
	pub fn get_height(&self) -> u16
	{
		self.height
	}
	
	pub fn get_tags(&self) -> &HashMap<String, String>
	{
		&self.tags
	}
	
	pub fn get_tags_mut(&mut self) -> &mut HashMap<String, String>
	{
		&mut self.tags
	}
	
	pub fn get(&self, x: u16, y: u16) -> Option<(&'static Block, Rotation)>
	{
		if x >= self.width || y >= self.height
		{
			panic!("position {x} / {y} out of bounds ({} / {})", self.width, self.height);
		}
		if self.block_cnt == 0 {return None;}
		let index = (x as usize) + (y as usize) * (self.width as usize);
		match self.blocks.get(index)
		{
			None => None,
			Some(None) => None,
			Some(Some(Storage(b, r))) => Some((*b, *r)),
		}
	}
	
	pub fn set(&mut self, x: u16, y: u16, block: &'static Block, rot: Rotation) -> Option<(&'static Block, Rotation)>
	{
		if x >= self.width || y >= self.height
		{
			panic!("position {x} / {y} out of bounds ({} / {})", self.width, self.height);
		}
		if self.blocks.len() == 0
		{
			self.blocks.resize_with((self.width as usize) * (self.height as usize), || None);
		}
		let index = (x as usize) + (y as usize) * (self.width as usize);
		match self.blocks[index].replace(Storage(block, rot))
		{
			None =>
			{
				self.block_cnt += 1;
				None
			},
			Some(s) => Some((s.0, s.1)),
		}
	}
	
	pub fn take(&mut self, x: u16, y: u16) -> Option<(&'static Block, Rotation)>
	{
		if x >= self.width || y >= self.height
		{
			panic!("position {x} / {y} out of bounds ({} / {})", self.width, self.height);
		}
		if self.blocks.len() > 0
		{
			let index = (x as usize) + (y as usize) * (self.width as usize);
			match self.blocks[index].take()
			{
				None => None,
				Some(s) =>
				{
					self.block_cnt -= 1;
					Some((s.0, s.1))
				},
			}
		}
		else {None}
	}
}
