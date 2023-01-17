use crate::content::content_enum;

content_enum!
{
	pub enum Type / Modifier for u16 | TryFromU16Error
	{
		None => "none",
		Burning => "burning",
		Freezing => "freezing",
		Unmoving => "unmoving",
		Slow => "slow",
		Wet => "wet",
		Muddy => "muddy",
		Melting => "melting",
		Sapped => "sapped",
		Electrified => "electrified",
		SporeSlowed => "spore-slowed",
		Tarred => "tarred",
		Overdrive => "overdrive",
		Overclock => "overclock",
		Shielded => "shielded",
		Boss => "boss",
		Shocked => "shocked",
		Blasted => "blasted",
		Corroded => "corroded",
		Disarmed => "disarmed",
		Invincible => "invincible",
	}
}
