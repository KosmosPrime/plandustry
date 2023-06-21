use std::any::Any;
use std::borrow::Cow;
use std::error::Error;
use std::fmt;

use crate::access::BoxAccess;
use crate::data::dynamic::{DynData, DynType};
use crate::data::GridPos;
use crate::item::storage::Storage as ItemStorage;
use crate::registry::RegistryEntry;
use crate::utils::OnceCell;

pub mod base;
pub mod content;
pub mod defense;
pub mod extraction;
pub mod factory;
pub mod fluid;
pub mod logic;
pub mod payload;
pub mod power;
pub mod simple;
pub mod transport;
pub mod turret;

pub trait BlockLogic {
    fn get_size(&self) -> u8;

    fn is_symmetric(&self) -> bool;

    fn create_build_cost(&self) -> Option<ItemStorage>;

    fn data_from_i32(&self, config: i32, pos: GridPos) -> Result<DynData, DataConvertError>;

    fn deserialize_state(&self, data: DynData) -> Result<Option<Box<dyn Any>>, DeserializeError>;

    fn clone_state(&self, state: &dyn Any) -> Box<dyn Any>;

    fn mirror_state(&self, state: &mut dyn Any, horizontally: bool, vertically: bool);

    fn rotate_state(&self, state: &mut dyn Any, clockwise: bool);

    fn serialize_state(&self, state: &dyn Any) -> Result<DynData, SerializeError>;
}

#[derive(Debug)]
pub enum DataConvertError {
    Custom(Box<dyn Error>),
}

impl DataConvertError {
    pub fn forward<T, E: Error + 'static>(result: Result<T, E>) -> Result<T, Self> {
        match result {
            Ok(v) => Ok(v),
            Err(e) => Err(Self::Custom(Box::new(e))),
        }
    }
}

impl fmt::Display for DataConvertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Custom(e) => e.fmt(f),
        }
    }
}

impl Error for DataConvertError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Custom(e) => e.source(),
        }
    }
}

#[derive(Debug)]
pub enum DeserializeError {
    InvalidType { have: DynType, expect: DynType },
    Custom(Box<dyn Error>),
}

impl DeserializeError {
    pub fn forward<T, E: Error + 'static>(result: Result<T, E>) -> Result<T, Self> {
        match result {
            Ok(v) => Ok(v),
            Err(e) => Err(Self::Custom(Box::new(e))),
        }
    }
}

impl fmt::Display for DeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidType { have, expect } => {
                write!(f, "expected type {expect:?} but got {have:?}")
            }
            Self::Custom(e) => e.fmt(f),
        }
    }
}

impl Error for DeserializeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Custom(e) => e.source(),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum SerializeError {
    Custom(Box<dyn Error>),
}

impl SerializeError {
    pub fn forward<T, E: Error + 'static>(result: Result<T, E>) -> Result<T, Self> {
        match result {
            Ok(v) => Ok(v),
            Err(e) => Err(Self::Custom(Box::new(e))),
        }
    }
}

impl fmt::Display for SerializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Custom(e) => e.fmt(f),
        }
    }
}

impl Error for SerializeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Custom(e) => e.source(),
        }
    }
}

pub struct Block {
    name: Cow<'static, str>,
    logic: BoxAccess<'static, dyn BlockLogic + Sync>,
    build_cost: OnceCell<Option<ItemStorage>>,
}

impl Block {
    pub const fn new(
        name: Cow<'static, str>,
        logic: BoxAccess<'static, dyn BlockLogic + Sync>,
    ) -> Self {
        Self {
            name,
            logic,
            build_cost: OnceCell::new(),
        }
    }

    pub fn get_size(&self) -> u8 {
        self.logic.get_size()
    }

    pub fn is_symmetric(&self) -> bool {
        self.logic.is_symmetric()
    }

    pub fn get_build_cost(&self) -> Option<&ItemStorage> {
        self.build_cost
            .get_or_init(|| self.logic.as_ref().create_build_cost())
            .as_ref()
    }

    pub fn data_from_i32(&self, config: i32, pos: GridPos) -> Result<DynData, DataConvertError> {
        self.logic.data_from_i32(config, pos)
    }

    pub fn deserialize_state(
        &self,
        data: DynData,
    ) -> Result<Option<Box<dyn Any>>, DeserializeError> {
        self.logic.deserialize_state(data)
    }

