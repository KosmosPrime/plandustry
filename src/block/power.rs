use crate::block::make_register;
use crate::block::simple::SimpleBlock;

make_register!
(
	POWER_NODE: "power-node" => SimpleBlock::new(1, true); // TODO config: destination
	POWER_NODE_LARGE: "power-node-large" => SimpleBlock::new(2, true); // TODO config: destination
	SURGE_TOWER: "surge-tower" => SimpleBlock::new(2, true); // TODO config: destination
	DIODE: "diode" => SimpleBlock::new(1, false);
	BATTERY: "battery" => SimpleBlock::new(1, true);
	BATTERY_LARGE: "battery-large" => SimpleBlock::new(3, true);
	COMBUSTION_GENERATOR: "combustion-generator" => SimpleBlock::new(1, true);
	THERMAL_GENERATOR: "thermal-generator" => SimpleBlock::new(2, true);
	STEAM_GENERATOR: "steam-generator" => SimpleBlock::new(2, true);
	DIFFERENTIAL_GENERATOR: "differential-generator" => SimpleBlock::new(3, true);
	RTG_GENERATOR: "rtg-generator" => SimpleBlock::new(2, true);
	SOLAR_PANEL: "solar-panel" => SimpleBlock::new(1, true);
	SOLAR_PANEL_LARGE: "solar-panel-large" => SimpleBlock::new(3, true);
	THORIUM_REACTOR: "thorium-reactor" => SimpleBlock::new(3, true);
	IMPACT_REACTOR: "impact-reactor" => SimpleBlock::new(4, true);
	POWER_SOURCE: "power-source" => SimpleBlock::new(1, true); // TODO config: destination
	POWER_VOID: "power-void" => SimpleBlock::new(1, true);
);
