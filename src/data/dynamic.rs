use std::error::Error;
use std::fmt;

use crate::content;
use crate::data::{self, DataRead, DataWrite, GridPos, Serializer};
use crate::data::command::{self, UnitCommand};
use crate::logic::LogicField;
use crate::team::Team;

#[derive(Clone, Debug, PartialEq)]
pub enum DynData
{
	Empty,
	Int(i32),
	Long(i64),
	Float(f32),
	String(Option<String>),
	Content(content::Type, u16),
	IntArray(Vec<i32>),
	Point2(i32, i32),
	Point2Array(Vec<(i16, i16)>),
	TechNode(content::Type, u16),
	Boolean(bool),
	Double(f64),
	Building(GridPos),
	LogicField(LogicField),
	ByteArray(Vec<u8>),
	UnitCommand(UnitCommand),
	BoolArray(Vec<bool>),
	Unit(u32),
	Vec2Array(Vec<(f32, f32)>),
	Vec2(f32, f32),
	Team(Team),
}

impl DynData
{
	pub fn get_type(&self) -> DynType
	{
		match self
		{
			Self::Empty => DynType::Empty,
			Self::Int(..) => DynType::Int,
			Self::Long(..) => DynType::Long,
			Self::Float(..) => DynType::Float,
			Self::String(..) => DynType::String,
			Self::Content(..) => DynType::Content,
			Self::IntArray(..) => DynType::IntArray,
			Self::Point2(..) => DynType::Point2,
			Self::Point2Array(..) => DynType::Point2Array,
			Self::TechNode(..) => DynType::TechNode,
			Self::Boolean(..) => DynType::Boolean,
			Self::Double(..) => DynType::Double,
			Self::Building(..) => DynType::Building,
			Self::LogicField(..) => DynType::LogicField,
			Self::ByteArray(..) => DynType::ByteArray,
			Self::UnitCommand(..) => DynType::UnitCommand,
			Self::BoolArray(..) => DynType::BoolArray,
			Self::Unit(..) => DynType::Unit,
			Self::Vec2Array(..) => DynType::Vec2Array,
			Self::Vec2(..) => DynType::Vec2,
			Self::Team(..) => DynType::Team,
		}
	}
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DynType
{
	Empty, Int, Long, Float, String, Content, IntArray, Point2, Point2Array, TechNode, Boolean, Double, Building,
	LogicField, ByteArray, UnitCommand, BoolArray, Unit, Vec2Array, Vec2, Team,
}

pub struct DynSerializer;

impl Serializer<DynData> for DynSerializer
{
	type ReadError = ReadError;
	type WriteError = WriteError;
	
