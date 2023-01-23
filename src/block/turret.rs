use crate::block::make_register;
use crate::block::simple::{cost, SimpleBlock};

make_register!
(
	DUO: "duo" => SimpleBlock::new(1, true, cost!(Copper: 35));
	SCATTER: "scatter" => SimpleBlock::new(2, true, cost!(Copper: 85, Lead: 45));
	SCORCH: "scorch" => SimpleBlock::new(1, true, cost!(Copper: 25, Graphite: 22));
	HAIL: "hail" => SimpleBlock::new(1, true, cost!(Copper: 40, Graphite: 17));
	WAVE: "wave" => SimpleBlock::new(2, true, cost!(Copper: 25, Lead: 75, Metaglass: 45));
	LANCER: "lancer" => SimpleBlock::new(2, true, cost!(Copper: 60, Lead: 70, Titanium: 30, Silicon: 60));
	ARC: "arc" => SimpleBlock::new(1, true, cost!(Copper: 50, Lead: 50));
	PARALLAX: "parallax" => SimpleBlock::new(2, true, cost!(Graphite: 30, Titanium: 90, Silicon: 120));
	SWARMER: "swarmer" => SimpleBlock::new(2, true, cost!(Graphite: 35, Titanium: 35, Silicon: 30, Plastanium: 45));
	SALVO: "salvo" => SimpleBlock::new(2, true, cost!(Copper: 100, Graphite: 80, Titanium: 50));
	SEGMENT: "segment" => SimpleBlock::new(2, true, cost!(Titanium: 40, Thorium: 80, Silicon: 130, PhaseFabric: 40));
	TSUNAMI: "tsunami" => SimpleBlock::new(3, true, cost!(Lead: 400, Metaglass: 100, Titanium: 250, Thorium: 100));
	FUSE: "fuse" => SimpleBlock::new(3, true, cost!(Copper: 225, Graphite: 225, Thorium: 100));
	RIPPLE: "ripple" => SimpleBlock::new(3, true, cost!(Copper: 150, Graphite: 135, Titanium: 60));
	CYCLONE: "cyclone" => SimpleBlock::new(3, true, cost!(Copper: 200, Titanium: 125, Plastanium: 80));
	FORESHADOW: "foreshadow" => SimpleBlock::new(4, true, cost!(Copper: 1000, Metaglass: 600, Silicon: 600, Plastanium: 200, SurgeAlloy: 300));
	SPECTRE: "spectre" => SimpleBlock::new(4, true, cost!(Copper: 900, Graphite: 300, Thorium: 250, Plastanium: 175, SurgeAlloy: 250));
	MELTDOWN: "meltdown" => SimpleBlock::new(4, true, cost!(Copper: 1200, Lead: 350, Graphite: 300, Silicon: 325, SurgeAlloy: 325));
);
