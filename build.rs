#![feature(let_chains)]
use image::codecs::png::PngDecoder;
use image::DynamicImage;
use std::fs::File;
use std::io::{BufReader, Write as _};
use std::iter::Iterator;
use std::path::Path;
use walkdir::WalkDir;

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
    let mut n = 2usize;
    for mut f in [&full, &eigh, &quar] {
        f.write_all(b"phf::phf_map! {\n").unwrap();
    }
    for i in 1..=16 {
        n += 1;
        writeln!(full, r#"  "build{}" => &EMPTY_FULL,"#, i).unwrap();
        writeln!(quar, r#"  "build{}" => &EMPTY_QUAR,"#, i).unwrap();
        writeln!(eigh, r#"  "build{}" => &EMPTY_EIGH,"#, i).unwrap();
    }

    for e in walkdir.into_iter().filter_map(|e| e.ok()) {
        let path = e.path();
        if path.is_file() && let Some(e) = path.extension() && e == "png" {
            let p = DynamicImage::from_decoder(PngDecoder::new(BufReader::new(File::open(path).unwrap())).unwrap()).unwrap().into_rgba8();
            let path = path.with_extension("");
            let path = path.file_name().unwrap().to_str().unwrap();
            macro_rules! writ {
                ($ext:ident / $scale:literal) => {
                    let mut buf = File::create(o.join(n.to_string() + "-" + stringify!($ext))).unwrap();
                    let new = if $scale == 1 {
                        p.clone()
                    } else {
                        // boulders
                        let (mx, my) = if p.width() + p.height() == 48+48 {
                            (32, 32)
                        // vents
                        } else if path.contains("vent") {
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
                    writeln!($ext,
                        r#"  "{path}" => r!(unsafe {{ RgbaImage::from_vec({x}, {y}, include_bytes!(concat!(env!("OUT_DIR"), "/{n}-{}")).to_vec()).unwrap_unchecked() }}),"#,
                        stringify!($ext)
                    ).unwrap();
                };
            }
            writ!(full / 1);
            // writ!(half + 0.5);
            writ!(quar / 4);
            writ!(eigh / 8);
            n += 1;
        }
    }
    for mut f in [full, eigh, quar] {
        f.write_all(b"}").unwrap();
    }
}
