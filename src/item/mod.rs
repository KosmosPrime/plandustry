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
