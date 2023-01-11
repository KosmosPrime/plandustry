use std::any::Any;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::error::Error;
use std::fmt::{self, Write};
use std::iter::FusedIterator;
use std::slice::Iter;

use flate2::{Compress, CompressError, Compression, Decompress, DecompressError, FlushCompress, FlushDecompress, Status};

use crate::block::{self, Block, BlockRegistry, Rotation};
use crate::data::{self, DataRead, DataWrite, GridPos, Serializer};
use crate::data::base64;
use crate::data::dynamic::{self, DynSerializer, DynData};

pub const MAX_DIMENSION: u16 = 128;
pub const MAX_BLOCKS: u32 = 128 * 128;

pub struct Placement
{
	pos: GridPos,
	block: &'static Block,
	state: Option<Box<dyn Any>>,
	rot: Rotation,
}

impl Placement
{
	pub fn get_pos(&self) -> GridPos
	{
		self.pos
	}
	
	pub fn get_block(&self) -> &'static Block
	{
		self.block
	}
	
	pub fn get_state(&self) -> Option<&dyn Any>
	{
		match self.state
		{
			None => None,
			Some(ref b) => Some(b.as_ref()),
		}
	}
	
	pub fn get_state_mut(&mut self) -> Option<&mut dyn Any>
	{
		match self.state
		{
			None => None,
			Some(ref mut b) => Some(b.as_mut()),
		}
	}
	
	pub fn set_state(&mut self, data: DynData) -> Result<Option<Box<dyn Any>>, block::DeserializeError>
	{
		let state = self.block.deserialize_state(data)?;
		Ok(std::mem::replace(&mut self.state, state))
	}
	
	pub fn get_rotation(&self) -> Rotation
	{
		self.rot
	}
	
	pub fn set_rotation(&mut self, rot: Rotation) -> Rotation
	{
		std::mem::replace(&mut self.rot, rot)
	}
}

// manual impl because trait objects cannot be cloned
impl Clone for Placement
{
	fn clone(&self) -> Self
	{
		Self
		{
			pos: self.pos,
			block: self.block,
			state: match self.state
			{
				None => None,
				Some(ref s) => Some(self.block.clone_state(s)),
			},
			rot: self.rot,
		}
	}
}

#[derive(Clone)]
pub struct Schematic
{
	width: u16,
	height: u16,
	tags: HashMap<String, String>,
	blocks: Vec<Placement>,
	lookup: Vec<Option<usize>>,
}

impl Schematic
{
	pub fn new(width: u16, height: u16) -> Self
	{
		match Self::try_new(width, height)
		{
			Ok(s) => s,
			Err(NewError::Width(w)) => panic!("invalid schematic width ({w})"),
			Err(NewError::Height(h)) => panic!("invalid schematic height ({h})"),
		}
	}
	
