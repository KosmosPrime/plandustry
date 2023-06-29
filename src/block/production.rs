//! the industry part of mindustry
use crate::block::make_register;
use crate::block::simple::{cost, SimpleBlock};

make_register! {
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
    "silicon-arc-furnace" => SimpleBlock::new(3, true, cost!(Beryllium: 70, Graphite: 80));
    "electrolyzer" => SimpleBlock::new(3, true, cost!(Silicon: 50, Graphite: 40, Beryllium: 130, Tungsten: 80));
    "atmospheric-concentrator" => SimpleBlock::new(3, true, cost!(Oxide: 60, Beryllium: 180, Silicon: 150));
    "oxidation-chamber" => SimpleBlock::new(3, true, cost!(Tungsten: 120, Graphite: 80, Silicon: 100, Beryllium: 120));
    "electric-heater" => SimpleBlock::new(2, false, cost!(Tungsten: 30, Oxide: 30));
    "slag-heater" => SimpleBlock::new(3, false, cost!(Tungsten: 50, Oxide: 20, Beryllium: 20));
    "phase-heater" => SimpleBlock::new(2, false, cost!(Oxide: 30, Carbide: 30, Beryllium: 30));
    "heat-redirector" => SimpleBlock::new(3, false, cost!(Tungsten: 10, Graphite: 10));
    "heat-router" => SimpleBlock::new(3, false, cost!(Tungsten: 15, Graphite: 10));
    "slag-incinerator" => SimpleBlock::new(1, true, cost!(Tungsten: 15));
    "carbide-crucible" => SimpleBlock::new(3, true, cost!(Tungsten: 110, Thorium: 150, Oxide: 60));
    // slag centrifuge
    "surge-crucible" => SimpleBlock::new(3, true, cost!(Silicon: 100, Graphite: 80, Tungsten: 80, Oxide: 80));
    "cyanogen-synthesizer" => SimpleBlock::new(3, true, cost!(Carbide: 50, Silicon: 80, Beryllium: 90));
    "phase-synthesizer" => SimpleBlock::new(3, true, cost!(Carbide: 90, Silicon: 100, Thorium: 100, Tungsten: 200));
    // heat reactor
    // sandbox only
    "heat-source" => SimpleBlock::new(1, false, &[]);
}
