use std::any::{Any, type_name};

use crate::block::{BlockLogic, make_register};
use crate::block::simple::{SimpleBlock, state_impl};
use crate::data::dynamic::DynData;

make_register!
(
	MESSAGE: "message" => MessageLogic;
	SWITCH: "switch" => SwitchLogic;
	MICRO_PROCESSOR: "micro-processor" => SimpleBlock::new(1, true); // TODO config: code & links
	LOGIC_PROCESSOR: "logic-processor" => SimpleBlock::new(2, true); // TODO config: code & links
	HYPER_PROCESSOR: "hyper-processor" => SimpleBlock::new(3, true); // TODO config: code & links
	MEMORY_CELL: "memory-cell" => SimpleBlock::new(1, true);
	MEMORY_BANK: "memory-bank" => SimpleBlock::new(2, true);
	LOGIC_DISPLAY: "logic-display" => SimpleBlock::new(3, true);
	LARGE_LOGIC_DISPLAY: "large-logic-display" => SimpleBlock::new(6, true);
);

pub struct MessageLogic;

impl MessageLogic
{
	state_impl!(pub String);
}

impl BlockLogic for MessageLogic
{
	fn get_size(&self) -> u8
	{
		1
	}
	
	fn is_symmetric(&self) -> bool
	{
		true
	}
	
	fn data_from_i32(&self, _: i32) -> DynData
	{
		DynData::Empty
	}
	
	fn deserialize_state(&self, data: DynData) -> Option<Box<dyn Any>>
	{
		match data
		{
			DynData::Empty | DynData::String(None) => Some(Self::create_state(String::new())),
			DynData::String(Some(s)) => Some(Self::create_state(s)),
			_ => panic!("{} cannot use data of {:?}", type_name::<Self>(), data.get_type()),
		}
	}
	
	fn clone_state(&self, state: &dyn Any) -> Box<dyn Any>
	{
		Box::new(Self::get_state(state).clone())
	}
	
	fn serialize_state(&self, state: &dyn Any) -> DynData
	{
		DynData::String(Some(Self::get_state(state).clone()))
	}
}

pub struct SwitchLogic;

impl SwitchLogic
{
	state_impl!(pub bool);
}

impl BlockLogic for SwitchLogic
{
	fn get_size(&self) -> u8
	{
		1
	}
	
	fn is_symmetric(&self) -> bool
	{
		true
	}
	
	fn data_from_i32(&self, _: i32) -> DynData
	{
		DynData::Empty
	}
	
	fn deserialize_state(&self, data: DynData) -> Option<Box<dyn Any>>
	{
		match data
		{
			DynData::Empty => Some(Self::create_state(true)),
			DynData::Boolean(enabled) => Some(Self::create_state(enabled)),
			_ => panic!("{} cannot use data of {:?}", type_name::<Self>(), data.get_type()),
		}
	}
	
	fn clone_state(&self, state: &dyn Any) -> Box<dyn Any>
	{
		Box::new(Self::get_state(state).clone())
	}
	
	fn serialize_state(&self, state: &dyn Any) -> DynData
	{
		DynData::Boolean(*Self::get_state(state))
	}
}
