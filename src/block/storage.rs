//! cores, vaults, containers
use crate::block::make_register;
use crate::block::simple::*;

make_register! {
  "core-shard" -> BasicBlock::new(3, true, cost!(Copper: 1000, Lead: 800));
  "core-foundation" -> BasicBlock::new(4, true, cost!(Copper: 3000, Lead: 3000, Silicon: 2000));
  "core-nucleus" -> BasicBlock::new(5, true, cost!(Copper: 8000, Lead: 8000, Thorium: 4000, Silicon: 5000));
  "core-bastion" -> BasicBlock::new(4, true, cost!(Graphite: 1000, Silicon: 1000, Beryllium: 800));
  "core-citadel" -> BasicBlock::new(5, true, cost!(Silicon: 4000, Beryllium: 4000, Tungsten: 3000, Oxide: 1000));
  "core-acropolis" -> BasicBlock::new(6, true, cost!(Beryllium: 6000, Silicon: 5000, Tungsten: 5000, Carbide: 3000, Oxide: 3000));
  "container" -> BasicBlock::new(2, true, cost!(Titanium: 100));
  "vault" -> BasicBlock::new(3, true, cost!(Titanium: 250, Thorium: 125));
  "reinforced-container" -> BasicBlock::new(2, true, cost!(Tungsten: 30, Graphite: 40));
  "reinforced-vault" -> BasicBlock::new(3, true, cost!(Tungsten: 125, Thorium: 70, Beryllium: 100));
}
