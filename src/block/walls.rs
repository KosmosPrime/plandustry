//! walls
use crate::block::simple::*;
use crate::block::*;
use crate::data::dynamic::DynType;
use crate::data::renderer::load;

make_simple!(WallBlock, |_, name, _, _, _, s| {
    match name {
        "thruster" => {
            let mut base = load("thruster", s);
            base.overlay(&load("thruster-top", s));
            base
        }
        _ => load(name, s),
    }
});

make_register! {
    "copper-wall" => WallBlock::new(1, true, cost!(Copper: 6));
    "copper-wall-large" => WallBlock::new(2, true, cost!(Copper: 6 * 4));
    "titanium-wall" => WallBlock::new(1, true, cost!(Titanium: 6));
    "titanium-wall-large" => WallBlock::new(2, true, cost!(Titanium: 6 * 4));
    "plastanium-wall" => WallBlock::new(1, true, cost!(Metaglass: 2, Plastanium: 5));
    "plastanium-wall-large" => WallBlock::new(2, true, cost!(Metaglass: 2 * 4, Plastanium: 5 * 4));
    "thorium-wall" => WallBlock::new(1, true, cost!(Thorium: 6));
    "thorium-wall-large" => WallBlock::new(2, true, cost!(Thorium: 6 * 4));
    "phase-wall" => WallBlock::new(1, true, cost!(PhaseFabric: 6));
    "phase-wall-large" => WallBlock::new(2, true, cost!(PhaseFabric: 6 * 4));
    "surge-wall" => WallBlock::new(1, true, cost!(SurgeAlloy: 6));
    "surge-wall-large" => WallBlock::new(2, true, cost!(SurgeAlloy: 6 * 4));
    "door" => DoorBlock::new(1, true, cost!(Titanium: 6, Silicon: 4));
    "door-large" => DoorBlock::new(2, true, cost!(Titanium: 6 * 4, Silicon: 4 * 4));
    "tungsten-wall" => WallBlock::new(1, true, cost!(Tungsten: 6));
    "tungsten-wall-large" => WallBlock::new(2, true, cost!(Tungsten: 6 * 4));
    "blast-door" => DoorBlock::new(2, true, cost!(Tungsten: 24, Silicon: 24));
    "reinforced-surge-wall" => WallBlock::new(1, true, cost!(SurgeAlloy: 6, Tungsten: 2));
    "reinforced-surge-wall-large" => WallBlock::new(2, true, cost!(SurgeAlloy: 6 * 4, Tungsten: 2 * 4));
    "carbide-wall" => WallBlock::new(1, true, cost!(Thorium: 6, Carbide: 6));
    "carbide-wall-large" => WallBlock::new(2, true, cost!(Thorium: 6 * 4, Carbide: 6 * 4));
    "shielded-wall" => WallBlock::new(2, true, cost!(PhaseFabric: 20, SurgeAlloy: 12, Beryllium: 12));
    "beryllium-wall" => WallBlock::new(1, true, cost!(Beryllium: 6));
    "beryllium-wall-large" => WallBlock::new(2, true, cost!(Beryllium: 6 * 4));
    // sandbox only
    "scrap-wall" => WallBlock::new(1, true, cost!(Scrap: 6));
    "scrap-wall-large" => WallBlock::new(2, true, cost!(Scrap: 24));
    "scrap-wall-huge" => WallBlock::new(3, true, cost!(Scrap: 54));
    "scrap-wall-gigantic" => WallBlock::new(4, true, cost!(Scrap: 96));
    "thruster" => WallBlock::new(4, false, cost!(Scrap: 96));
}

pub struct DoorBlock {
    size: u8,
    symmetric: bool,
    build_cost: BuildCost,
}

impl DoorBlock {
    #[must_use]
    pub const fn new(size: u8, symmetric: bool, build_cost: BuildCost) -> Self {
        assert!(size != 0, "invalid size");
        Self {
            size,
            symmetric,
            build_cost,
        }
    }

    state_impl!(pub bool);
}

impl BlockLogic for DoorBlock {
    impl_block!();

    fn draw(
        &self,
        name: &str,
        state: Option<&State>,
        _: Option<&RenderingContext>,
        _: Rotation,
        s: Scale,
    ) -> ImageHolder {
        if let Some(state) = state {
            return if *Self::get_state(state) {
                // TODO check
                load(
                    match name {
                        "door" => "door-open",
                        "blast-door" => "blast-door-open",
                        _ => "door-large-open",
                    },
                    s,
                )
            } else {
                load(name, s)
            };
        }
        load(name, s)
    }

    fn data_from_i32(&self, _: i32, _: GridPos) -> Result<DynData, DataConvertError> {
        Ok(DynData::Boolean(false))
    }

    fn deserialize_state(&self, data: DynData) -> Result<Option<State>, DeserializeError> {
        match data {
            DynData::Boolean(opened) => Ok(Some(Self::create_state(opened))),
            _ => Err(DeserializeError::InvalidType {
                have: data.get_type(),
                expect: DynType::Boolean,
            }),
        }
    }

    fn clone_state(&self, state: &State) -> State {
        let state = Self::get_state(state);
        Box::new(Self::create_state(*state))
    }

    fn serialize_state(&self, state: &State) -> Result<DynData, SerializeError> {
        let state = Self::get_state(state);
        Ok(DynData::Boolean(*state))
    }

    fn read(
        &self,
        build: &mut Build,
        _: &BlockRegistry,
        _: &EntityMapping,
        buff: &mut DataRead,
    ) -> Result<(), DataReadError> {
        build.state = Some(Self::create_state(buff.read_bool()?));
        Ok(())
    }
}
