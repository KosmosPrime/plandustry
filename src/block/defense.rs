use crate::block::make_register;
use crate::block::simple::SimpleBlock;

make_register!
(
	COPPER_WALL: "copper-wall" => SimpleBlock::new(1, true);
	COPPER_WALL_LARGE: "copper-wall-large" => SimpleBlock::new(2, true);
	TITANIUM_WALL: "titanium-wall" => SimpleBlock::new(1, true);
	TITANIUM_WALL_LARGE: "titanium-wall-large" => SimpleBlock::new(2, true);
	PLASTANIUM_WALL: "plastanium-wall" => SimpleBlock::new(1, true);
	PLASTANIUM_WALL_LARGE: "plastanium-wall-large" => SimpleBlock::new(2, true);
	THORIUM_WALL: "thorium-wall" => SimpleBlock::new(1, true);
	THORIUM_WALL_LARGE: "thorium-wall-large" => SimpleBlock::new(2, true);
	PHASE_WALL: "phase-wall" => SimpleBlock::new(1, true);
	PHASE_WALL_LARGE: "phase-wall-large" => SimpleBlock::new(2, true);
	SURGE_WALL: "surge-wall" => SimpleBlock::new(1, true);
	SURGE_WALL_LARGE: "surge-wall-large" => SimpleBlock::new(2, true);
	DOOR: "door" => SimpleBlock::new(1, true); // TODO config: opened
	DOOR_LARGE: "door-large" => SimpleBlock::new(2, true); // TODO config: opened
	// sandbox only
	SCRAP_WALL: "scrap-wall" => SimpleBlock::new(1, true);
	SCRAP_WALL_LARGE: "scrap-wall-large" => SimpleBlock::new(2, true);
	SCRAP_WALL_HUGE: "scrap-wall-huge" => SimpleBlock::new(3, true);
	SCRAP_WALL_GIGANTIC: "scrap-wall-gigantic" => SimpleBlock::new(4, true);
	THRUSTER: "thruster" => SimpleBlock::new(4, false);
);
