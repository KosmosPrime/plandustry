use crate::block::make_register;
use crate::block::simple::SimpleBlock;

make_register!
(
	GROUND_FACTORY: "ground-factory" => SimpleBlock::new(3, false); // TODO config: unit index
	AIR_FACTORY: "air-factory" => SimpleBlock::new(3, false); // TODO config: unit index
	NAVAL_FACTORY: "naval-factory" => SimpleBlock::new(3, false); // TODO config: unit index
	ADDITIVE_RECONSTRUCTOR: "additive-reconstructor" => SimpleBlock::new(3, false);
	MULTIPLICATIVE_RECONSTRUCTOR: "multiplicative-reconstructor" => SimpleBlock::new(5, false);
	EXPONENTIAL_RECONSTRUCTOR: "exponential-reconstructor" => SimpleBlock::new(7, false);
	TETRATIVE_RECONSTRUCTOR: "tetrative-reconstructor" => SimpleBlock::new(9, false);
	REPAIR_POINT: "repair-point" => SimpleBlock::new(1, true);
	REPAIR_TURRET: "repair-turret" => SimpleBlock::new(2, true);
	PAYLOAD_CONVEYOR: "payload-conveyor" => SimpleBlock::new(3, false);
	PAYLOAD_ROUTER: "payload-router" => SimpleBlock::new(3, false);
	// sandbox only
	PAYLOAD_SOURCE: "payload-source" => SimpleBlock::new(5, false); // TODO config: block/unit
	PAYLOAD_VOID: "payload-void" => SimpleBlock::new(5, true);
);
