use crate::content::content_enum;

pub mod storage;

content_enum! {
    pub enum Type / Item for u16 | TryFromU16Error {
        "copper",
        "lead",
        "metaglass",
        "graphite",
        "sand",
        "coal",
        "titanium",
        "thorium",
        "scrap",
        "silicon",
        "plastanium",
        "phase-fabric",
        "surge-alloy",
        "spore-pod",
        "blast-compound",
        "pyratite",
        "beryllium",
        "tungsten",
        "oxide",
        "carbide",
        "fissile-matter",
        "dormant-cyst",
    }
}
