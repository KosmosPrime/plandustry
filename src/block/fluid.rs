use crate::block::make_register;
use crate::block::simple::SimpleBlock;

make_register!
(
	MECHANICAL_PUMP: "mechanical-pump" => SimpleBlock::new(1, true);
	ROTARY_PUMP: "rotary-pump" => SimpleBlock::new(2, true);
	IMPULSE_PUMP: "impulse-pump" => SimpleBlock::new(3, true);
	CONDUIT: "conduit" => SimpleBlock::new(1, false);
	PULSE_CONDUIT: "pulse-conduit" => SimpleBlock::new(1, false);
	PLATED_CONDUIT: "plated-conduit" => SimpleBlock::new(1, false);
	LIQUID_ROUTER: "liquid-router" => SimpleBlock::new(1, true);
	LIQUID_CONTAINER: "liquid-container" => SimpleBlock::new(2, true);
	LIQUID_TANK: "liquid-tank" => SimpleBlock::new(3, true);
	LIQUID_JUNCTION: "liquid-junction" => SimpleBlock::new(1, true);
	BRIDGE_CONDUIT: "bridge-conduit" => SimpleBlock::new(1, true); // TODO config: destination
	PHASE_CONDUIT: "phase-conduit" => SimpleBlock::new(1, true); // TODO config: destination
	// sandbox only
	LIQUID_SOURCE: "liquid-source" => SimpleBlock::new(1, true); // TODO config: fluid
	LIQUID_VOID: "liquid-void" => SimpleBlock::new(1, true);
);
