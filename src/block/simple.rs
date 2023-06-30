//! type used for basic blocks, eg turrets and factorys
use crate::item;

macro_rules! state_impl {
	($vis:vis $type:ty) => {
		$vis fn get_state(state: &$crate::block::State) -> &$type
		where Self: Sized {
			state.downcast_ref::<$type>().unwrap()
		}

		$vis fn get_state_mut(state: &mut $crate::block::State) -> &mut $type
		where Self: Sized {
			state.downcast_mut::<$type>().unwrap()
		}

		fn create_state(val: $type) -> $crate::block::State
		where Self: Sized {
			Box::new(val)
		}
	};
}

pub(crate) use state_impl;

macro_rules! make_simple {
    ($name: ident, $draw: expr) => {
        pub struct $name {
            size: u8,
            symmetric: bool,
            build_cost: BuildCost,
        }
        impl $name {
            #[must_use]
            pub const fn new(size: u8, symmetric: bool, build_cost: BuildCost) -> Self {
                assert!(size != 0, "invalid size");
                Self {
                    size,
                    symmetric,
                    build_cost,
                }
            }
        }

        use crate::block::{
            impl_block, simple::BuildCost, BlockLogic, DataConvertError, DeserializeError,
            SerializeError, State,
        };
        use crate::data::dynamic::DynData;
        use crate::data::GridPos;
        impl BlockLogic for $name {
            impl_block!();

            fn data_from_i32(&self, _: i32, _: GridPos) -> Result<DynData, DataConvertError> {
                Ok(DynData::Empty)
            }

            fn deserialize_state(&self, _: DynData) -> Result<Option<State>, DeserializeError> {
                Ok(None)
            }

            fn clone_state(&self, _: &State) -> State {
                panic!("{} has no custom state", stringify!($name))
            }

            fn mirror_state(&self, _: &mut State, _: bool, _: bool) {
                panic!("{} has no custom state", stringify!($name));
            }

            fn rotate_state(&self, _: &mut State, _: bool) {
                panic!("{} has no custom state", stringify!($name));
            }

            fn serialize_state(&self, _: &State) -> Result<DynData, SerializeError> {
                Ok(DynData::Empty)
            }

            fn draw(
                &self,
                category: &str,
                name: &str,
                state: Option<&State>,
            ) -> Option<image::RgbaImage> {
                $draw(self, category, name, state)
            }
        }
    };
    ($name: ident) => {
        crate::block::simple::make_simple!($name, |_, _, _, _| { None });
    };
}
pub(crate) use make_simple;

pub type BuildCost = &'static [(item::Type, u32)];

macro_rules! cost {
	($($item:ident: $cnt:expr),+) => {
		&[$((crate::item::Type::$item, $cnt)),*]
	};
}
pub(crate) use cost;
