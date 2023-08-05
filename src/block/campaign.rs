//! campaign blocks
use crate::block::make_register;
use crate::block::simple::{cost, make_simple};
make_simple!(CampaignBlock);
make_register! {
    "launch-pad" -> CampaignBlock::new(3, true, cost!(Copper: 350, Lead: 200, Titanium: 150, Silicon: 140));
    "interplanetary-accelerator" -> CampaignBlock::new(7, true, cost!(Copper: 16000, Silicon: 11000, Thorium: 13000, Titanium: 12000, SurgeAlloy: 6000, PhaseFabric: 5000));
}
