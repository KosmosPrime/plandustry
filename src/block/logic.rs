use std::any::{Any, type_name};
use std::borrow::Cow;

use flate2::{Compress, Compression, Decompress, FlushCompress, FlushDecompress, Status};

use crate::block::{BlockLogic, make_register};
use crate::block::simple::{SimpleBlock, state_impl};
use crate::data::{DataRead, DataWrite};
use crate::data::dynamic::DynData;

make_register!
(
	MESSAGE: "message" => MessageLogic;
	SWITCH: "switch" => SwitchLogic;
	MICRO_PROCESSOR: "micro-processor" => ProcessorLogic{size: 1};
	LOGIC_PROCESSOR: "logic-processor" => ProcessorLogic{size: 2};
	HYPER_PROCESSOR: "hyper-processor" => ProcessorLogic{size: 3};
	MEMORY_CELL: "memory-cell" => SimpleBlock::new(1, true);
	MEMORY_BANK: "memory-bank" => SimpleBlock::new(2, true);
	LOGIC_DISPLAY: "logic-display" => SimpleBlock::new(3, true);
	LARGE_LOGIC_DISPLAY: "large-logic-display" => SimpleBlock::new(6, true);
);

pub struct MessageLogic;

impl MessageLogic
{
	state_impl!(pub String);
}

impl BlockLogic for MessageLogic
{
	fn get_size(&self) -> u8
	{
		1
	}
	
	fn is_symmetric(&self) -> bool
	{
		true
	}
	
	fn data_from_i32(&self, _: i32) -> DynData
	{
		DynData::Empty
	}
	
	fn deserialize_state(&self, data: DynData) -> Option<Box<dyn Any>>
	{
		match data
		{
			DynData::Empty | DynData::String(None) => Some(Self::create_state(String::new())),
			DynData::String(Some(s)) => Some(Self::create_state(s)),
			_ => panic!("{} cannot use data of {:?}", type_name::<Self>(), data.get_type()),
		}
	}
	
	fn clone_state(&self, state: &dyn Any) -> Box<dyn Any>
	{
		Box::new(Self::get_state(state).clone())
	}
	
	fn serialize_state(&self, state: &dyn Any) -> DynData
	{
		DynData::String(Some(Self::get_state(state).clone()))
	}
}

pub struct SwitchLogic;

impl SwitchLogic
{
	state_impl!(pub bool);
}

impl BlockLogic for SwitchLogic
{
	fn get_size(&self) -> u8
	{
		1
	}
	
	fn is_symmetric(&self) -> bool
	{
		true
	}
	
	fn data_from_i32(&self, _: i32) -> DynData
	{
		DynData::Empty
	}
	
	fn deserialize_state(&self, data: DynData) -> Option<Box<dyn Any>>
	{
		match data
		{
			DynData::Empty => Some(Self::create_state(true)),
			DynData::Boolean(enabled) => Some(Self::create_state(enabled)),
			_ => panic!("{} cannot use data of {:?}", type_name::<Self>(), data.get_type()),
		}
	}
	
	fn clone_state(&self, state: &dyn Any) -> Box<dyn Any>
	{
		Box::new(Self::get_state(state).clone())
	}
	
	fn serialize_state(&self, state: &dyn Any) -> DynData
	{
		DynData::Boolean(*Self::get_state(state))
	}
}

pub struct ProcessorLogic
{
	size: u8,
}

impl ProcessorLogic
{
	state_impl!(pub ProcessorState);
}

impl BlockLogic for ProcessorLogic
{
	fn get_size(&self) -> u8
	{
		self.size
	}
	
	fn is_symmetric(&self) -> bool
	{
		true
	}
	
	fn data_from_i32(&self, _: i32) -> DynData
	{
		DynData::Empty
	}
	
	fn deserialize_state(&self, data: DynData) -> Option<Box<dyn Any>>
	{
		match data
		{
			DynData::Empty => Some(Self::create_state(ProcessorState::new())),
			DynData::ByteArray(arr) =>
			{
				let mut input = arr.as_ref();
				let mut dec = Decompress::new(true);
				let mut raw = Vec::<u8>::new();
				raw.reserve(1024);
				loop
				{
					let t_in = dec.total_in();
					let t_out = dec.total_out();
					let res = dec.decompress_vec(input, &mut raw, FlushDecompress::Finish).unwrap();
					if dec.total_in() > t_in
					{
						// we have to advance input every time, decompress_vec only knows the output position
						input = &input[(dec.total_in() - t_in) as usize..];
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
						panic!("decompressor stalled");
					}
					raw.reserve(1024);
				}
				let mut buff = DataRead::new(&raw);
				let ver = buff.read_u8().unwrap();
				if ver != 1
				{
					panic!("unknown version {ver}");
				}
				
				let code_len = buff.read_i32().unwrap();
				if code_len < 0 || code_len > 500 * 1024
				{
					panic!("invalid code length ({code_len})");
				}
				let mut code = Vec::<u8>::new();
				code.resize(code_len as usize, 0);
				buff.read_bytes(&mut code).unwrap();
				let code = String::from_utf8(code).unwrap();
				let link_cnt = buff.read_i32().unwrap();
				if link_cnt < 0
				{
					panic!("link count is negative ({link_cnt})");
				}
				let mut links = Vec::<ProcessorLink>::new();
				links.reserve(link_cnt as usize);
				for _ in 0..link_cnt
				{
					let name = buff.read_utf().unwrap();
					let x = buff.read_i16().unwrap();
					let y = buff.read_i16().unwrap();
					links.push(ProcessorLink{name: String::from(name), x, y});
				}
				Some(Self::create_state(ProcessorState{code, links}))
			},
			_ => panic!("{} cannot use data of {:?}", type_name::<Self>(), data.get_type()),
		}
	}
	
