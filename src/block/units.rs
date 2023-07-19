//! unit creation related blocks
use thiserror::Error;

use crate::block::simple::*;
use crate::block::*;
use crate::data::dynamic::DynType;
use crate::unit;

make_simple!(ConstructorBlock);
make_simple!(UnitBlock);

const GROUND_UNITS: &[unit::Type] = &[unit::Type::Dagger, unit::Type::Crawler, unit::Type::Nova];
const AIR_UNITS: &[unit::Type] = &[unit::Type::Flare, unit::Type::Mono];
const NAVAL_UNITS: &[unit::Type] = &[unit::Type::Risso, unit::Type::Retusa];

make_register! {
    "ground-factory" => AssemblerBlock::new(3, false, cost!(Copper: 50, Lead: 120, Silicon: 80), GROUND_UNITS);
    "air-factory" => AssemblerBlock::new(3, false, cost!(Copper: 60, Lead: 70), AIR_UNITS);
    "naval-factory" => AssemblerBlock::new(3, false, cost!(Copper: 150, Lead: 130, Metaglass: 120), NAVAL_UNITS);
    "additive-reconstructor" => ConstructorBlock::new(3, false, cost!(Copper: 200, Lead: 120, Silicon: 90));
    "multiplicative-reconstructor" => ConstructorBlock::new(5, false, cost!(Lead: 650, Titanium: 350, Thorium: 650, Silicon: 450));
    "exponential-reconstructor" => ConstructorBlock::new(7, false,
        cost!(Lead: 2000, Titanium: 2000, Thorium: 750, Silicon: 1000, Plastanium: 450, PhaseFabric: 600));
    "tetrative-reconstructor" => ConstructorBlock::new(9, false,
        cost!(Lead: 4000, Thorium: 1000, Silicon: 3000, Plastanium: 600, PhaseFabric: 600, SurgeAlloy: 800));
    "repair-point" => UnitBlock::new(1, true, cost!(Copper: 30, Lead: 30, Silicon: 20));
    "repair-turret" => UnitBlock::new(2, true, cost!(Thorium: 80, Silicon: 90, Plastanium: 60));
    "tank-fabricator" => ConstructorBlock::new(3, true, cost!(Silicon: 200, Beryllium: 150));
    "ship-fabricator" => ConstructorBlock::new(3, true, cost!(Silicon: 250, Beryllium: 200));
    "mech-fabricator" => ConstructorBlock::new(3, true, cost!(Silicon: 200, Graphite: 300, Tungsten: 60));
    "tank-refabricator" => ConstructorBlock::new(3, true, cost!(Beryllium: 200, Tungsten: 80, Silicon: 100));
    "mech-refabricator" => ConstructorBlock::new(3, true, cost!(Beryllium: 250, Tungsten: 120, Silicon: 150));
    "ship-refabricator" => ConstructorBlock::new(3, true, cost!(Beryllium: 200, Tungsten: 100, Silicon: 150, Oxide: 40));
    "prime-refabricator" => ConstructorBlock::new(5, true, cost!(Thorium: 250, Oxide: 200, Tungsten: 200, Silicon: 400));
    "tank-assembler" => ConstructorBlock::new(5, true, cost!(Thorium: 500, Oxide: 150, Carbide: 80, Silicon: 500));
    "ship-assembler" => ConstructorBlock::new(5, true, cost!(Carbide: 100, Oxide: 200, Tungsten: 500, Silicon: 800, Thorium: 400));
    "mech-assembler" => ConstructorBlock::new(5, true, cost!(Carbide: 200, Thorium: 600, Oxide: 200, Tungsten: 500, Silicon: 900)); // smh collaris
    "basic-assembler-module" => ConstructorBlock::new(5, true, cost!(Carbide: 300, Thorium: 500, Oxide: 200, PhaseFabric: 400)); // the dummy block
    "unit-repair-tower" => UnitBlock::new(2, true, cost!(Graphite: 90, Silicon: 90, Tungsten: 80));

}

pub struct AssemblerBlock {
    size: u8,
    symmetric: bool,
    build_cost: BuildCost,
    valid: &'static [unit::Type],
}

impl AssemblerBlock {
    #[must_use]
    pub const fn new(
        size: u8,
        symmetric: bool,
        build_cost: BuildCost,
        valid: &'static [unit::Type],
    ) -> Self {
        assert!(size != 0, "invalid size");
        assert!(!valid.is_empty(), "no valid units");
        assert!(valid.len() <= i32::MAX as usize, "too many valid units");
        Self {
            size,
            symmetric,
            build_cost,
            valid,
        }
    }

    state_impl!(pub Option<unit::Type>);
}

impl BlockLogic for AssemblerBlock {
    impl_block!();

    fn data_from_i32(&self, _: i32, _: GridPos) -> Result<DynData, DataConvertError> {
        Ok(DynData::Int(-1))
    }

    fn deserialize_state(&self, data: DynData) -> Result<Option<State>, DeserializeError> {
        match data {
            DynData::Empty => Ok(Some(Self::create_state(None))),
            DynData::Int(idx) => {
                if idx == -1 {
                    Ok(Some(Self::create_state(None)))
                } else if idx >= 0 && idx < self.valid.len() as i32 {
                    Ok(Some(Self::create_state(Some(self.valid[idx as usize]))))
                } else {
                    Err(DeserializeError::Custom(Box::new(
                        AssemblerDeserializeError {
                            idx,
                            count: self.valid.len() as i32,
                        },
                    )))
                }
            }
            _ => Err(DeserializeError::InvalidType {
                have: data.get_type(),
                expect: DynType::Int,
            }),
        }
    }

    fn clone_state(&self, state: &State) -> State {
        let state = Self::get_state(state);
        Box::new(Self::create_state(*state))
    }

    fn mirror_state(&self, _: &mut State, _: bool, _: bool) {}

    fn rotate_state(&self, _: &mut State, _: bool) {}

    fn serialize_state(&self, state: &State) -> Result<DynData, SerializeError> {
        if let Some(state) = Self::get_state(state) {
            for (i, curr) in self.valid.iter().enumerate() {
                if curr == state {
                    return Ok(DynData::Int(i as i32));
                }
            }
            Err(SerializeError::Custom(Box::new(AssemblerSerializeError(
                *state,
            ))))
        } else {
            Ok(DynData::Int(-1))
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
#[error("invalid unit index ({idx}, valid: {count})")]
pub struct AssemblerDeserializeError {
    pub idx: i32,
    pub count: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
#[error("invalid unit {0:?}")]
pub struct AssemblerSerializeError(unit::Type);
