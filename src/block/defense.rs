//! defense
use crate::block::make_register;
use crate::block::simple::{cost, make_simple};
make_simple!(DefenseBlock);
make_register! {
    "mender" => DefenseBlock::new(1, true, cost!(Copper: 25, Lead: 30));
    "mend-projector" => DefenseBlock::new(2, true, cost!(Copper: 50, Lead: 100, Titanium: 25, Silicon: 40));
    "overdrive-projector" => DefenseBlock::new(2, true, cost!(Lead: 100, Titanium: 75, Silicon: 75, Plastanium: 30));
    "overdrive-dome" => DefenseBlock::new(3, true, cost!(Lead: 200, Titanium: 130, Silicon: 130, Plastanium: 80, SurgeAlloy: 120));
    "force-projector" => DefenseBlock::new(3, true, cost!(Lead: 100, Titanium: 75, Silicon: 125));
    "regen-projector" => DefenseBlock::new(3, true, cost!(Silicon: 80, Tungsten: 60, Oxide: 40, Beryllium: 80));
    "shock-mine" => DefenseBlock::new(1, true, cost!(Lead: 25, Silicon: 12));
    "radar" => DefenseBlock::new(1, true, cost!(Silicon: 60, Graphite: 50, Beryllium: 10));
    "build-tower" => DefenseBlock::new(3, true, cost!(Silicon: 150, Oxide: 40, Thorium: 60));
    // barrier projector
    // editor only
    "shockwave-tower" => DefenseBlock::new(3, true, cost!(SurgeAlloy: 50, Silicon: 150, Oxide: 30, Tungsten: 100));
    "shield-projector" => DefenseBlock::new(3, true, &[]);
    "large-shield-projector" => DefenseBlock::new(4, true, &[]);
}
