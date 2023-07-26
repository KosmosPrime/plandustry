//! extraction of raw resources (mine part)
use super::production::ProductionBlock;
use crate::block::simple::{cost, make_simple};
use crate::block::*;

make_simple!(
    DrillBlock,
    |_, _, name, _, _, rot: Rotation| {
        if matches!(name, "large-plasma-bore" | "plasma-bore") {
            let mut base = load("drills", name).unwrap().clone();
            let top = load("drills", &format!("{name}-top")).unwrap();
            if rot == Rotation::Right {
                base.overlay(&top, 0, 0);
            } else {
                let mut top = top.clone();
                top.rotate(rot.rotated(false).count());
                base.overlay(&top, 0, 0);
            }
            return Some(ImageHolder::from(base));
        }
        Some(ImageHolder::Borrow(load("drills", name).unwrap()))
    },
    |_, _, _, buff: &mut DataRead| { read_drill(buff) }
);
make_simple!(ExtractorBlock);
make_simple!(WallCrafter, |_, _, _, _, _, rot: Rotation| {
    let mut base = load("drills", "cliff-crusher").unwrap().clone();
    let top = load("drills", "cliff-crusher-top").unwrap();
    if rot == Rotation::Right {
        base.overlay(&top, 0, 0);
    } else {
        let mut top = top.clone();
        top.rotate(rot.rotated(false).count());
        base.overlay(&top, 0, 0);
    }
    Some(ImageHolder::from(base))
});

make_register! {
    "mechanical-drill" => DrillBlock::new(2, true, cost!(Copper: 12));
    "pneumatic-drill" => DrillBlock::new(2, true, cost!(Copper: 18, Graphite: 10));
    "laser-drill" => DrillBlock::new(3, true, cost!(Copper: 35, Graphite: 30, Titanium: 20, Silicon: 30));
    "blast-drill" => DrillBlock::new(4, true, cost!(Copper: 65, Titanium: 50, Thorium: 75, Silicon: 60));
    "water-extractor" => ExtractorBlock::new(2, true, cost!(Copper: 30, Lead: 30, Metaglass: 30, Graphite: 30));
    "oil-extractor" => ExtractorBlock::new(3, true, cost!(Copper: 150, Lead: 115, Graphite: 175, Thorium: 115, Silicon: 75));
    "vent-condenser" => ProductionBlock::new(3, true, cost!(Graphite: 20, Beryllium: 60));
    "cliff-crusher" => WallCrafter::new(2, false, cost!(Beryllium: 100, Graphite: 40));
    "plasma-bore" => DrillBlock::new(2, false, cost!(Beryllium: 40));
    "large-plasma-bore" => DrillBlock::new(3, false, cost!(Silicon: 100, Oxide: 25, Beryllium: 100, Tungsten: 70));
    "impact-drill" => DrillBlock::new(4, true, cost!(Silicon: 70, Beryllium: 90, Graphite: 60));
    "eruption-drill" => DrillBlock::new(5, true, cost!(Silicon: 200, Oxide: 20, Tungsten: 200, Thorium: 120));
}

/// format:
/// - progress: [`f32`]
/// - warmup: [`f32`]
fn read_drill(buff: &mut DataRead) -> Result<(), DataReadError> {
    buff.skip(8)
}
