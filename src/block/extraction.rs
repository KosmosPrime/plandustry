use crate::block::make_register;
use crate::block::simple::{cost, SimpleBlock};

make_register!
(
	MECHANICAL_DRILL: "mechanical-drill" => SimpleBlock::new(2, true, cost!(Copper: 12));
	PNEUMATIC_DRILL: "pneumatic-drill" => SimpleBlock::new(2, true, cost!(Copper: 18, Graphite: 10));
	LASER_DRILL: "laser-drill" => SimpleBlock::new(3, true, cost!(Copper: 35, Graphite: 30, Titanium: 20, Silicon: 30));
	BLAST_DRILL: "blast-drill" => SimpleBlock::new(4, true, cost!(Copper: 65, Titanium: 50, Thorium: 75, Silicon: 60));
	WATER_EXTRACTOR: "water-extractor" => SimpleBlock::new(2, true, cost!(Copper: 30, Lead: 30, Metaglass: 30, Graphite: 30));
	CULTIVATOR: "cultivator" => SimpleBlock::new(2, true, cost!(Copper: 25, Lead: 25, Silicon: 10));
	OIL_EXTRACTOR: "oil-extractor" => SimpleBlock::new(3, true, cost!(Copper: 150, Lead: 115, Graphite: 175, Thorium: 115, Silicon: 75));
);
