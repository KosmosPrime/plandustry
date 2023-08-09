use crate::content::numeric_enum;

numeric_enum!
{
	pub enum UnitStrategy for u8 | TryFromU8Error
	{
		Attack, Rally, Idle
	}
}

numeric_enum!
{
	pub enum UnitCommand for u16 | TryFromU16Error
	{
		Move, Repair, Rebuild, Assist, Mine, Boost
	}
}
