use crate::block::make_register;
use crate::block::simple::SimpleBlock;

make_register!
(
	MESSAGE: "message" => SimpleBlock::new(1, true); // TODO config: message
	SWITCH: "switch" => SimpleBlock::new(1, true); // TODO config: enabled
	MICRO_PROCESSOR: "micro-processor" => SimpleBlock::new(1, true); // TODO config: code & links
	LOGIC_PROCESSOR: "logic-processor" => SimpleBlock::new(2, true); // TODO config: code & links
	HYPER_PROCESSOR: "hyper-processor" => SimpleBlock::new(3, true); // TODO config: code & links
	MEMORY_CELL: "memory-cell" => SimpleBlock::new(1, true);
	MEMORY_BANK: "memory-bank" => SimpleBlock::new(2, true);
	LOGIC_DISPLAY: "logic-display" => SimpleBlock::new(3, true);
	LARGE_LOGIC_DISPLAY: "large-logic-display" => SimpleBlock::new(6, true);
);