	pub fn try_new(width: u16, height: u16) -> Result<Self, NewError>
	{
		if width > MAX_DIMENSION
		{
			return Err(NewError::Width(width));
		}
		if height > MAX_DIMENSION
		{
			return Err(NewError::Height(height));
		}
		let mut tags = HashMap::<String, String>::new();
		tags.insert("name".to_string(), String::new());
		tags.insert("description".to_string(), String::new());
		tags.insert("labels".to_string(), "[]".to_string());
		Ok(Self{width, height, tags, blocks: Vec::new(), lookup: Vec::new()})
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
	
	pub fn is_empty(&self) -> bool
	{
		self.blocks.is_empty()
	}
	
	pub fn get_block_count(&self) -> usize
	{
		self.blocks.len()
	}
	
	pub fn is_region_empty(&self, x: u16, y: u16, w: u16, h: u16) -> bool
	{
		if self.blocks.len() == 0 {return true;}
		if x >= self.width || y >= self.height || w == 0 || h == 0 {return true;}
		if w > 1 || h > 1
		{
			let stride = self.width as usize;
			let x_end = if self.width - x > w {x + w} else {self.width} as usize;
			let y_end = if self.height - y > h {y + h} else {self.height} as usize;
			let x = x as usize;
			let y = y as usize;
			for cy in y..y_end
			{
				for cx in x..x_end
				{
					if self.lookup[cx + cy * stride].is_some() {return false;}
				}
			}
			true
		}
		else {self.lookup[(x as usize) + (y as usize) * (self.width as usize)].is_none()}
	}
	
	pub fn get(&self, x: u16, y: u16) -> Result<Option<&Placement>, PosError>
	{
		if x >= self.width || y >= self.height
		{
			return Err(PosError{x, y, w: self.width, h: self.height});
		}
		if self.blocks.len() == 0 {return Ok(None);}
		let pos = (x as usize) + (y as usize) * (self.width as usize);
		match self.lookup[pos]
		{
			None => Ok(None),
			Some(idx) => Ok(Some(&self.blocks[idx])),
		}
	}
	
	pub fn get_mut(&mut self, x: u16, y: u16) -> Result<Option<&mut Placement>, PosError>
	{
		if x >= self.width || y >= self.height
		{
			return Err(PosError{x, y, w: self.width, h: self.height});
		}
		if self.blocks.len() == 0 {return Ok(None);}
		let pos = (x as usize) + (y as usize) * (self.width as usize);
		match self.lookup[pos]
		{
			None => Ok(None),
			Some(idx) => Ok(Some(&mut self.blocks[idx])),
		}
	}
	
	fn swap_remove(&mut self, idx: usize) -> Placement
	{
		// swap_remove not only avoids moves in self.blocks but also reduces the lookup changes we have to do
		let prev = self.blocks.swap_remove(idx);
		self.fill_lookup(prev.pos.0 as usize, prev.pos.1 as usize, prev.block.get_size() as usize, None);
		if idx < self.blocks.len()
		{
			// fix the swapped block's lookup entries
			let swapped = &self.blocks[idx];
			self.fill_lookup(swapped.pos.0 as usize, swapped.pos.1 as usize, swapped.block.get_size() as usize, Some(idx));
		}
		prev
	}
	
	fn fill_lookup(&mut self, x: usize, y: usize, sz: usize, val: Option<usize>)
	{
		if self.lookup.len() == 0
		{
			self.lookup.resize((self.width as usize) * (self.height as usize), None);
		}
		if sz > 1
		{
			let off = ((sz - 1) / 2) as usize;
			let (x0, y0) = (x - off, y - off);
			for dy in 0..sz
			{
				for dx in 0..sz
				{
					self.lookup[(x0 + dx) + (y0 + dy) * (self.width as usize)] = val;
				}
			}
		}
		else {self.lookup[x + y * (self.width as usize)] = val;}
	}
	
	pub fn set(&mut self, x: u16, y: u16, block: &'static Block, data: DynData, rot: Rotation) -> Result<&Placement, PlaceError>
	{
		let sz = block.get_size() as u16;
		let off = (sz - 1) / 2;
		if x < off || y < off
		{
			return Err(PlaceError::Bounds{x, y, sz: block.get_size(), w: self.width, h: self.height});
		}
		if self.width - x < sz - off || self.height - y < sz - off
		{
			return Err(PlaceError::Bounds{x, y, sz: block.get_size(), w: self.width, h: self.height});
		}
		if self.is_region_empty(x - off, y - off, sz, sz)
		{
			let idx = self.blocks.len();
			let state = block.deserialize_state(data)?;
			self.blocks.push(Placement{pos: GridPos(x, y), block, state, rot});
			self.fill_lookup(x as usize, y as usize, block.get_size() as usize, Some(idx));
			Ok(&self.blocks[idx])
		}
		else {Err(PlaceError::Overlap{x, y})}
	}
	
	pub fn replace(&mut self, x: u16, y: u16, block: &'static Block, data: DynData, rot: Rotation, collect: bool)
		-> Result<Option<Vec<Placement>>, PlaceError>
	{
		let sz = block.get_size() as u16;
		let off = (sz - 1) / 2;
		if x < off || y < off
		{
			return Err(PlaceError::Bounds{x, y, sz: block.get_size(), w: self.width, h: self.height});
		}
		if self.width - x < sz - off || self.height - y < sz - off
		{
			return Err(PlaceError::Bounds{x, y, sz: block.get_size(), w: self.width, h: self.height});
		}
		if sz > 1
		{
			let mut result = if collect {Some(Vec::new())} else {None};
			// remove all blocks in the region
			for dy in 0..(sz as usize)
			{
				for dx in 0..(sz as usize)
				{
					if let Some(idx) = self.lookup[(x as usize + dx) + (y as usize + dy) * (self.width as usize)]
					{
						let prev = self.swap_remove(idx);
						if let Some(ref mut v) = result {v.push(prev);}
					}
				}
			}
			let idx = self.blocks.len();
			let state = block.deserialize_state(data)?;
			self.blocks.push(Placement{pos: GridPos(x, y), block, state, rot});
			self.fill_lookup(x as usize, y as usize, sz as usize, Some(idx));
			Ok(result)
		}
		else
		{
			let pos = (x as usize) + (y as usize) * (self.width as usize);
			match self.lookup[pos]
			{
				None =>
				{
					let idx = self.blocks.len();
					let state = block.deserialize_state(data)?;
					self.blocks.push(Placement{pos: GridPos(x, y), block, state, rot});
					self.lookup[pos] = Some(idx);
					Ok(if collect {Some(Vec::new())} else {None})
				},
				Some(idx) =>
				{
					let state = block.deserialize_state(data)?;
					let prev = std::mem::replace(&mut self.blocks[idx], Placement{pos: GridPos(x, y), block, state, rot});
					self.fill_lookup(prev.pos.0 as usize, prev.pos.1 as usize, prev.block.get_size() as usize, None);
					self.fill_lookup(x as usize, y as usize, sz as usize, Some(idx));
					Ok(if collect {Some(vec![prev])} else {None})
				}
			}
		}
	}
	
	pub fn take(&mut self, x: u16, y: u16) -> Result<Option<Placement>, PosError>
	{
		if x >= self.width || y >= self.height
		{
			return Err(PosError{x, y, w: self.width, h: self.height});
		}
		if self.blocks.len() > 0
		{
			let pos = (x as usize) + (y as usize) * (self.width as usize);
			match self.lookup[pos]
			{
				None => Ok(None),
				Some(idx) => Ok(Some(self.swap_remove(idx))),
			}
		}
		else {Ok(None)}
	}
	
	fn rebuild_lookup(&mut self)
	{
		self.lookup.clear();
		if self.blocks.len() > 0
		{
			self.lookup.resize((self.width as usize) * (self.height as usize), None);
			for (i, curr) in self.blocks.iter().enumerate()
			{
				let sz = curr.block.get_size() as usize;
				let x = curr.pos.0 as usize - (sz - 1) / 2;
				let y = curr.pos.1 as usize - (sz - 1) / 2;
				if sz > 1
				{
					for dy in 0..sz
					{
						for dx in 0..sz
						{
							self.lookup[(x + dx) + (y + dy) * (self.width as usize)] = Some(i);
						}
					}
				}
				else {self.lookup[x + y * (self.width as usize)] = Some(i);}
			}
		}
	}
	
	pub fn mirror(&mut self, horizontally: bool, vertically: bool)
	{
		if !self.blocks.is_empty() && (horizontally || vertically)
		{
			for curr in self.blocks.iter_mut()
			{
				// because position is the bottom left corner (which changes during mirroring)
				let shift = curr.block.get_size() as u16 - 1;
				if horizontally {curr.pos.0 = self.width - 1 - curr.pos.0 - shift;}
				if vertically {curr.pos.1 = self.height - 1 - curr.pos.1 - shift;}
				if !curr.block.is_symmetric() {curr.rot.mirror(horizontally, vertically);}
			}
			self.rebuild_lookup();
		}
	}
	
	pub fn rotate(&mut self, clockwise: bool)
	{
		let w = self.width;
		let h = self.height;
		self.width = h;
		self.height = w;
		if !self.blocks.is_empty()
		{
			for curr in self.blocks.iter_mut()
			{
				let x = curr.pos.0;
				let y = curr.pos.1;
				// because position is the bottom left corner (which changes during rotation)
				let shift = curr.block.get_size() as u16 - 1;
				if clockwise
				{
					curr.pos.0 = y;
					curr.pos.1 = w - 1 - x - shift;
				}
				else
				{
					curr.pos.0 = h - 1 - y - shift;
					curr.pos.1 = x;
				}
				if !curr.block.is_symmetric() {curr.rot.rotate(clockwise);}
			}
			self.rebuild_lookup();
		}
	}
	
	pub fn rotate_180(&mut self)
	{
		self.mirror(true, true);
	}
	
	pub fn pos_iter(&self) -> PosIter
	{
		PosIter{x: 0, y: 0, w: self.width, h: self.height}
	}
	
	pub fn block_iter<'s>(&'s self) -> Iter<'s, Placement>
	{
		self.blocks.iter()
	}
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum NewError
{
	Width(u16),
	Height(u16),
}

impl fmt::Display for NewError
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
	{
		match self
		{
			NewError::Width(w) => write!(f, "Invalid schematic width ({w})"),
			NewError::Height(h) => write!(f, "Invalid schematic height ({h})"),
		}
	}
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct PosError
{
	pub x: u16,
	pub y: u16,
	pub w: u16,
	pub h: u16,
}

impl fmt::Display for PosError
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
	{
		write!(f, "Position {x} / {y} out of bounds {w} / {h}", x = self.x, y = self.y, w = self.w, h = self.h)
	}
}

#[derive(Debug)]
pub enum PlaceError
{
	Bounds{x: u16, y: u16, sz: u8, w: u16, h: u16},
	Overlap{x: u16, y: u16},
	Deserialize(block::DeserializeError),
}

impl From<block::DeserializeError> for PlaceError
{
	fn from(value: block::DeserializeError) -> Self
	{
		PlaceError::Deserialize(value)
	}
}

impl fmt::Display for PlaceError
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
	{
		match self
		{
			PlaceError::Bounds{x, y, sz, w, h} => write!(f, "Block placement {x} / {y} (size {sz}) within {w} / {h}"),
			PlaceError::Overlap{x, y} => write!(f, "Overlapping an existing block at {x} / {y}"),
			PlaceError::Deserialize(e) => e.fmt(f),
		}
	}
}

impl Error for PlaceError
{
	fn source(&self) -> Option<&(dyn Error + 'static)>
	{
		match self
		{
			PlaceError::Deserialize(e) => Some(e),
			_ => None,
		}
	}
}

impl fmt::Display for Schematic
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
	{
		/*
		Because characters are about twice as tall as they are wide, two are used to represent a single block.
		Each block has a single letter to describe what it is + an optional rotation.
		For size-1 blocks, that's "*]" for symmetric and "*>", "*^", "*<", "*v" for rotations.
		Larger blocks are formed using pipes, slashes and minuses to form a border, which is filled with spaces.
		Then, the letter is placed inside followed by the rotation (if any).
		*/
		
		// find unique letters for each block, more common blocks pick first
		let mut name_cnt = HashMap::<&str, u16>::new();
		for p in self.blocks.iter()
		{
			match name_cnt.entry(p.block.get_name())
			{
				Entry::Occupied(mut e) => *e.get_mut() += 1,
				Entry::Vacant(e) => {e.insert(1);},
			}
		}
		// only needed the map for counting
		let mut name_cnt = Vec::from_iter(name_cnt);
		name_cnt.sort_by(|l, r| r.1.cmp(&l.1));
		// set for control characters, space, b'*', DEL and b">^<v]/|\\-"
		let mut used = [0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8, 0x01u8, 0xA4u8, 0x00u8, 0x50u8, 0x00u8, 0x00u8, 0x00u8, 0x70u8, 0x00u8, 0x00u8, 0x40u8, 0x90u8];
		let mut types = HashMap::<&str, char>::new();
		for &(name, _) in name_cnt.iter()
		{
			let mut found = false;
			for c in name.chars()
			{
				if c > ' ' && c <= '~'
				{
					let upper = c.to_ascii_uppercase() as usize;
					let lower = c.to_ascii_lowercase() as usize;
					if used[upper >> 3] & (1 << (upper & 7)) == 0
					{
						found = true;
						used[upper >> 3] |= 1 << (upper & 7);
						types.insert(name, unsafe{char::from_u32_unchecked(upper as u32)});
						break;
					}
					if lower != upper && used[lower >> 3] & (1 << (lower & 7)) == 0
					{
						found = true;
						used[lower >> 3] |= 1 << (lower & 7);
						types.insert(name, unsafe{char::from_u32_unchecked(lower as u32)});
						break;
					}
				}
			}
			if !found
			{
				// just take whatever symbol's still free (avoids collisions with letters)
				match used.iter().enumerate().find(|(_, &v)| v != u8::MAX)
				{
					// there's no more free symbols... how? use b'*' instead for all of them (reserved)
					None => {types.insert(name, '*');},
					Some((i, v)) =>
					{
						let idx = i + v.trailing_ones() as usize;
						used[idx >> 3] |= 1 << (idx & 7);
						types.insert(name, unsafe{char::from_u32_unchecked(idx as u32)});
					},
				}
			}
		}
		
		// coordinates start in the bottom left, so y starts at self.height - 1
		if self.blocks.len() > 0
		{
			for y in (0..self.height as usize).rev()
			{
				let mut x = 0usize;
				while x < self.width as usize
				{
					if let Some(idx) = self.lookup[x + y * (self.width as usize)]
					{
						let Placement{pos, block, state: _, rot} = self.blocks[idx];
						let c = *types.get(block.get_name()).unwrap();
						match block.get_size() as usize
						{
							0 => unreachable!(),
							1 =>
							{
								f.write_char(c)?;
								match rot
								{
									_ if block.is_symmetric() => f.write_char(']')?,
									Rotation::Right => f.write_char('>')?,
									Rotation::Up => f.write_char('^')?,
									Rotation::Left => f.write_char('<')?,
									Rotation::Down => f.write_char('v')?,
								}
							},
							s =>
							{
								let y0 = pos.1 as usize - (s - 1) / 2;
								if y == y0 + (s - 1)
								{
									// top row, which looks like /---[...]---\
									f.write_char('/')?;
									if s == 2
									{
										// label & rotation are in this row
										f.write_char(c)?;
										match rot
										{
											_ if block.is_symmetric() => f.write_char('-')?,
											Rotation::Right => f.write_char('>')?,
											Rotation::Up => f.write_char('^')?,
											Rotation::Left => f.write_char('<')?,
											Rotation::Down => f.write_char('v')?,
										}
									}
									else
									{
										// label & rotation are not in this row
										for _ in 0..(2 * s - 2)
										{
											f.write_char('-')?;
										}
									}
									f.write_char('\\')?;
								}
								else if y == y0
								{
									// bottom row, which looks like \---[...]---/
									f.write_char('\\')?;
									for _ in 0..(2 * s - 2)
									{
										f.write_char('-')?;
									}
									f.write_char('/')?;
								}
								else if s > 2 && y == y0 + s / 2
								{
									// middle row with label
									f.write_char('|')?;
									for cx in 0..(2 * s - 2)
									{
										if cx == s - 2 {f.write_char(c)?;}
										else if cx == s - 1
										{
											match rot
											{
												_ if block.is_symmetric() => f.write_char(' ')?,
												Rotation::Right => f.write_char('>')?,
												Rotation::Up => f.write_char('^')?,
												Rotation::Left => f.write_char('<')?,
												Rotation::Down => f.write_char('v')?,
											}
										}
										else {f.write_char(' ')?;}
									}
									f.write_char('|')?;
								}
								else
								{
									// middle row, which looks like |   [...]   |
									f.write_char('|')?;
									for _ in 0..(2 * s - 2)
									{
										f.write_char(' ')?;
									}
									f.write_char('|')?;
								}
							},
						}
						x += block.get_size() as usize;
					}
					else
					{
						f.write_str("  ")?;
						x += 1;
					}
				}
				writeln!(f)?;
			}
			// print the letters assigned to blocks
			for (k, _) in name_cnt
			{
				let v = *types.get(k).unwrap();
				write!(f, "\n({v}) {k}")?;
			}
		}
		else {write!(f, "<empty {} * {}>", self.width, self.height)?;}
		Ok(())
	}
}

const SCHEMATIC_HEADER: u32 = ((b'm' as u32) << 24) | ((b's' as u32) << 16) | ((b'c' as u32) << 8) | (b'h' as u32);

pub struct SchematicSerializer<'l>(pub &'l BlockRegistry<'static>);

impl<'l> Serializer<Schematic> for SchematicSerializer<'l>
{
	type ReadError = ReadError;
	type WriteError = WriteError;
	
