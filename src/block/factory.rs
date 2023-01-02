use crate::block::make_register;
use crate::block::simple::SimpleBlock;

make_register!
(
	GRAPHITE_PRESS: "graphite-press" => SimpleBlock::new(2, true);
	MULTI_PRESS: "multi-press" => SimpleBlock::new(3, true);
	SILICON_SMELTER: "silicon-smelter" => SimpleBlock::new(2, true);
	SILICON_CRUCIBLE: "silicon-crucible" => SimpleBlock::new(3, true);
	KILN: "kiln" => SimpleBlock::new(2, true);
	PLASTANIUM_COMPRESSOR: "plastanium-compressor" => SimpleBlock::new(2, true);
	PHASE_WEAVER: "phase-weaver" => SimpleBlock::new(2, true);
	SURGE_SMELTER: "surge-smelter" => SimpleBlock::new(3, true);
	CRYOFLUID_MIXER: "cryofluid-mixer" => SimpleBlock::new(2, true);
	PYRATITE_MIXER: "pyratite-mixer" => SimpleBlock::new(2, true);
	BLAST_MIXER: "blast-mixer" => SimpleBlock::new(2, true);
	MELTER: "melter" => SimpleBlock::new(1, true);
	SEPARATOR: "separator" => SimpleBlock::new(2, true);
	DISASSEMBLER: "disassembler" => SimpleBlock::new(3, true);
	SPORE_PRESS: "spore-press" => SimpleBlock::new(2, true);
	PULVERIZER: "pulverizer" => SimpleBlock::new(1, true);
	COAL_CENTRIFUGE: "coal-centrifuge" => SimpleBlock::new(2, true);
	INCINERATOR: "incinerator" => SimpleBlock::new(1, true);
	// sandbox only
	HEAT_SOURCE: "heat-source" => SimpleBlock::new(1, false);
);
