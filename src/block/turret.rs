use crate::block::make_register;
use crate::block::simple::{cost, SimpleBlock};

make_register!
(
    "duo" => SimpleBlock::new(1, true, cost!(Copper: 35));
    "scatter" => SimpleBlock::new(2, true, cost!(Copper: 85, Lead: 45));
    "scorch" => SimpleBlock::new(1, true, cost!(Copper: 25, Graphite: 22));
    "hail" => SimpleBlock::new(1, true, cost!(Copper: 40, Graphite: 17));
    "wave" => SimpleBlock::new(2, true, cost!(Copper: 25, Lead: 75, Metaglass: 45));
    "lancer" => SimpleBlock::new(2, true, cost!(Copper: 60, Lead: 70, Titanium: 30, Silicon: 60));
    "arc" => SimpleBlock::new(1, true, cost!(Copper: 50, Lead: 50));
    "parallax" => SimpleBlock::new(2, true, cost!(Graphite: 30, Titanium: 90, Silicon: 120));
    "swarmer" => SimpleBlock::new(2, true, cost!(Graphite: 35, Titanium: 35, Silicon: 30, Plastanium: 45));
    "salvo" => SimpleBlock::new(2, true, cost!(Copper: 100, Graphite: 80, Titanium: 50));
    "segment" => SimpleBlock::new(2, true, cost!(Titanium: 40, Thorium: 80, Silicon: 130, PhaseFabric: 40));
    "tsunami" => SimpleBlock::new(3, true, cost!(Lead: 400, Metaglass: 100, Titanium: 250, Thorium: 100));
    "fuse" => SimpleBlock::new(3, true, cost!(Copper: 225, Graphite: 225, Thorium: 100));
    "ripple" => SimpleBlock::new(3, true, cost!(Copper: 150, Graphite: 135, Titanium: 60));
    "cyclone" => SimpleBlock::new(3, true, cost!(Copper: 200, Titanium: 125, Plastanium: 80));
    "foreshadow" => SimpleBlock::new(4, true, cost!(Copper: 1000, Metaglass: 600, Silicon: 600, Plastanium: 200, SurgeAlloy: 300));
    "spectre" => SimpleBlock::new(4, true, cost!(Copper: 900, Graphite: 300, Thorium: 250, Plastanium: 175, SurgeAlloy: 250));
    "meltdown" => SimpleBlock::new(4, true, cost!(Copper: 1200, Lead: 350, Graphite: 300, Silicon: 325, SurgeAlloy: 325));
);