	fn deserialize(&mut self, buff: &mut DataRead<'_>) -> Result<DynData, Self::ReadError>
	{
		match buff.read_u8()?
		{
			0 => Ok(DynData::Empty),
			1 => Ok(DynData::Int(buff.read_i32()?)),
			2 => Ok(DynData::Long(buff.read_i64()?)),
			3 => Ok(DynData::Float(buff.read_f32()?)),
			4 =>
			{
				if buff.read_bool()?
				{
					Ok(DynData::String(Some(String::from(buff.read_utf()?))))
				}
				else {Ok(DynData::String(None))}
			},
			5 => Ok(DynData::Content(content::Type::try_from(buff.read_u8()?)?, buff.read_u16()?)),
			6 =>
			{
				let len = buff.read_i16()?;
				if len < 0
				{
					return Err(ReadError::IntArrayLen(len));
				}
				let mut result = Vec::<i32>::new();
				result.reserve(len as usize);
				for _ in 0..len
				{
					result.push(buff.read_i32()?);
				}
				Ok(DynData::IntArray(result))
			},
			7 => Ok(DynData::Point2(buff.read_i32()?, buff.read_i32()?)),
			8 =>
			{
				let len = buff.read_i8()?;
				if len < 0
				{
					return Err(ReadError::Point2ArrayLen(len));
				}
				let mut result = Vec::<(i16, i16)>::new();
				result.reserve(len as usize);
				for _ in 0..len
				{
					let pt = buff.read_i32()?;
					result.push(((pt >> 16) as i16, pt as i16));
				}
				Ok(DynData::Point2Array(result))
			},
			9 => Ok(DynData::TechNode(content::Type::try_from(buff.read_u8()?)?, buff.read_u16()?)),
			10 => Ok(DynData::Boolean(buff.read_bool()?)),
			11 => Ok(DynData::Double(buff.read_f64()?)),
			12 => Ok(DynData::Building(GridPos::from(buff.read_u32()?))),
			13 =>
			{
				let id = buff.read_u8()?;
				match LogicField::try_from(id)
				{
					Ok(f) => Ok(DynData::LogicField(f)),
					Err(..) => Err(ReadError::LogicField(id)),
				}
			},
			14 =>
			{
				let len = buff.read_i32()?;
				if len < 0
				{
					return Err(ReadError::ByteArrayLen(len));
				}
				let mut result = Vec::<u8>::new();
				buff.read_vec(&mut result, len as usize)?;
				Ok(DynData::ByteArray(result))
			},
			15 =>
			{
				let id = buff.read_u8()?;
				match UnitCommand::try_from(id)
				{
					Ok(f) => Ok(DynData::UnitCommand(f)),
					Err(e) => Err(ReadError::UnitCommand(e)),
				}
			},
			16 =>
			{
				let len = buff.read_i32()?;
				if len < 0
				{
					return Err(ReadError::BoolArrayLen(len));
				}
				let mut result = Vec::<bool>::new();
				result.reserve(len as usize);
				for _ in 0..len
				{
					result.push(buff.read_bool()?);
				}
				Ok(DynData::BoolArray(result))
			},
			17 => Ok(DynData::Unit(buff.read_u32()?)),
			18 =>
			{
				let len = buff.read_i16()?;
				if len < 0
				{
					return Err(ReadError::Vec2ArrayLen(len));
				}
				let mut result = Vec::<(f32, f32)>::new();
				result.reserve(len as usize);
				for _ in 0..len
				{
					result.push((buff.read_f32()?, buff.read_f32()?));
				}
				Ok(DynData::Vec2Array(result))
			},
			19 => Ok(DynData::Vec2(buff.read_f32()?, buff.read_f32()?)),
			20 => Ok(DynData::Team(Team::of(buff.read_u8()?))),
			id => Err(ReadError::Type(id)),
		}
	}
	
	fn serialize(&mut self, buff: &mut DataWrite<'_>, data: &DynData) -> Result<(), Self::WriteError>
	{
		match data
		{
			DynData::Empty =>
			{
				buff.write_u8(0)?;
				Ok(())
			},
			DynData::Int(val) =>
			{
				buff.write_u8(1)?;
				buff.write_i32(*val)?;
				Ok(())
			},
			DynData::Long(val) =>
			{
				buff.write_u8(2)?;
				buff.write_i64(*val)?;
				Ok(())
			},
			DynData::Float(val) =>
			{
				buff.write_u8(3)?;
				buff.write_f32(*val)?;
				Ok(())
			},
			DynData::String(opt) =>
			{
				buff.write_u8(4)?;
				match opt
				{
					None => buff.write_bool(false)?,
					Some(s) =>
					{
						buff.write_bool(true)?;
						buff.write_utf(s)?
					},
				}
				Ok(())
			},
			DynData::Content(ty, id) =>
			{
				buff.write_u8(5)?;
				buff.write_u8((*ty).into())?;
				buff.write_u16(*id)?;
				Ok(())
			},
			DynData::IntArray(arr) =>
			{
				if arr.len() > i16::MAX as usize
				{
					return Err(WriteError::IntArrayLen(arr.len()));
				}
				buff.write_u8(6)?;
				buff.write_i16(arr.len() as i16)?;
				for &v in arr.iter()
				{
					buff.write_i32(v)?;
				}
				Ok(())
			},
			DynData::Point2(x, y) =>
			{
				buff.write_u8(7)?;
				buff.write_i32(*x)?;
				buff.write_i32(*y)?;
				Ok(())
			},
			DynData::Point2Array(arr) =>
			{
				if arr.len() > i8::MAX as usize
				{
					return Err(WriteError::Point2ArrayLen(arr.len()));
				}
				buff.write_u8(8)?;
				buff.write_i8(arr.len() as i8)?;
				for &(x, y) in arr.iter()
				{
					buff.write_i32(((x as i32) << 16) | ((y as i32) & 0xFFFF))?;
				}
				Ok(())
			},
			DynData::TechNode(ty, id) =>
			{
				buff.write_u8(9)?;
				buff.write_u8((*ty).into())?;
				buff.write_u16(*id)?;
				Ok(())
			},
			DynData::Boolean(val) =>
			{
				buff.write_u8(10)?;
				buff.write_bool(*val)?;
				Ok(())
			},
			DynData::Double(val) =>
			{
				buff.write_u8(11)?;
				buff.write_f64(*val)?;
				Ok(())
			},
			DynData::Building(pos) =>
			{
				buff.write_u8(12)?;
				buff.write_u32(u32::from(*pos))?;
				Ok(())
			},
			DynData::LogicField(fld) =>
			{
				buff.write_u8(13)?;
				buff.write_u8(u8::from(*fld))?;
				Ok(())
			},
			DynData::ByteArray(arr) =>
			{
				if arr.len() > i32::MAX as usize
				{
					return Err(WriteError::ByteArrayLen(arr.len()));
				}
				buff.write_u8(14)?;
				buff.write_i32(arr.len() as i32)?;
				buff.write_bytes(arr)?;
				Ok(())
			},
			DynData::UnitCommand(cmd) =>
			{
				buff.write_u8(15)?;
				buff.write_u8(u8::from(*cmd))?;
				Ok(())
			},
			DynData::BoolArray(arr) =>
			{
				if arr.len() > i32::MAX as usize
				{
					return Err(WriteError::BoolArrayLen(arr.len()));
				}
				buff.write_u8(16)?;
				buff.write_i32(arr.len() as i32)?;
				for &b in arr.iter()
				{
					buff.write_bool(b)?;
				}
				Ok(())
			},
			DynData::Unit(id) =>
			{
				buff.write_u8(17)?;
				buff.write_u32(*id)?;
				Ok(())
			},
			DynData::Vec2Array(arr) =>
			{
				if arr.len() > i16::MAX as usize
				{
					return Err(WriteError::Vec2ArrayLen(arr.len()));
				}
				buff.write_u8(18)?;
				buff.write_i16(arr.len() as i16)?;
				for &(x, y) in arr.iter()
				{
					buff.write_f32(x)?;
					buff.write_f32(y)?;
				}
				Ok(())
			},
			DynData::Vec2(x, y) =>
			{
				buff.write_u8(19)?;
				buff.write_f32(*x)?;
				buff.write_f32(*y)?;
				Ok(())
			},
			DynData::Team(team) =>
			{
				buff.write_u8(20)?;
				buff.write_u8(u8::from(*team))?;
				Ok(())
			},
		}
	}
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReadError
{
	Underlying(data::ReadError),
	Type(u8),
	ContentType(content::TryFromU8Error),
	IntArrayLen(i16),
	Point2ArrayLen(i8),
	LogicField(u8),
	ByteArrayLen(i32),
	UnitCommand(command::TryFromU8Error),
	BoolArrayLen(i32),
	Vec2ArrayLen(i16),
}

impl From<data::ReadError> for ReadError
{
	fn from(err: data::ReadError) -> Self
	{
		Self::Underlying(err)
	}
}

impl From<content::TryFromU8Error> for ReadError
{
	fn from(err: content::TryFromU8Error) -> Self
	{
		Self::ContentType(err)
	}
}

impl From<command::TryFromU8Error> for ReadError
{
	fn from(err: command::TryFromU8Error) -> Self
	{
		Self::UnitCommand(err)
	}
}

impl fmt::Display for ReadError
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
	{
		match self
		{
			Self::Underlying(..) => f.write_str("failed to read from buffer"),
			Self::Type(id) => write!(f, "invalid dynamic data type ({id})"),
			Self::ContentType(..) => f.write_str("content type not found"),
			Self::IntArrayLen(len) => write!(f, "integer array too long ({len})"),
			Self::Point2ArrayLen(len) => write!(f, "point2 array too long ({len})"),
			Self::LogicField(id) => write!(f, "invalid logic field ({id})"),
			Self::ByteArrayLen(len) => write!(f, "byte array too long ({len})"),
			Self::UnitCommand(..) => f.write_str("unit command not found"),
			Self::BoolArrayLen(len) => write!(f, "boolean array too long ({len})"),
			Self::Vec2ArrayLen(len) => write!(f, "vec2 array too long ({len})"),
		}
	}
}

impl Error for ReadError
{
	fn source(&self) -> Option<&(dyn Error + 'static)>
	{
		match self
		{
			Self::Underlying(e) => Some(e),
			_ => None,
		}
	}
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WriteError
{
	Underlying(data::WriteError),
	IntArrayLen(usize),
	Point2ArrayLen(usize),
	ByteArrayLen(usize),
	BoolArrayLen(usize),
	Vec2ArrayLen(usize),
}

impl From<data::WriteError> for WriteError
{
	fn from(err: data::WriteError) -> Self
	{
		Self::Underlying(err)
	}
}

impl fmt::Display for WriteError
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
	{
		match self
		{
			Self::Underlying(..) => f.write_str("failed to write to buffer"),
			Self::IntArrayLen(len) => write!(f, "integer array too long ({len})"),
			Self::Point2ArrayLen(len) => write!(f, "point2 array too long ({len})"),
			Self::ByteArrayLen(len) => write!(f, "byte array too long ({len})"),
			Self::BoolArrayLen(len) => write!(f, "boolean array too long ({len})"),
			Self::Vec2ArrayLen(len) => write!(f, "vec2 array too long ({len})"),
		}
	}
}

impl Error for WriteError
{
	fn source(&self) -> Option<&(dyn Error + 'static)>
	{
		match self
		{
			Self::Underlying(e) => Some(e),
			_ => None,
		}
	}
}

#[cfg(test)]
mod test
{
	use super::*;
	use crate::team::{CRUX, DERELICT, SHARDED};
	
	macro_rules!_zero
	{
		($tt:tt) => {0usize};
	}
	
	macro_rules!make_dyn_test
	{
		($name:ident, $($val:expr),+) =>
		{
			#[test]
			fn $name()
			{
				let input = [$($val),+];
				let mut positions = [$(_zero!($val)),+];
				let mut writer = DataWrite::new();
				for (i, d) in input.iter().enumerate()
				{
					assert_eq!(DynSerializer.serialize(&mut writer, d), Ok(()));
					positions[i] = writer.get_written().len();
				}
				let written = writer.get_written();
				let end = written.len();
				let mut reader = DataRead::new(written);
				for (i, original) in input.iter().enumerate()
				{
					match DynSerializer.deserialize(&mut reader)
					{
						Ok(read) => assert_eq!(*original, read, "serialization of {original:?} became {read:?}"),
						e => assert!(false, "could not re-read {original:?} (at {i}), got {e:?}"),
					}
					let expect = end - reader.data.len();
					let before = if i > 0 {positions[i - 1]} else {0};
					assert_eq!(expect, positions[i], "uneven deserialization of {original:?} ({} vs {})", expect - before, positions[i] - before);
				}
			}
		};
	}
	
	make_dyn_test!(reparse_empty, DynData::Empty, DynData::Empty, DynData::Empty);
	make_dyn_test!(reparse_int, DynData::Int(581923), DynData::Int(2147483647), DynData::Int(-1047563850));
	make_dyn_test!(reparse_long, DynData::Long(11295882949812), DynData::Long(-5222358074010407789));
	make_dyn_test!(reparse_float, DynData::Float(3.14159265), DynData::Float(f32::INFINITY), DynData::Float(f32::EPSILON), DynData::Float(f32::NAN));
	make_dyn_test!(reparse_string, DynData::String(None), DynData::String(Some("hello \u{10FE03}".to_string())), DynData::String(Some("".to_string())));
	make_dyn_test!(reparse_content, DynData::Content(content::Type::Item, 12345), DynData::Content(content::Type::Planet, 25431));
	make_dyn_test!(reparse_int_array, DynData::IntArray(vec![581923, 2147483647, -1047563850]), DynData::IntArray(vec![1902864703]));
	make_dyn_test!(reparse_point2, DynData::Point2(17, 0), DynData::Point2(234, -345), DynData::Point2(-2147483648, -1));
	make_dyn_test!(reparse_point2_array, DynData::Point2Array(vec![(44, 55), (-33, 66), (-22, -77)]), DynData::Point2Array(vec![(22, -88)]));
	make_dyn_test!(reparse_technode, DynData::TechNode(content::Type::Item, 12345), DynData::TechNode(content::Type::Planet, 25431));
	make_dyn_test!(reparse_boolean, DynData::Boolean(false), DynData::Boolean(true), DynData::Boolean(false));
	make_dyn_test!(reparse_double, DynData::Double(2.718281828459045), DynData::Double(-f64::INFINITY), DynData::Double(f64::NAN));
	make_dyn_test!(reparse_building, DynData::Building(GridPos(10, 0)), DynData::Building(GridPos(4444, 0xFE98)));
	make_dyn_test!(reparse_logic, DynData::LogicField(LogicField::Enabled), DynData::LogicField(LogicField::Shoot), DynData::LogicField(LogicField::Color));
	make_dyn_test!(reparse_byte_array, DynData::ByteArray(b"c\x00nstruct \xADditio\nal pylons".to_vec()), DynData::ByteArray(b"\x00\x01\xFE\xFF".to_vec()));
	make_dyn_test!(reparse_unit_command, DynData::UnitCommand(UnitCommand::Idle), DynData::UnitCommand(UnitCommand::Rally));
	make_dyn_test!(reparse_bool_array, DynData::BoolArray(vec![true, true, true, false, true, false, true]), DynData::BoolArray(vec![false, true]));
	make_dyn_test!(reparse_unit, DynData::Unit(0), DynData::Unit(2147483647));
	make_dyn_test!(reparse_vec2_array, DynData::Vec2Array(vec![(4.4, 5.5), (-3.3, 6.6), (-2.2, -7.7)]), DynData::Vec2Array(vec![(2.2, -8.8)]));
	make_dyn_test!(reparse_vec2, DynData::Vec2(1.5, 9.1234), DynData::Vec2(-0.0, -17.0), DynData::Vec2(-10.7, 3.8));
	make_dyn_test!(reparse_team, DynData::Team(SHARDED), DynData::Team(CRUX), DynData::Team(DERELICT));
}
