use std::fmt;

use crate::content::content_enum;

pub mod storage;

content_enum!
{
	pub enum Type / Item for u16 | TryFromU16Error
	{
		Copper => "copper",
		Lead => "lead",
		Metaglass => "metaglass",
		Graphite => "graphite",
		Sand => "sand",
		Coal => "coal",
		Titanium => "titanium",
		Thorium => "thorium",
		Scrap => "scrap",
		Silicon => "silicon",
		Plastanium => "plastanium",
		PhaseFabric => "phase-fabric",
		SurgeAlloy => "surge-alloy",
		SporePod => "spore-pod",
		BlastCompound => "blast-compound",
		Pyratite => "pyratite",
		Beryllium => "beryllium",
		Tungsten => "tungsten",
		Oxide => "oxide",
		Carbide => "carbide",
		FissileMatter => "fissile-matter",
		DormantCyst => "dormant-cyst",
	}
}

impl fmt::Display for Type
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
	{
		match self
		{
			Self::Copper => f.write_str("Copper"),
			Self::Lead => f.write_str("Lead"),
			Self::Metaglass => f.write_str("Metaglass"),
			Self::Graphite => f.write_str("Graphite"),
			Self::Sand => f.write_str("Sand"),
			Self::Coal => f.write_str("Coal"),
			Self::Titanium => f.write_str("Titanium"),
			Self::Thorium => f.write_str("Thorium"),
			Self::Scrap => f.write_str("Scrap"),
			Self::Silicon => f.write_str("Silicon"),
			Self::Plastanium => f.write_str("Plastanium"),
			Self::PhaseFabric => f.write_str("Phase Fabric"),
			Self::SurgeAlloy => f.write_str("Surge Alloy"),
			Self::SporePod => f.write_str("Spore Pod"),
			Self::BlastCompound => f.write_str("Blast Compound"),
			Self::Pyratite => f.write_str("Pyratite"),
			Self::Beryllium => f.write_str("Beryllium"),
			Self::Tungsten => f.write_str("Tungsten"),
			Self::Oxide => f.write_str("Oxide"),
			Self::Carbide => f.write_str("Carbide"),
			Self::FissileMatter => f.write_str("Fissile Matter"),
			Self::DormantCyst => f.write_str("Dormant Cyst"),
		}
	}
}
