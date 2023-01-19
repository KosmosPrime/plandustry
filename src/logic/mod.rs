use crate::content::numeric_enum;

numeric_enum!
{
	pub enum LogicField for u8 | TryFromU8Error
	{
		TotalItems, FirstItem, TotalLiquids, TotalPower, ItemCapacity, LiquidCapacity, PowerCapacity, PowerNetCapacity, PowerNetStored, PowerNetIn,
		PowerNetOut, Ammo, AmmoCapacity, Health, MaxHealth, Heat, Efficiency, Progress, Timescale, Rotation, PosX, PosY, ShootX, ShootY, Size, Dead, Range,
		Shooting, Boosting, MineX, MineY, Mining, Speed, Team, Type, Flag, Controlled, Controller, Name, PayloadCount, PayloadType, Enabled, Shoot, ShootP,
		Config, Color
	}
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

impl LogicField
{
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