    pub fn clone_state(&self, state: &dyn Any) -> Box<dyn Any> {
        self.logic.clone_state(state)
    }

    pub fn mirror_state(&self, state: &mut dyn Any, horizontally: bool, vertically: bool) {
        self.logic.mirror_state(state, horizontally, vertically);
    }

    pub fn rotate_state(&self, state: &mut dyn Any, clockwise: bool) {
        self.logic.rotate_state(state, clockwise);
    }

    pub fn rotate_180(&mut self, state: &mut dyn Any) {
        self.logic.mirror_state(state, true, true);
    }

    pub fn serialize_state(&self, state: &dyn Any) -> Result<DynData, SerializeError> {
        self.logic.serialize_state(state)
    }
}

impl fmt::Debug for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name: &str = &self.name;
        write!(f, "Block {{ name: {:?} }}", name)
    }
}

impl RegistryEntry for Block {
    fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Rotation {
    Right,
    Up,
    Left,
    Down,
}

impl Rotation {
    pub fn mirrored(self, horizontally: bool, vertically: bool) -> Self {
        match self {
            Self::Right => {
                if horizontally {
                    Self::Left
                } else {
                    Self::Right
                }
            }
            Self::Up => {
                if vertically {
                    Self::Down
                } else {
                    Self::Up
                }
            }
            Self::Left => {
                if horizontally {
                    Self::Right
                } else {
                    Self::Left
                }
            }
            Self::Down => {
                if vertically {
                    Self::Up
                } else {
                    Self::Down
                }
            }
        }
    }

    pub fn mirror(&mut self, horizontally: bool, vertically: bool) {
        *self = self.mirrored(horizontally, vertically);
    }

    pub fn rotated(self, clockwise: bool) -> Self {
        match self {
            Self::Right => {
                if clockwise {
                    Self::Down
                } else {
                    Self::Up
                }
            }
            Self::Up => {
                if clockwise {
                    Self::Right
                } else {
                    Self::Left
                }
            }
            Self::Left => {
                if clockwise {
                    Self::Up
                } else {
                    Self::Down
                }
            }
            Self::Down => {
                if clockwise {
                    Self::Left
                } else {
                    Self::Right
                }
            }
        }
    }

    pub fn rotate(&mut self, clockwise: bool) {
        *self = self.rotated(clockwise);
    }

    pub fn rotated_180(self) -> Self {
        match self {
            Self::Right => Self::Left,
            Self::Up => Self::Down,
            Self::Left => Self::Right,
            Self::Down => Self::Up,
        }
    }

    pub fn rotate_180(&mut self) {
        *self = self.rotated_180();
    }
}

impl From<u8> for Rotation {
    fn from(val: u8) -> Self {
        match val & 3 {
            0 => Self::Right,
            1 => Self::Up,
            2 => Self::Left,
            _ => Self::Down,
        }
    }
}

impl From<Rotation> for u8 {
    fn from(rot: Rotation) -> Self {
        match rot {
            Rotation::Right => 0,
            Rotation::Up => 1,
            Rotation::Left => 2,
            Rotation::Down => 3,
        }
    }
}

pub type BlockRegistry<'l> = crate::registry::Registry<'l, Block>;
pub type RegisterError<'l> = crate::registry::RegisterError<'l, Block>;

macro_rules! make_register
{
	($($field:literal => $logic:expr;)+) =>
	{
        paste::paste! {
		$(
			pub static [<$field:snake:upper>]: $crate::block::Block = $crate::block::Block::new(
				std::borrow::Cow::Borrowed($field), $crate::access::Access::Borrowed(&$logic));
		)+

		pub fn register<'l>(reg: &mut $crate::block::BlockRegistry<'l>) {
			$(assert!(reg.register(&[<$field:snake:upper>]).is_ok(), "duplicate block {:?}", $field);)+
		}
    }
	};
}
pub(crate) use make_register;

pub fn build_registry() -> BlockRegistry<'static> {
    let mut reg = BlockRegistry::new();
    register(&mut reg);
    reg
}

pub fn register<'l>(reg: &mut BlockRegistry<'l>) {
    turret::register(reg);
    extraction::register(reg);
    transport::register(reg);
    fluid::register(reg);
    power::register(reg);
    defense::register(reg);
    factory::register(reg);
    payload::register(reg);
    base::register(reg);
    logic::register(reg);
}
