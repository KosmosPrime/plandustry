use crate::block::make_register;
use crate::block::simple::{cost, SimpleBlock};

make_register!
(
    "graphite-press" => SimpleBlock::new(2, true, cost!(Copper: 75, Lead: 30));
    "multi-press" => SimpleBlock::new(3, true, cost!(Lead: 100, Graphite: 50, Titanium: 100, Silicon: 25));
    "silicon-smelter" => SimpleBlock::new(2, true, cost!(Copper: 30, Lead: 25));
    "silicon-crucible" => SimpleBlock::new(3, true, cost!(Metaglass: 80, Titanium: 120, Silicon: 60, Plastanium: 35));
    "kiln" => SimpleBlock::new(2, true, cost!(Copper: 60, Lead: 30, Graphite: 30));
    "plastanium-compressor" => SimpleBlock::new(2, true, cost!(Lead: 115, Graphite: 60, Titanium: 80, Silicon: 80));
    "phase-weaver" => SimpleBlock::new(2, true, cost!(Lead: 120, Thorium: 75, Silicon: 130));
    "surge-smelter" => SimpleBlock::new(3, true, cost!(Lead: 80, Thorium: 70, Silicon: 80));
    "cryofluid-mixer" => SimpleBlock::new(2, true, cost!(Lead: 65, Thorium: 60, Silicon: 40));
    "pyratite-mixer" => SimpleBlock::new(2, true, cost!(Copper: 50, Lead: 25));
    "blast-mixer" => SimpleBlock::new(2, true, cost!(Lead: 30, Thorium: 20));
    "melter" => SimpleBlock::new(1, true, cost!(Copper: 30, Lead: 35, Graphite: 45));
    "separator" => SimpleBlock::new(2, true, cost!(Copper: 30, Titanium: 25));
    "disassembler" => SimpleBlock::new(3, true, cost!(Titanium: 100, Thorium: 80, Silicon: 150, Plastanium: 40));
    "spore-press" => SimpleBlock::new(2, true, cost!(Lead: 35, Silicon: 30));
    "pulverizer" => SimpleBlock::new(1, true, cost!(Copper: 30, Lead: 25));
    "coal-centrifuge" => SimpleBlock::new(2, true, cost!(Lead: 30, Graphite: 40, Titanium: 20));
    "incinerator" => SimpleBlock::new(1, true, cost!(Lead: 15, Graphite: 5));
    // sandbox only
    "heat-source" => SimpleBlock::new(1, false, &[]);
);