	fn deserialize(&mut self, buff: &mut DataRead<'_>) -> Result<Schematic, Self::ReadError>
	{
		let hdr = buff.read_u32()?;
		if hdr != SCHEMATIC_HEADER {return Err(ReadError::Header(hdr));}
		let version = buff.read_u8()?;
		if version > 1 {return Err(ReadError::Version(version));}
		let mut dec = Decompress::new(true);
		let mut raw = Vec::<u8>::new();
		raw.reserve(1024);
		loop
		{
			let t_in = dec.total_in();
			let t_out = dec.total_out();
			let res = dec.decompress_vec(buff.data, &mut raw, FlushDecompress::Finish)?;
			if dec.total_in() > t_in
			{
				// we have to advance input every time, decompress_vec only knows the output position
				buff.data = &buff.data[(dec.total_in() - t_in) as usize..];
			}
			match res
			{
				// there's no more input (and the flush mode says so), we need to reserve additional space
				Status::Ok | Status::BufError => (),
				// input was already at the end, so this is referring to the output
				Status::StreamEnd => break,
			}
			if dec.total_in() == t_in && dec.total_out() == t_out
			{
				// protect against looping forever
				return Err(ReadError::DecompressStall);
			}
			raw.reserve(1024);
		}
		assert_eq!(dec.total_out() as usize, raw.len());
		let mut rbuff = DataRead::new(&raw);
		let w = rbuff.read_i16()?;
		let h = rbuff.read_i16()?;
		if w < 0 || h < 0 || w as u16 > MAX_DIMENSION || h as u16 > MAX_DIMENSION
		{
			return Err(ReadError::Dimensions(w, h));
		}
		let mut schematic = Schematic::new(w as u16, h as u16);
		for _ in 0..rbuff.read_u8()?
		{
			let key = rbuff.read_utf()?;
			let value = rbuff.read_utf()?;
			schematic.tags.insert(key.to_owned(), value.to_owned());
		}
		let num_table = rbuff.read_i8()?;
		if num_table < 0
		{
			return Err(ReadError::TableSize(num_table));
		}
		let mut block_table = Vec::<&'static Block>::new();
		block_table.reserve(num_table as usize);
		for _ in 0..num_table
		{
			let name = rbuff.read_utf()?;
			match self.0.get(name)
			{
				None => return Err(ReadError::NoSuchBlock(name.to_owned())),
				Some(b) => block_table.push(b),
			}
		}
		let num_blocks = rbuff.read_i32()?;
		if num_blocks < 0 || num_blocks as u32 > MAX_BLOCKS
		{
			return Err(ReadError::BlockCount(num_blocks));
		}
		for _ in 0..num_blocks
		{
			let idx = rbuff.read_i8()?;
			if idx < 0 || idx as usize >= block_table.len()
			{
				return Err(ReadError::BlockIndex(idx, block_table.len()));
			}
			let pos = GridPos::from(rbuff.read_u32()?);
			let block = block_table[idx as usize];
			let config = if version < 1
			{
				block.data_from_i32(rbuff.read_i32()?)
			}
			else {DynSerializer.deserialize(&mut rbuff)?};
			let rot = Rotation::from(rbuff.read_u8()?);
			schematic.set(pos.0, pos.1, block, config, rot)?;
		}
		Ok(schematic)
	}
	
	fn serialize(&mut self, buff: &mut DataWrite<'_>, data: &Schematic) -> Result<(), Self::WriteError>
	{
		// write the header first just in case
		buff.write_u32(SCHEMATIC_HEADER)?;
		buff.write_u8(1)?;
		
		let mut rbuff = DataWrite::new();
		// don't have to check dimensions because they're already limited to MAX_DIMENSION
		rbuff.write_i16(data.width as i16)?;
		rbuff.write_i16(data.height as i16)?;
		if data.tags.len() > u8::MAX as usize
		{
			return Err(WriteError::TagCount(data.tags.len()));
		}
		rbuff.write_u8(data.tags.len() as u8)?;
		for (k, v) in data.tags.iter()
		{
			rbuff.write_utf(k)?;
			rbuff.write_utf(v)?;
		}
		// use string keys here to avoid issues with different block refs with the same name
		let mut block_map = HashMap::<&str, u32>::new();
		let mut block_table = Vec::<&str>::new();
		for curr in data.blocks.iter()
		{
			match block_map.entry(curr.block.get_name())
			{
				Entry::Vacant(e) =>
				{
					e.insert(block_table.len() as u32);
					block_table.push(curr.block.get_name());
				},
				_ => (),
			}
		}
		if block_table.len() > i8::MAX as usize
		{
			return Err(WriteError::TableSize(block_table.len()));
		}
		// else: implies contents are also valid i8 (they're strictly less than the map length)
		rbuff.write_i8(block_table.len() as i8)?;
		for &name in block_table.iter()
		{
			rbuff.write_utf(name)?;
		}
		// don't have to check data.blocks.len() because dimensions don't allow exceeding MAX_BLOCKS
		rbuff.write_i32(data.blocks.len() as i32)?;
		let mut num = 0;
		for curr in data.blocks.iter()
		{
			rbuff.write_i8(block_map[curr.block.get_name()] as i8)?;
			rbuff.write_u32(u32::from(curr.pos))?;
			let data = match curr.state
			{
				None => DynData::Empty,
				Some(ref s) => curr.block.serialize_state(s.as_ref())?,
			};
			DynSerializer.serialize(&mut rbuff, &data)?;
			rbuff.write_u8(curr.rot.into())?;
			num += 1;
		}
		assert_eq!(num, data.blocks.len());
		
		// compress into the provided buffer
		let raw = match rbuff.data
		{
			data::WriteBuff::Vec(v) => v,
			_ => unreachable!("write buffer not owned"),
		};
		let mut comp = Compress::new(Compression::default(), true);
		// compress the immediate buffer into a temp buffer to copy it to buff? no thanks
		match buff.data
		{
			data::WriteBuff::Ref{raw: ref mut dst, ref mut pos} =>
			{
				match comp.compress(&raw, &mut dst[*pos..], FlushCompress::Finish)?
				{
					// there's no more input (and the flush mode says so), but we can't resize the output
					Status::Ok | Status::BufError => return Err(WriteError::CompressEof(raw.len() - comp.total_in() as usize)),
					Status::StreamEnd => (),
				}
			},
			data::WriteBuff::Vec(ref mut dst) =>
			{
				let mut input = raw.as_ref();
				dst.reserve(1024);
				loop
				{
					let t_in = comp.total_in();
					let t_out = comp.total_out();
					let res = comp.compress_vec(input, dst, FlushCompress::Finish)?;
					if comp.total_in() > t_in
					{
						// we have to advance input every time, compress_vec only knows the output position
						input = &input[(comp.total_in() - t_in) as usize..];
					}
					match res
					{
						// there's no more input (and the flush mode says so), we need to reserve additional space
						Status::Ok | Status::BufError => (),
						// input was already at the end, so this is referring to the output
						Status::StreamEnd => break,
					}
					if comp.total_in() == t_in && comp.total_out() == t_out
					{
						// protect against looping forever
						return Err(WriteError::CompressStall);
					}
					dst.reserve(1024);
				}
			},
		}
		assert_eq!(comp.total_in() as usize, raw.len());
		Ok(())
	}
}

#[derive(Debug)]
pub enum ReadError
{
	Read(data::ReadError),
	Header(u32),
	Version(u8),
	Decompress(DecompressError),
	DecompressStall,
	Dimensions(i16, i16),
	TableSize(i8),
	NoSuchBlock(String),
	BlockCount(i32),
	BlockIndex(i8, usize),
	BlockState(dynamic::ReadError),
	Placement(PlaceError),
}

impl From<data::ReadError> for ReadError
{
	fn from(value: data::ReadError) -> Self
	{
		Self::Read(value)
	}
}

impl From<DecompressError> for ReadError
{
	fn from(value: DecompressError) -> Self
	{
		Self::Decompress(value)
	}
}

impl From<dynamic::ReadError> for ReadError
{
	fn from(value: dynamic::ReadError) -> Self
	{
		Self::BlockState(value)
	}
}

impl From<PlaceError> for ReadError
{
	fn from(value: PlaceError) -> Self
	{
		Self::Placement(value)
	}
}

impl fmt::Display for ReadError
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
	{
		match self
		{
			ReadError::Read(..) => write!(f, "Failed to read data from buffer"),
			ReadError::Header(hdr) => write!(f, "Incorrect header ({hdr:08X})"),
			ReadError::Version(ver) => write!(f, "Unsupported version ({ver})"),
			ReadError::Decompress(e) => e.fmt(f),
			ReadError::DecompressStall => write!(f, "Decompressor stalled before completion"),
			ReadError::Dimensions(w, h) => write!(f, "Invalid schematic dimensions ({w} * {h})"),
			ReadError::TableSize(cnt) => write!(f, "Invalid block table size ({cnt})"),
			ReadError::NoSuchBlock(name) => write!(f, "Unknown block {name:?}"),
			ReadError::BlockCount(cnt) => write!(f, "Invalid total block count ({cnt})"),
			ReadError::BlockIndex(idx, cnt) => write!(f, "Invalid block index ({idx} / {cnt})"),
			ReadError::BlockState(..) => write!(f, "Failed to read block state"),
			ReadError::Placement(e) => e.fmt(f),
		}
	}
}

#[derive(Debug)]
pub enum WriteError
{
	Write(data::WriteError),
	TagCount(usize),
	TableSize(usize),
	StateSerialize(block::SerializeError),
	BlockState(dynamic::WriteError),
	Compress(CompressError),
	CompressEof(usize),
	CompressStall,
}

impl From<data::WriteError> for WriteError
{
	fn from(value: data::WriteError) -> Self
	{
		Self::Write(value)
	}
}

impl From<block::SerializeError> for WriteError
{
	fn from(value: block::SerializeError) -> Self
	{
		Self::StateSerialize(value)
	}
}

impl From<CompressError> for WriteError
{
	fn from(value: CompressError) -> Self
	{
		Self::Compress(value)
	}
}

impl From<dynamic::WriteError> for WriteError
{
	fn from(value: dynamic::WriteError) -> Self
	{
		Self::BlockState(value)
	}
}

impl fmt::Display for WriteError
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
	{
		match self
		{
			WriteError::Write(..) => write!(f, "Failed to write data to buffer"),
			WriteError::TagCount(cnt) => write!(f, "Invalid tag count ({cnt})"),
			WriteError::TableSize(cnt) => write!(f, "Invalid block table size ({cnt})"),
			WriteError::StateSerialize(e) => e.fmt(f),
			WriteError::BlockState(..) => write!(f, "Failed to write block state"),
			WriteError::Compress(e) => e.fmt(f),
			WriteError::CompressEof(remain) => write!(f, "Compression overflow with {remain} bytes of input remaining"),
			WriteError::CompressStall => write!(f, "Compressor stalled before completion"),
		}
	}
}

impl<'l> SchematicSerializer<'l>
{
	pub fn deserialize_base64(&mut self, data: &str) -> Result<Schematic, R64Error>
	{
		let mut buff = Vec::<u8>::new();
		buff.resize(data.len() / 4 * 3 + 1, 0);
		let n_out = base64::decode(data.as_bytes(), buff.as_mut())?;
		Ok(self.deserialize(&mut DataRead::new(&buff[..n_out]))?)
	}
	
