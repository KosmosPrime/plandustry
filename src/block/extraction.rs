use crate::block::make_register;
use crate::block::simple::SimpleBlock;

make_register!
(
	MECHANICAL_DRILL: "mechanical-drill" => SimpleBlock::new(2, true);
	PNEUMATIC_DRILL: "pneumatic-drill" => SimpleBlock::new(2, true);
	LASER_DRILL: "laser-drill" => SimpleBlock::new(3, true);
	BLAST_DRILL: "blast-drill" => SimpleBlock::new(4, true);
	WATER_EXTRACTOR: "water-extractor" => SimpleBlock::new(2, true);
	CULTIVATOR: "cultivator" => SimpleBlock::new(2, true);
	OIL_EXTRACTOR: "oil-extractor" => SimpleBlock::new(3, true);
);
