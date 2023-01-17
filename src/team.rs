use std::fmt;

use crate::content::{Content, Type};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Team(u8);

impl Team
{
	pub fn of(id: u8) -> Self
	{
		Self(id)
	}
	
	pub fn get_id(&self) -> u8
	{
		self.0
	}
	
	pub fn is_base(&self) -> bool
	{
		self.0 < 6
	}
	
	pub fn get_name(&self) -> Option<&'static str>
	{
		match self.0
		{
			0 => Some("derelict"),
			1 => Some("sharded"),
			2 => Some("crux"),
			3 => Some("malis"),
			4 => Some("green"),
			5 => Some("blue"),
			_ => None,
		}
	}
}

impl From<u8> for Team
{
	fn from(value: u8) -> Self
	{
		Team::of(value)
	}
}

impl TryFrom<u16> for Team
{
	type Error = TryFromU16Error;
	
	fn try_from(value: u16) -> Result<Self, Self::Error>
	{
		if value <= u8::MAX as u16 {Ok(Team(value as u8))}
		else {Err(TryFromU16Error(value))}
	}
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct TryFromU16Error(pub u16);

impl From<Team> for u8
{
	fn from(value: Team) -> Self
	{
		value.0
	}
}

impl From<Team> for u16
{
	fn from(value: Team) -> Self
	{
		value.0 as u16
	}
}

impl fmt::Display for Team
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
	{
		match self.0
		{
			0 => f.write_str("Derelict"),
			1 => f.write_str("Sharded"),
			2 => f.write_str("Crux"),
			3 => f.write_str("Malis"),
			4 => f.write_str("Green"),
			5 => f.write_str("Blue"),
			id => write!(f, "Team #{id}"),
		}
	}
}

impl Content for Team
{
	fn get_type(&self) -> Type
	{
		Type::Team
	}
	
	fn get_id(&self) -> u16
	{
		self.0 as u16
	}
	
	fn get_name(&self) -> &str
	{
		match self.0
		{
			0 => "derelict",
			1 => "sharded",
			2 => "crux",
			3 => "malis",
			4 => "green",
			5 => "blue",
			_ => "<custom>", // TODO
		}
	}
}

pub const DERELICT: Team = Team(0);
pub const SHARDED: Team = Team(1);
pub const CRUX: Team = Team(2);
pub const MALIS: Team = Team(3);
pub const GREEN: Team = Team(4);
pub const BLUE: Team = Team(5);
