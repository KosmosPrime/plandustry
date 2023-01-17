use std::fmt;

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

impl From<Team> for u8
{
	fn from(value: Team) -> Self
	{
		value.0
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

pub const DERELICT: Team = Team(0);
pub const SHARDED: Team = Team(1);
pub const CRUX: Team = Team(2);
pub const MALIS: Team = Team(3);
pub const GREEN: Team = Team(4);
pub const BLUE: Team = Team(5);
