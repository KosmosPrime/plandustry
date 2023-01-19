use std::error::Error;
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
	
	pub fn is_base(&self) -> bool
	{
		self.0 < 6
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

impl fmt::Display for TryFromU16Error
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
	{
		write!(f, "No content of type Team for value {}", self.0)
	}
}

impl Error for TryFromU16Error {}

const TEAM_NAMES: &str = include_str!("../res/team_names.txt");

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
	
	fn get_name(&self) -> &'static str
	{
		match self.0
		{
			0 => "derelict",
			1 => "sharded",
			2 => "crux",
			3 => "malis",
			4 => "green",
			5 => "blue",
			// dark magic: offsets manually computed, then rely on the format "...|team#{i}|..."
			i @ 6..=9 =>
			{
				// length: 7 ("team#" (5) + 1 digit + "|" (1))
				let s = 0 + ((i - 6) as usize) * 7;
				&TEAM_NAMES[s..s + 6] // exclude the trailing "|"
			},
			i @ 10..=99 =>
			{
				// length: 8 ("team#" (5) + 2 digits + "|" (1))
				let s = 28 + ((i - 10) as usize) * 8;
				&TEAM_NAMES[s..s + 7] // exclude the trailing "|"
			},
			i @ 100..=255 =>
			{
				// length: 9 ("team#" (5) + 3 digits + "|" (1))
				let s = 748 + ((i - 100) as usize) * 9;
				&TEAM_NAMES[s..s + 8] // exclude the trailing "|"
			},
		}
	}
}

pub const DERELICT: Team = Team(0);
pub const SHARDED: Team = Team(1);
pub const CRUX: Team = Team(2);
pub const MALIS: Team = Team(3);
pub const GREEN: Team = Team(4);
pub const BLUE: Team = Team(5);
