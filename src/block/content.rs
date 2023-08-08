//! everything
use crate::content::{content_enum};

content_enum! {
    pub enum Type / Block for u16 | TryFromU16Error
    {
        "air",
        "spawn",
        "cliff",
        "build1",
        "build2",
        "build3",
        "build4",
        "build5",
        "build6",
        "build7",
        "build8",
        "build9",
        "build10",
        "build11",
        "build12",
        "build13",
        "build14",
        "build15",
        "build16",
        "deep-water",
        "shallow-water",
        "tainted-water",
        "deep-tainted-water",
        "darksand-tainted-water",
        "sand-water",
        "darksand-water",
        "tar",
        "pooled-cryofluid",
        "molten-slag",
        "space",
        "empty",
        "stone",
        "crater-stone",
        "char",
        "basalt",
        "hotrock",
        "magmarock",
        "sand-floor",
        "darksand",
        "dirt",
        "mud",
        "dacite",
        "rhyolite",
        "rhyolite-crater",
        "rough-rhyolite",
        "regolith",
        "yellow-stone",
        "carbon-stone",
        "ferric-stone",
        "ferric-craters",
        "beryllic-stone",
        "crystalline-stone",
        "crystal-floor",
        "yellow-stone-plates",
        "red-stone",
        "dense-red-stone",
        "red-ice",
        "arkycite-floor",
        "arkyic-stone",
        "rhyolite-vent",
        "carbon-vent",
        "arkyic-vent",
        "yellow-stone-vent",
        "red-stone-vent",
        "crystalline-vent",
        "redmat",
        "bluemat",
        "grass",
        "salt",
        "snow",
        "ice",
        "ice-snow",
        "shale",
        "moss",
        "core-zone",
        "spore-moss",
        "stone-wall",
        "spore-wall",
        "dirt-wall",
        "dacite-wall",
        "ice-wall",
        "snow-wall",
        "dune-wall",
        "regolith-wall",
        "yellow-stone-wall",
        "rhyolite-wall",
        "carbon-wall",
        "ferric-stone-wall",
        "beryllic-stone-wall",
        "arkyic-wall",
        "crystalline-stone-wall",
        "red-ice-wall",
        "red-stone-wall",
        "red-diamond-wall",
        "sand-wall",
        "salt-wall",
        "shrubs",
        "shale-wall",
        "spore-pine",
        "snow-pine",
        "pine",
        "white-tree-dead",
        "white-tree",
        "spore-cluster",
        "redweed",
        "pur-bush",
        "yellowcoral",
        "boulder",
        "snow-boulder",
        "shale-boulder",
        "sand-boulder",
        "dacite-boulder",
        "basalt-boulder",
        "carbon-boulder",
        "ferric-boulder",
        "beryllic-boulder",
        "yellow-stone-boulder",
        "arkyic-boulder",
        "crystal-cluster",
        "vibrant-crystal-cluster",
        "crystal-blocks",
        "crystal-orbs",
        "crystalline-boulder",
        "red-ice-boulder",
        "rhyolite-boulder",
        "red-stone-boulder",
        "metal-floor",
        "metal-floor-damaged",
        "metal-floor-2",
        "metal-floor-3",
        "metal-floor-4",
        "metal-floor-5",
        "dark-panel-1",
        "dark-panel-2",
        "dark-panel-3",
        "dark-panel-4",
        "dark-panel-5",
        "dark-panel-6",
        "dark-metal",
        "pebbles",
        "tendrils",
        "ore-copper",
        "ore-lead",
        "ore-scrap",
        "ore-coal",
        "ore-titanium",
        "ore-thorium",
        "ore-beryllium",
        "ore-tungsten",
        "ore-crystal-thorium",
        "ore-wall-thorium",
        "ore-wall-beryllium",
        "graphitic-wall",
        "ore-wall-tungsten",
        "graphite-press",
        "multi-press",
        "silicon-smelter",
        "silicon-crucible",
        "kiln",
        "plastanium-compressor",
        "phase-weaver",
        "surge-smelter",
        "cryofluid-mixer",
        "pyratite-mixer",
        "blast-mixer",
        "melter",
        "separator",
        "disassembler",
        "spore-press",
        "pulverizer",
        "coal-centrifuge",
        "incinerator",
        "silicon-arc-furnace",
        "electrolyzer",
        "atmospheric-concentrator",
        "oxidation-chamber",
        "electric-heater",
        "slag-heater",
        "phase-heater",
        "heat-redirector",
        "heat-router",
        "slag-incinerator",
        "carbide-crucible",
        "slag-centrifuge",
        "surge-crucible",
        "cyanogen-synthesizer",
        "phase-synthesizer",
        "heat-reactor",
        "copper-wall",
        "copper-wall-large",
        "titanium-wall",
        "titanium-wall-large",
        "plastanium-wall",
        "plastanium-wall-large",
        "thorium-wall",
        "thorium-wall-large",
        "phase-wall",
        "phase-wall-large",
        "surge-wall",
        "surge-wall-large",
        "door",
        "door-large",
        "scrap-wall",
        "scrap-wall-large",
        "scrap-wall-huge",
        "scrap-wall-gigantic",
        "thruster",
        "beryllium-wall",
        "beryllium-wall-large",
        "tungsten-wall",
        "tungsten-wall-large",
        "blast-door",
        "reinforced-surge-wall",
        "reinforced-surge-wall-large",
        "carbide-wall",
        "carbide-wall-large",
        "shielded-wall",
        "mender",
        "mend-projector",
        "overdrive-projector",
        "overdrive-dome",
        "force-projector",
        "shock-mine",
        "radar",
        "build-tower",
        "regen-projector",
        "shockwave-tower",
        "shield-projector",
        "large-shield-projector",
        "conveyor",
        "titanium-conveyor",
        "plastanium-conveyor",
        "armored-conveyor",
        "junction",
        "bridge-conveyor",
        "phase-conveyor",
        "sorter",
        "inverted-sorter",
        "router",
        "distributor",
        "overflow-gate",
        "underflow-gate",
        "mass-driver",
        "duct",
        "armored-duct",
        "duct-router",
        "overflow-duct",
        "underflow-duct",
        "duct-bridge",
        "duct-unloader",
        "surge-conveyor",
        "surge-router",
        "unit-cargo-loader",
        "unit-cargo-unload-point",
        "mechanical-pump",
        "rotary-pump",
        "impulse-pump",
        "conduit",
        "pulse-conduit",
        "plated-conduit",
        "liquid-router",
        "liquid-container",
        "liquid-tank",
        "liquid-junction",
        "bridge-conduit",
        "phase-conduit",
        "reinforced-pump",
        "reinforced-conduit",
        "reinforced-liquid-junction",
        "reinforced-bridge-conduit",
        "reinforced-liquid-router",
        "reinforced-liquid-container",
        "reinforced-liquid-tank",
        "power-node",
        "power-node-large",
        "surge-tower",
        "diode",
        "battery",
        "battery-large",
        "combustion-generator",
        "thermal-generator",
        "steam-generator",
        "differential-generator",
        "rtg-generator",
        "solar-panel",
        "solar-panel-large",
        "thorium-reactor",
        "impact-reactor",
        "beam-node",
        "beam-tower",
        "beam-link",
        "turbine-condenser",
        "chemical-combustion-chamber",
        "pyrolysis-generator",
        "flux-reactor",
        "neoplasia-reactor",
        "mechanical-drill",
        "pneumatic-drill",
        "laser-drill",
        "blast-drill",
        "water-extractor",
        "cultivator",
        "oil-extractor",
        "vent-condenser",
        "cliff-crusher",
        "plasma-bore",
        "large-plasma-bore",
        "impact-drill",
        "eruption-drill",
        "core-shard",
        "core-foundation",
        "core-nucleus",
        "core-bastion",
        "core-citadel",
        "core-acropolis",
        "container",
        "vault",
        "unloader",
        "reinforced-container",
        "reinforced-vault",
        "duo",
        "scatter",
        "scorch",
        "hail",
        "wave",
        "lancer",
        "arc",
        "parallax",
        "swarmer",
        "salvo",
        "segment",
        "tsunami",
        "fuse",
        "ripple",
        "cyclone",
        "foreshadow",
        "spectre",
        "meltdown",
        "breach",
        "diffuse",
        "sublimate",
        "titan",
        "disperse",
        "afflict",
        "lustre",
        "scathe",
        "smite",
        "malign",
        "ground-factory",
        "air-factory",
        "naval-factory",
        "additive-reconstructor",
        "multiplicative-reconstructor",
        "exponential-reconstructor",
        "tetrative-reconstructor",
        "repair-point",
        "repair-turret",
        "tank-fabricator",
        "ship-fabricator",
        "mech-fabricator",
        "tank-refabricator",
        "mech-refabricator",
        "ship-refabricator",
        "prime-refabricator",
        "tank-assembler",
        "ship-assembler",
        "mech-assembler",
        "basic-assembler-module",
        "unit-repair-tower",
        "payload-conveyor",
        "payload-router",
        "reinforced-payload-conveyor",
        "reinforced-payload-router",
        "payload-mass-driver",
        "large-payload-mass-driver",
        "small-deconstructor",
        "deconstructor",
        "constructor",
        "large-constructor",
        "payload-loader",
        "payload-unloader",
        "power-source",
        "power-void",
        "item-source",
        "item-void",
        "liquid-source",
        "liquid-void",
        "payload-source",
        "payload-void",
        "heat-source",
        "illuminator",
        "legacy-mech-pad",
        "legacy-unit-factory",
        "legacy-unit-factory-air",
        "legacy-unit-factory-ground",
        "command-center",
        "launch-pad",
        "interplanetary-accelerator",
        "message",
        "switch",
        "micro-processor",
        "logic-processor",
        "hyper-processor",
        "memory-cell",
        "memory-bank",
        "logic-display",
        "large-logic-display",
        "canvas",
        "reinforced-message",
        "world-processor",
        "world-cell",
        "world-message",
    }
}

