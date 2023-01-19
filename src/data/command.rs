use crate::content::numeric_enum;

numeric_enum!
{
	pub enum UnitCommand for u8 | TryFromU8Error
	{
		Attack, Rally, Idle
	}
}
