use crate::block::make_register;
use crate::block::simple::SimpleBlock;

make_register!
(
	DUO: "duo" => SimpleBlock::new(1, true);
	SCATTER: "scatter" => SimpleBlock::new(2, true);
	SCORCH: "scorch" => SimpleBlock::new(1, true);
	HAIL: "hail" => SimpleBlock::new(1, true);
	WAVE: "wave" => SimpleBlock::new(2, true);
	LANCER: "lancer" => SimpleBlock::new(2, true);
	ARC: "arc" => SimpleBlock::new(1, true);
	PARALLAX: "parallax" => SimpleBlock::new(2, true);
	SWARMER: "swarmer" => SimpleBlock::new(2, true);
	SALVO: "salvo" => SimpleBlock::new(2, true);
	SEGMENT: "segment" => SimpleBlock::new(2, true);
	TSUNAMI: "tsunami" => SimpleBlock::new(3, true);
	FUSE: "fuse" => SimpleBlock::new(3, true);
	RIPPLE: "ripple" => SimpleBlock::new(3, true);
	CYCLONE: "cyclone" => SimpleBlock::new(3, true);
	FORESHADOW: "foreshadow" => SimpleBlock::new(4, true);
	SPECTRE: "spectre" => SimpleBlock::new(4, true);
	MELTDOWN: "meltdown" => SimpleBlock::new(4, true);
);