	fn clone_state(&self, state: &dyn Any) -> Box<dyn Any>
	{
		Box::new(Self::get_state(state).clone())
	}
	
	fn serialize_state(&self, state: &dyn Any) -> DynData
	{
		let state = Self::get_state(state);
		let mut rbuff = DataWrite::new();
		rbuff.write_u8(1).unwrap();
		if state.code.len() > i32::MAX as usize
		{
			panic!("code too long ({})", state.code.len());
		}
		rbuff.write_i32(state.code.len() as i32).unwrap();
		rbuff.write_bytes(state.code.as_bytes()).unwrap();
		if state.links.len() > i32::MAX as usize
		{
			panic!("too many links ({})", state.links.len());
		}
		rbuff.write_i32(state.links.len() as i32).unwrap();
		for link in state.links.iter()
		{
			rbuff.write_utf(&link.name).unwrap();
			rbuff.write_i16(link.x).unwrap();
			rbuff.write_i16(link.y).unwrap();
		}
		let mut input = rbuff.get_written();
		let mut comp = Compress::new(Compression::default(), true);
		let mut dst = Vec::<u8>::new();
		dst.reserve(1024);
		loop
		{
			let t_in = comp.total_in();
			let t_out = comp.total_out();
			let res = comp.compress_vec(input, &mut dst, FlushCompress::Finish).unwrap();
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
				panic!("compressor stalled");
			}
			dst.reserve(1024);
		}
		DynData::ByteArray(dst)
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessorLink
{
	name: String,
	x: i16,
	y: i16,
}

impl ProcessorLink
{
	pub fn new(name: Cow<'_, str>, x: i16, y: i16) -> Self
	{
		if name.len() > u16::MAX as usize
		{
			panic!("name too long ({})", name.len());
		}
		Self{name: name.into_owned(), x, y}
	}
	
	pub fn get_name(&self) -> &str
	{
		&self.name
	}
	
	pub fn get_pos(&self) -> (i16, i16)
	{
		(self.x, self.y)
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessorState
{
	code: String,
	links: Vec<ProcessorLink>
}

impl ProcessorState
{
	pub fn new() -> Self
	{
		Self{code: String::new(), links: Vec::new()}
	}
	
	pub fn get_code(&self) -> &str
	{
		&self.code
	}
	
	pub fn set_code(&mut self, code: Cow<'_, str>) -> Result<(), CodeError>
	{
		let as_str = &code as &str;
		if as_str.len() > 500 * 1024
		{
			return Err(CodeError::TooLong(as_str.len()));
		}
		match code
		{
			Cow::Borrowed(s) =>
			{
				self.code.clear();
				self.code.push_str(s);
			},
			Cow::Owned(s) => self.code = s,
		}
		Ok(())
	}
	
	pub fn get_links(&self) -> &[ProcessorLink]
	{
		&self.links
	}
	
	pub fn create_link(&mut self, mut name: String, x: i16, y: i16) -> Result<&ProcessorLink, CreateError>
	{
		if name.len() > u16::MAX as usize
		{
			return Err(CreateError::NameLength{len: name.len()})
		}
		for curr in self.links.iter()
		{
			if &name == &curr.name
			{
				return Err(CreateError::DuplicateName{name});
			}
			if x == curr.x && y == curr.y
			{
				name.clear();
				name.push_str(&curr.name);
				return Err(CreateError::DuplicatePos{name, x, y});
			}
		}
		let idx = self.links.len();
		self.links.push(ProcessorLink{name, x, y});
		Ok(&self.links[idx])
	}
	
	pub fn add_link(&mut self, link: ProcessorLink) -> Result<&ProcessorLink, CreateError>
	{
		self.create_link(link.name, link.x, link.y)
	}
	
	pub fn remove_link(&mut self, idx: usize) -> Option<ProcessorLink>
	{
		if idx < self.links.len()
		{
			Some(self.links.remove(idx))
		}
		else {None}
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CodeError
{
	TooLong(usize),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CreateError
{
	NameLength{len: usize},
	DuplicateName{name: String},
	DuplicatePos{name: String, x: i16, y: i16},
}
