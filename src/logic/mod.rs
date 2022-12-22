#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LogicField
{
	TotalItems, FirstItem, TotalLiquids, TotalPower, ItemCapacity, LiquidCapacity, PowerCapacity, PowerNetCapacity, PowerNetStored, PowerNetIn, PowerNetOut,
	Ammo, AmmoCapacity, Health, MaxHealth, Heat, Efficiency, Progress, Timescale, Rotation, PosX, PosY, ShootX, ShootY, Size, Dead, Range, Shooting, Boosting,
	MineX, MineY, Mining, Speed, Team, Type, Flag, Controlled, Controller, Name, PayloadCount, PayloadType, Enabled, Shoot, ShootP, Config, Color
}

macro_rules!match_select
{
	($val:expr, $base:ty, $($name:ident),+) =>
	{
		match $val
		{
			$(<$base>::$name => true,)+
			_ => false,
		}
	};
}

macro_rules!map_from_enum
{
	($from:ident => $to:ty, $val:expr, $($name:ident),+) =>
	{
		{
			#![allow(dead_code, non_upper_case_globals)]
			$(const $name: $to = <$from>::$name as $to;)+
			match $val
			{
				$($name => Some(<$from>::$name),)+
				_ => None,
			}
		}
	};
}

impl LogicField
{
	pub fn of(value: u8) -> Option<Self>
	{
		map_from_enum!(LogicField => u8, value, TotalItems, FirstItem, TotalLiquids, TotalPower, ItemCapacity, LiquidCapacity, PowerCapacity, PowerNetCapacity,
			PowerNetStored, PowerNetIn, PowerNetOut, Ammo, AmmoCapacity, Health, MaxHealth, Heat, Efficiency, Progress, Timescale, Rotation, PosX, PosY,
			ShootX, ShootY, Size, Dead, Range, Shooting, Boosting, MineX, MineY, Mining, Speed, Team, Type, Flag, Controlled, Controller, Name, PayloadCount,
			PayloadType, Enabled, Shoot, ShootP, Config, Color)
	}
	
	pub fn is_readable(&self) -> bool
	{
		match_select!(self, LogicField, TotalItems, FirstItem, TotalLiquids, TotalPower, ItemCapacity, LiquidCapacity, PowerCapacity, PowerNetCapacity,
			PowerNetStored, PowerNetIn, PowerNetOut, Ammo, AmmoCapacity, Health, MaxHealth, Heat, Efficiency, Progress, Timescale, Rotation, PosX, PosY,
			ShootX, ShootY, Size, Dead, Range, Shooting, Boosting, MineX, MineY, Mining, Speed, Team, Type, Flag, Controlled, Controller, Name, PayloadCount,
			PayloadType, Enabled, Color)
	}
	
	pub fn is_writable(&self) -> bool
	{
		match_select!(self, LogicField, Enabled, Shoot, ShootP, Config, Color)
	}
}

impl TryFrom<u8> for LogicField
{
	type Error = u8;
	
	fn try_from(value: u8) -> Result<Self, Self::Error>
	{
		match Self::of(value)
		{
			None => Err(value),
			Some(f) => Ok(f),
		}
	}
}

impl From<LogicField> for u8
{
	fn from(value: LogicField) -> u8
	{
		value as u8
	}
}
