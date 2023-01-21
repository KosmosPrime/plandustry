use crate::block::make_register;
use crate::block::simple::SimpleBlock;
use crate::block::transport::ItemBlock;

make_register!
(
	MENDER: "mender" => SimpleBlock::new(1, true);
	MEND_PROJECTOR: "mend-projector" => SimpleBlock::new(2, true);
	OVERDRIVE_PROJECTOR: "overdrive-projector" => SimpleBlock::new(2, true);
	OVERDRIVE_DOME: "overdrive-dome" => SimpleBlock::new(3, true);
	FORCE_PROJECTOR: "force-projector" => SimpleBlock::new(3, true);
	SHOCK_MINE: "shock-mine" => SimpleBlock::new(1, true);
	CORE_SHARD: "core-shard" => SimpleBlock::new(3, true);
	CORE_FOUNDATION: "core-foundation" => SimpleBlock::new(4, true);
	CORE_NUCLEUS: "core-nucleus" => SimpleBlock::new(5, true);
	CONTAINER: "container" => SimpleBlock::new(2, true);
	VAULT: "vault" => SimpleBlock::new(3, true);
	UNLOADER: "unloader" => ItemBlock::new(1, true);
	ILLUMINATOR: "illuminator" => SimpleBlock::new(1, true); // TODO config: color
	LAUNCH_PAD: "launch-pad" => SimpleBlock::new(3, true);
);
