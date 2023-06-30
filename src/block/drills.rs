//! extraction of raw resources (mine part)
use crate::block::make_register;
use crate::block::simple::{cost, make_simple};
use crate::data::renderer::read_with;
make_simple!(DrillBlock, |_, _, name, _| {
    if name == "cliff-crusher" {
        const SFX: &[&str; 3] = &["", "-top", "-rotator"];
        return Some(read_with("drills", "cliff-crusher", SFX, 2u16));
    }
    None
});

make_register! {
    "mechanical-drill" => DrillBlock::new(2, true, cost!(Copper: 12));
    "pneumatic-drill" => DrillBlock::new(2, true, cost!(Copper: 18, Graphite: 10));
    "laser-drill" => DrillBlock::new(3, true, cost!(Copper: 35, Graphite: 30, Titanium: 20, Silicon: 30));
    "blast-drill" => DrillBlock::new(4, true, cost!(Copper: 65, Titanium: 50, Thorium: 75, Silicon: 60));
    "water-extractor" => DrillBlock::new(2, true, cost!(Copper: 30, Lead: 30, Metaglass: 30, Graphite: 30));
    "cultivator" => DrillBlock::new(2, true, cost!(Copper: 25, Lead: 25, Silicon: 10));
    "oil-extractor" => DrillBlock::new(3, true, cost!(Copper: 150, Lead: 115, Graphite: 175, Thorium: 115, Silicon: 75));
    "vent-condenser" => DrillBlock::new(3, true, cost!(Graphite: 20, Beryllium: 60));
    "cliff-crusher" => DrillBlock::new(2, false, cost!(Beryllium: 100, Graphite: 40));
    "plasma-bore" => DrillBlock::new(2, false, cost!(Beryllium: 40));
    "large-plasma-bore" => DrillBlock::new(3, false, cost!(Silicon: 100, Oxide: 25, Beryllium: 100, Tungsten: 70));
    "impact-drill" => DrillBlock::new(4, true, cost!(Silicon: 70, Beryllium: 90, Graphite: 60));
    "eruption-drill" => DrillBlock::new(5, true, cost!(Silicon: 200, Oxide: 20, Tungsten: 200, Thorium: 120));
}
