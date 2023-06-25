//! extraction of raw resources (mine part)
use crate::block::make_register;
use crate::block::simple::{cost, SimpleBlock};

make_register!
(
    "mechanical-drill" => SimpleBlock::new(2, true, cost!(Copper: 12));
    "pneumatic-drill" => SimpleBlock::new(2, true, cost!(Copper: 18, Graphite: 10));
    "laser-drill" => SimpleBlock::new(3, true, cost!(Copper: 35, Graphite: 30, Titanium: 20, Silicon: 30));
    "blast-drill" => SimpleBlock::new(4, true, cost!(Copper: 65, Titanium: 50, Thorium: 75, Silicon: 60));
    "water-extractor" => SimpleBlock::new(2, true, cost!(Copper: 30, Lead: 30, Metaglass: 30, Graphite: 30));
    "cultivator" => SimpleBlock::new(2, true, cost!(Copper: 25, Lead: 25, Silicon: 10));
    "oil-extractor" => SimpleBlock::new(3, true, cost!(Copper: 150, Lead: 115, Graphite: 175, Thorium: 115, Silicon: 75));
    "vent-condenser" => SimpleBlock::new(3, true, cost!(Graphite: 20, Beryllium: 60));
    "cliff-crusher" => SimpleBlock::new(2, false, cost!(Beryllium: 100, Graphite: 40));
    "plasma-bore" => SimpleBlock::new(2, false, cost!(Beryllium: 40));
    "large-plasma-bore" => SimpleBlock::new(3, false, cost!(Silicon: 100, Oxide: 25, Beryllium: 100, Tungsten: 70));
    "impact-drill" => SimpleBlock::new(4, true, cost!(Silicon: 70, Beryllium: 90, Graphite: 60));
    "eruption-drill" => SimpleBlock::new(5, true, cost!(Silicon: 200, Oxide: 20, Tungsten: 200, Thorium: 120));
);
