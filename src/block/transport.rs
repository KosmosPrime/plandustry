use crate::block::make_register;
use crate::block::simple::SimpleBlock;

make_register!
(
	CONVEYOR: "conveyor" => SimpleBlock::new(1, false);
	TITANIUM_CONVEYOR: "titanium-conveyor" => SimpleBlock::new(1, false);
	PLASTANIUM_CONVEYOR: "plastanium-conveyor" => SimpleBlock::new(1, false);
	ARMORED_CONVEYOR: "armored-conveyor" => SimpleBlock::new(1, false);
	JUNCTION: "junction" => SimpleBlock::new(1, true);
	BRIDGE_CONVEYOR: "bridge-conveyor" => SimpleBlock::new(1, false); // TODO config: destination
	PHASE_CONVEYOR: "phase-conveyor" => SimpleBlock::new(1, false); // TODO config: destination
	SORTER: "sorter" => SimpleBlock::new(1, true); // TODO config: item
	INVERTED_SORTER: "inverted-sorter" => SimpleBlock::new(1, true); // TODO config: item
	ROUTER: "router" => SimpleBlock::new(1, true);
	DISTRIBUTOR: "distributor" => SimpleBlock::new(2, true);
	OVERFLOW_GATE: "overflow-gate" => SimpleBlock::new(1, true);
	UNDERFLOW_GATE: "underflow-gate" => SimpleBlock::new(1, true);
	MASS_DRIVER: "mass-driver" => SimpleBlock::new(3, true); // TODO config: destination
	// sandbox only
	ITEM_SOURCE: "item-source" => SimpleBlock::new(1, true); // TODO config: item
	ITEM_VOID: "item-void" => SimpleBlock::new(1, true);
);
