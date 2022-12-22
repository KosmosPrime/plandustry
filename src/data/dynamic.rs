use crate::data::{self, DataRead, DataWrite, GridPos, Serializer, Team};
use crate::logic::LogicField;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnitCommand
{
	Attack, Rally, Idle
}

impl UnitCommand
{
	pub fn of(id: u8) -> Option<Self>
	{
		match id
		{
			0 => Some(Self::Attack),
			1 => Some(Self::Rally),
			2 => Some(Self::Idle),
			_ => None
		}
	}
}

impl TryFrom<u8> for UnitCommand
{
	type Error = u8;
	
	fn try_from(value: u8) -> Result<Self, Self::Error>
	{
		match Self::of(value)
		{
			None => Err(value),
			Some(c) => Ok(c),
		}
	}
}

impl From<UnitCommand> for u8
{
	fn from(value: UnitCommand) -> u8
	{
		value as u8
	}
}

#[derive(Clone, Debug, PartialEq)]
pub enum DynData
{
	Empty,
	Int(i32),
	Long(i64),
	Float(f32),
	String(Option<String>),
	Content(u8, u16),
	IntArray(Vec<i32>),
	Point2(i32, i32),
	Point2Array(Vec<(i16, i16)>),
	TechNode(u8, u16),
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
			5 => Ok(DynData::Content(buff.read_u8()?, buff.read_u16()?)),
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
			9 => Ok(DynData::TechNode(buff.read_u8()?, buff.read_u16()?)),
			10 => Ok(DynData::Boolean(buff.read_bool()?)),
			11 => Ok(DynData::Double(buff.read_f64()?)),
			12 => Ok(DynData::Building(GridPos::from(buff.read_u32()?))),
			13 =>
			{
				let id = buff.read_u8()?;
				match LogicField::of(id)
				{
					None => Err(ReadError::LogicField(id)),
					Some(f) => Ok(DynData::LogicField(f)),
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
				match UnitCommand::of(id)
				{
					None => Err(ReadError::UnitCommand(id)),
					Some(f) => Ok(DynData::UnitCommand(f)),
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
				buff.write_u8(*ty)?;
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
				buff.write_u8(*ty)?;
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
	IntArrayLen(i16),
	Point2ArrayLen(i8),
	LogicField(u8),
	ByteArrayLen(i32),
	UnitCommand(u8),
	BoolArrayLen(i32),
	Vec2ArrayLen(i16),
}

impl From<data::ReadError> for ReadError
{
	fn from(err: data::ReadError) -> Self
	{
		ReadError::Underlying(err)
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
		WriteError::Underlying(err)
	}
}

#[cfg(test)]
mod test
{
	use super::*;
	use crate::data::{TEAM_CRUX, TEAM_DERELICT, TEAM_SHARDED};
	
	fn compare_vec2(lhs: (f32, f32), rhs: (f32, f32)) -> bool
	{
		// these probably have no reason to be non-normal values but thight bounds are nice for consistency
		f32::to_bits(lhs.0) == f32::to_bits(rhs.0) && f32::to_bits(lhs.1) == f32::to_bits(rhs.1)
	}
	
	fn compare(lhs: &DynData, rhs: &DynData) -> bool
	{
		match (lhs, rhs)
		{
			(DynData::Empty, DynData::Empty) => true,
			(DynData::Int(l), DynData::Int(r)) => l == r,
			(DynData::Long(l), DynData::Long(r)) => l == r,
			// normally this would be bad, but we want to get the f32 back as it was written
			(DynData::Float(l), DynData::Float(r)) => f32::to_bits(*l) == f32::to_bits(*r),
			(DynData::String(l), DynData::String(r)) => l == r,
			(DynData::Content(l0, l1), DynData::Content(r0, r1)) => l0 == r0 && l1 == r1,
			(DynData::IntArray(l), DynData::IntArray(r)) => l == r,
			(DynData::Point2(lx, ly), DynData::Point2(rx, ry)) => lx == rx && ly == ry,
			(DynData::Point2Array(l), DynData::Point2Array(r)) => l == r,
			(DynData::TechNode(l0, l1), DynData::TechNode(r0, r1)) => l0 == r0 && l1 == r1,
			(DynData::Boolean(l), DynData::Boolean(r)) => l == r,
			// normally this would be bad, but we want to get the f32 back as it was written
			(DynData::Double(l), DynData::Double(r)) => f64::to_bits(*l) == f64::to_bits(*r),
			(DynData::Building(l), DynData::Building(r)) => l == r,
			(DynData::LogicField(l), DynData::LogicField(r)) => l == r,
			(DynData::ByteArray(l), DynData::ByteArray(r)) => l == r,
			(DynData::UnitCommand(l), DynData::UnitCommand(r)) => l == r,
			(DynData::BoolArray(l), DynData::BoolArray(r)) => l == r,
			(DynData::Unit(l), DynData::Unit(r)) => l == r,
			(DynData::Vec2Array(l), DynData::Vec2Array(r)) => l.iter().zip(r.iter()).all(|(&(lx, ly), &(rx, ry))| compare_vec2((lx, ly), (rx, ry))),
			(DynData::Vec2(lx, ly), DynData::Vec2(rx, ry)) => compare_vec2((*lx, *ly), (*rx, *ry)),
			(DynData::Team(l), DynData::Team(r)) => l == r,
			_ => false,
		}
	}
	
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
						Ok(read) => assert!(compare(original, &read), "serialization of {original:?} became {read:?}"),
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
	make_dyn_test!(reparse_content, DynData::Content(0, 12345), DynData::Content(13, 25431));
	make_dyn_test!(reparse_int_array, DynData::IntArray(vec![581923, 2147483647, -1047563850]), DynData::IntArray(vec![1902864703]));
	make_dyn_test!(reparse_point2, DynData::Point2(17, 0), DynData::Point2(234, -345), DynData::Point2(-2147483648, -1));
	make_dyn_test!(reparse_point2_array, DynData::Point2Array(vec![(44, 55), (-33, 66), (-22, -77)]), DynData::Point2Array(vec![(22, -88)]));
	make_dyn_test!(reparse_technode, DynData::TechNode(0, 12345), DynData::TechNode(13, 25431));
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
	make_dyn_test!(reparse_team, DynData::Team(TEAM_SHARDED), DynData::Team(TEAM_CRUX), DynData::Team(TEAM_DERELICT));
}
