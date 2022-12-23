use std::collections::HashMap;
use std::iter::FusedIterator;

use crate::block::{Block, Rotation};
use crate::data::GridPos;

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
	
	pub fn pos_iter(&self) -> PosIter
	{
		PosIter{x: 0, y: 0, w: self.width, h: self.height}
	}
	
	pub fn block_iter(&self) -> BlockIter
	{
		BlockIter{x: 0, y: 0, schematic: self, encountered: 0}
	}
}

pub struct PosIter
{
	x: u16,
	y: u16,
	w: u16,
	h: u16,
}

impl Iterator for PosIter
{
	type Item = GridPos;
	
	fn next(&mut self) -> Option<Self::Item>
	{
		if self.w > 0 && self.y < self.h
		{
			let p = GridPos(self.x, self.y);
			self.x += 1;
			if self.x == self.w
			{
				self.x = 0;
				self.y += 1;
			}
			Some(p)
		}
		else {None}
	}
	
	fn size_hint(&self) -> (usize, Option<usize>)
	{
		let pos = (self.x as usize) + (self.y as usize) * (self.w as usize);
		let end = (self.w as usize) * (self.h as usize);
		(end - pos, Some(end - pos))
	}
	
	fn count(self) -> usize
	{
		let pos = (self.x as usize) + (self.y as usize) * (self.w as usize);
		let end = (self.w as usize) * (self.h as usize);
		end - pos
	}
	
	fn last(self) -> Option<Self::Item>
	{
		// self.y < self.h implies self.h > 0
		if self.w > 0 && self.y < self.h
		{
			Some(GridPos(self.w - 1, self.h - 1))
		}
		else {None}
	}
}

impl FusedIterator for PosIter {}

pub struct BlockIter<'l>
{
	x: u16,
	y: u16,
	schematic: &'l Schematic,
	encountered: u32,
}

impl<'l> Iterator for BlockIter<'l>
{
	type Item = (GridPos, &'static Block, Rotation);
	
	fn next(&mut self) -> Option<Self::Item>
	{
		let w = self.schematic.width;
		let blocks: &[Option<Storage>] = &self.schematic.blocks;
		let pos = (self.x as usize) + (self.y as usize) * (w as usize);
		if blocks.len() <= pos
		{
			return None;
		}
		if let Some(ref s) = blocks[pos]
		{
			let p = GridPos(self.x, self.y);
			self.x += 1;
			if self.x == w
			{
				self.x = 0;
				self.y += 1;
			}
			self.encountered += 1;
			Some((p, s.0, s.1))
		}
		else
		{
			match blocks[pos..].iter().enumerate().find(|(_, v)| v.is_some())
			{
				None =>
				{
					// move to the end of the iterator
					self.x = 0;
					self.y = self.schematic.height;
					None
				},
				Some((i, Some(s))) =>
				{
					// compute the coordinate of this result
					let i0 = i + self.x as usize;
					let x = (i0 % w as usize) as u16;
					let y = (i / w as usize) as u16;
					self.x = x + 1;
					if self.x == w
					{
						self.x = 0;
						self.y += 1;
					}
					self.encountered += 1;
					Some((GridPos(x, y), s.0, s.1))
				},
				_ => unreachable!("searched for Some but got None"),
			}
		}
	}
	
	fn size_hint(&self) -> (usize, Option<usize>)
	{
		let remain = self.schematic.block_cnt - self.encountered;
		(remain as usize, Some(remain as usize))
	}
	
	fn count(self) -> usize
	{
		(self.schematic.block_cnt - self.encountered) as usize
	}
	
	fn last(self) -> Option<Self::Item>
	{
		let w = self.schematic.width;
		let h = self.schematic.height;
		// self.y < h implies h > 0
		if w > 0 && self.y < h
		{
			let pos = (self.x as usize) + (self.y as usize) * (w as usize);
			let end = (w as usize) * (h as usize);
			let blocks: &[Option<Storage>] = &self.schematic.blocks;
			for i in (pos..end).rev()
			{
				if let Some(ref s) = blocks[i]
				{
					// last consumes self so we don't have to update fields
					let i0 = i + self.x as usize;
					let x = (i0 % w as usize) as u16;
					let y = (i / w as usize) as u16;
					return Some((GridPos(x, y), s.0, s.1));
				}
			}
			None
		}
		else {None}
	}
}

impl<'l> FusedIterator for BlockIter<'l> {}

#[cfg(test)]
mod test
{
	use super::*;
	
	macro_rules!test_iter
	{
		($name:ident, $it:expr, $($val:expr),+) =>
		{
			#[test]
			fn $name()
			{
				let mut it = $it;
				$(test_iter!(impl it, $val);)+
			}
		};
		(impl $it:ident, $val:literal) =>
		{
			for _ in 0..$val
			{
				assert_ne!($it.next(), None, "iterator returned None too early");
			}
		};
		(impl $it:ident, $val:expr) =>
		{
			assert_eq!($it.next(), $val);
		};
	}
	
	test_iter!(block_iter, Schematic::new(3, 4).pos_iter(), Some(GridPos(0, 0)), Some(GridPos(1, 0)), Some(GridPos(2, 0)),
		Some(GridPos(0, 1)), 7, Some(GridPos(2, 3)), None);
}