	pub fn serialize_base64(&mut self, data: &Schematic) -> Result<String, W64Error>
	{
		let mut buff = DataWrite::new();
		self.serialize(&mut buff, data)?;
		let buff = buff.get_written();
		// round up because of padding
		let required = 4 * (buff.len() / 3 + if buff.len() % 3 != 0 {1} else {0});
		let mut text = Vec::<u8>::new();
		text.resize(required, 0);
		let n_out = base64::encode(buff, text.as_mut())?;
		// trailing zeros are valid UTF8, but not valid base64
		assert_eq!(n_out, text.len());
		// SAFETY: base64 encoding outputs pure ASCII (see base64::CHARS)
		Ok(unsafe{String::from_utf8_unchecked(text)})
	}
}

#[derive(Debug)]
pub enum R64Error
{
	Base64(base64::DecodeError),
	Content(ReadError),
}

impl From<base64::DecodeError> for R64Error
{
	fn from(value: base64::DecodeError) -> Self
	{
		Self::Base64(value)
	}
}

impl From<ReadError> for R64Error
{
	fn from(value: ReadError) -> Self
	{
		Self::Content(value)
	}
}

impl fmt::Display for R64Error
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
	{
		match self
		{
			R64Error::Base64(e) => e.fmt(f),
			R64Error::Content(e) => e.fmt(f),
		}
	}
}

#[derive(Debug)]
pub enum W64Error
{
	Base64(base64::EncodeError),
	Content(WriteError),
}

impl From<base64::EncodeError> for W64Error
{
	fn from(value: base64::EncodeError) -> Self
	{
		Self::Base64(value)
	}
}

impl From<WriteError> for W64Error
{
	fn from(value: WriteError) -> Self
	{
		Self::Content(value)
	}
}

impl fmt::Display for W64Error
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
	{
		match self
		{
			W64Error::Base64(e) => e.fmt(f),
			W64Error::Content(e) => e.fmt(f),
		}
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
