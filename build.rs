#![feature(let_chains)]
use image::codecs::png::PngDecoder;
use image::DynamicImage;
use std::fs::File;
use std::io::{BufReader, Write as _};
use std::iter::Iterator;
use std::path::Path;
use walkdir::WalkDir;

macro_rules! wr {
    ($dst:expr => $($arg:tt)*) => { writeln!($dst, $($arg)*).unwrap() };
}

fn kebab2bigsnek(kebab: &str) -> String {
    let mut n = String::new();
    n.reserve(kebab.len());
    for c in kebab.chars() {
        if c == '-' {
            n.push('_');
        } else {
            n.push(c.to_ascii_uppercase());
        }
    }
    n
}

fn main() {
    let _ = std::fs::remove_dir_all("target/out");
    let walkdir = WalkDir::new("assets");
    println!("cargo:rerun-if-changed=assets/");
    println!("cargo:rerun-if-changed=build.rs");
    let o = std::env::var("OUT_DIR").unwrap();
    let o = Path::new(&o);
    let mut full = File::create(o.join("full.rs")).unwrap();
    // let mut half = File::create(o.join("half.rs")).unwrap();
    let mut quar = File::create(o.join("quar.rs")).unwrap();
    let mut eigh = File::create(o.join("eigh.rs")).unwrap();
    let mut n = 23usize;

    wr!(full => "pub mod full {{");
    wr!(full => "pub static EMPTY: LazyLock<RgbaImage> = LazyLock::new(|| RgbaImage::new(32, 32));");

    wr!(quar => "pub mod quar {{");
    wr!(quar => "pub static EMPTY: LazyLock<RgbaImage> = LazyLock::new(|| RgbaImage::new(8, 8));");

    wr!(eigh => "pub mod eigh {{");
    wr!(eigh => "pub static EMPTY: LazyLock<RgbaImage> = LazyLock::new(|| RgbaImage::new(4, 4));");

    for mut file in [&full, &quar, &eigh] {
        file.write_all(b"macro_rules!img{($v:expr)=>{{static TMP:LazyLock<RgbaImage>=LazyLock::new(||$v);&TMP}};}\n").unwrap();
        wr!(file => "use image::RgbaImage;");
        wr!(file => "use crate::utils::Lock as LazyLock;");
        wr!(file => "pub static CLIFF: &LazyLock<RgbaImage> = &EMPTY;");
        for i in 1..=16 {
            wr!(file => "pub static BUILD{}: &LazyLock<RgbaImage> = &EMPTY;", i);
        }
    }
    let mut warmup = File::create(o.join("warmup.rs")).unwrap();
    wr!(warmup => "/// # Safety\n///\n/// this function must only be called once.");
    wr!(warmup => "pub unsafe fn warmup() {{");
    wr!(warmup => "LazyLock::load(&EMPTY);");
    for e in walkdir.into_iter().filter_map(Result::ok) {
        let path = e.path();
        if path.is_file() && let Some(e) = path.extension() && e == "png" {
            let p = DynamicImage::from_decoder(PngDecoder::new(BufReader::new(File::open(path).unwrap())).unwrap()).unwrap().into_rgba8();
            if path.file_name().unwrap().to_str().unwrap().contains("-liquid.png") {
                continue
            }
            let f = path.file_name().unwrap().to_str().unwrap();
            if f.contains("bottom") || f.contains("-team") || f.contains("-end") || f.contains("stack") {
                continue;
            }
            let path = kebab2bigsnek(path.with_extension("").file_name().unwrap().to_str().unwrap());
            if matches!(path.as_str(), "CLIFF_CRUSHER_ROTATOR" | "NEOPLASIA_REACTOR_CENTER" | "FLUX_REACTOR_MID" | "EDGE" | "PHASE_CONVEYOR_BRIDGE" | "BRIDGE_ARROW" | "DUCT_BRIDGE_BRIDGE" | "DUCT_BRIDGE_ARROW" | "LAUNCHPOD" | "BRIDGE_CONVEYOR_BRIDGE" | "BRIDGE_CONVEYOR_ARROW" | "PHASE_CONVEYOR_ARROW" | "REINFORCED_BRIDGE_CONDUIT_ARROW" | "REINFORCED_BRIDGE_CONDUIT_BRIDGE" | "PHASE_CONDUIT_BRIDGE" | "BRIDGE_CONDUIT_ARROW" | "PHASE_CONDUIT_ARROW" | "BRIDGE_CONDUIT_BRIDGE" | "PLATED_CONDUIT_CAP") {
                continue
            }
            macro_rules! writ {
                ($ext:ident / $scale:literal) => {
                    let mut buf = File::create(o.join(n.to_string() + "-" + stringify!($ext))).unwrap();
                    let new = if $scale == 1 {
                        p.clone()
                    } else {
                        // boulders
                        let (mx, my) = if p.width() + p.height() == 48+48 {
                            (32, 32)
                        // vents (dont match VENT_CONDENSER, do match (RHYOLITE_VENT)
                        } else if path.contains("_VENT")
                            // talls
                            || matches!(path.as_str(), "YELLOWCORAL" | "WHITE_TREE" | "WHITE_TREE_DEAD" | "REDWEED" | "SPORE_CLUSTER" | "CRYSTAL_BLOCKS" | "CRYSTAL_CLUSTER" | "VIBRANT_CRYSTAL_CLUSTER" | "CRYSTAL_ORBS") {
                            (32, 32)
                        } else {
                            (p.height(), p.width())
                        };
                        image::imageops::resize(
                            &p,
                            mx / $scale,
                            my / $scale,
                            image::imageops::Nearest,
                        )
                    };
                    let x = new.width();
                    let y = new.height();
                    buf.write_all(&new.into_raw()).unwrap();
                    wr!($ext =>
                        r#"pub(crate) static {path}: &LazyLock<RgbaImage> = img!(unsafe {{ RgbaImage::from_vec({x}, {y}, include_bytes!(concat!(env!("OUT_DIR"), "/{n}-{}")).to_vec()).unwrap_unchecked() }});"#,
                        stringify!($ext)
                    );
                };
            }
            writ!(full / 1);
            // writ!(half + 0.5);
            writ!(quar / 4);
            writ!(eigh / 8);
            wr!(warmup => "LazyLock::load({path});");
            n += 1;
        }
    }
    warmup.write_all(b"}").unwrap();
    for mut f in [full, eigh, quar] {
        // brazillian literal
        f.write_all(br#"include!(concat!(env!("OUT_DIR"), "/warmup.rs"));"#)
            .unwrap();
        f.write_all(b"}").unwrap();
    }
}
