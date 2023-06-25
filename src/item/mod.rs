use image::Rgb;

pub mod storage;

macro_rules! item_enum {
	($($val:literal: $col:literal),* $(,)?) =>
	{
		paste::paste! {
		$crate::content::content_enum!(pub enum Type / Item for u16 | TryFromU16Error {
			$($val),*,
		});

        impl Type {
            pub fn color(&self) -> Rgb<u8> {
                match &self {
                    $(Self::[<$val:camel>] => {
                        Rgb(color_hex::color_from_hex!($col))
                    },)*
                }
            }
        }
    }}
}

item_enum! {
    "copper": "d99d73",
    "lead": "8c7fa9",
    "metaglass": "ebeef5",
    "graphite": "b2c6d2",
    "sand": "f7cba4",
    "coal": "272727",
    "titanium": "8da1e3",
    "thorium": "f9a3c7",
    "scrap": "777777",
    "silicon": "53565c",
    "plastanium": "cbd97f",
    "phase-fabric": "f4ba6e",
    "surge-alloy": "f3e979",
    "spore-pod": "7457ce",
    "blast-compound": "ff795e",
    "pyratite": "ffaa5f",
    "beryllium": "3a8f64",
    "tungsten": "768a9a",
    "oxide": "e4ffd6",
    "carbide": "89769a",
    "fissile-matter": "5e988d",
    "dormant-cyst": "df824d",
}
