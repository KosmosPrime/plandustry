use crate::block::make_register;
use crate::block::simple::{cost, SimpleBlock};

make_register!
(
	GRAPHITE_PRESS: "graphite-press" => SimpleBlock::new(2, true, cost!(Copper: 75, Lead: 30));
	MULTI_PRESS: "multi-press" => SimpleBlock::new(3, true, cost!(Lead: 100, Graphite: 50, Titanium: 100, Silicon: 25));
	SILICON_SMELTER: "silicon-smelter" => SimpleBlock::new(2, true, cost!(Copper: 30, Lead: 25));
	SILICON_CRUCIBLE: "silicon-crucible" => SimpleBlock::new(3, true, cost!(Metaglass: 80, Titanium: 120, Silicon: 60, Plastanium: 35));
	KILN: "kiln" => SimpleBlock::new(2, true, cost!(Copper: 60, Lead: 30, Graphite: 30));
	PLASTANIUM_COMPRESSOR: "plastanium-compressor" => SimpleBlock::new(2, true, cost!(Lead: 115, Graphite: 60, Titanium: 80, Silicon: 80));
	PHASE_WEAVER: "phase-weaver" => SimpleBlock::new(2, true, cost!(Lead: 120, Thorium: 75, Silicon: 130));
	SURGE_SMELTER: "surge-smelter" => SimpleBlock::new(3, true, cost!(Lead: 80, Thorium: 70, Silicon: 80));
	CRYOFLUID_MIXER: "cryofluid-mixer" => SimpleBlock::new(2, true, cost!(Lead: 65, Thorium: 60, Silicon: 40));
	PYRATITE_MIXER: "pyratite-mixer" => SimpleBlock::new(2, true, cost!(Copper: 50, Lead: 25));
	BLAST_MIXER: "blast-mixer" => SimpleBlock::new(2, true, cost!(Lead: 30, Thorium: 20));
	MELTER: "melter" => SimpleBlock::new(1, true, cost!(Copper: 30, Lead: 35, Graphite: 45));
	SEPARATOR: "separator" => SimpleBlock::new(2, true, cost!(Copper: 30, Titanium: 25));
	DISASSEMBLER: "disassembler" => SimpleBlock::new(3, true, cost!(Titanium: 100, Thorium: 80, Silicon: 150, Plastanium: 40));
	SPORE_PRESS: "spore-press" => SimpleBlock::new(2, true, cost!(Lead: 35, Silicon: 30));
	PULVERIZER: "pulverizer" => SimpleBlock::new(1, true, cost!(Copper: 30, Lead: 25));
	COAL_CENTRIFUGE: "coal-centrifuge" => SimpleBlock::new(2, true, cost!(Lead: 30, Graphite: 40, Titanium: 20));
	INCINERATOR: "incinerator" => SimpleBlock::new(1, true, cost!(Lead: 15, Graphite: 5));
	// sandbox only
	HEAT_SOURCE: "heat-source" => SimpleBlock::new(1, false, &[]);
);
