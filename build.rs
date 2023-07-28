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
    let mut f = File::create(Path::new(&o).join("asset")).unwrap();
    let mut n = 1usize;
    f.write_all(b"fn put(map: &DashMap<String, RgbaImage>) {")
        .unwrap();
    let mut s = String::new(); // idk write_all / write wasnt working
    for e in walkdir.into_iter().filter_map(|e| e.ok()) {
        let path = e.path();
        if path.is_file() && let Some(e) = path.extension() && e == "png" {
            let p = DynamicImage::from_decoder(PngDecoder::new(BufReader::new(File::open(path).unwrap())).unwrap()).unwrap().into_rgba8();
            let x = p.width();
            let y = p.height();
            let path = path.with_extension("");
            let path = path.file_name().unwrap().to_str().unwrap();
            let mut f = File::create(Path::new(&o).join(n.to_string())).unwrap();
            f.write_all(&p.into_raw()).unwrap();
            println!("writing {path:?}");
            s+= &format!("\tmap.insert(String::from(\"{path}\"), RgbaImage::from_vec({x}, {y}, include_bytes!(concat!(env!(\"OUT_DIR\"), \"/{n}\")).to_vec()).unwrap());\n");
            n += 1;
        }
    }
    f.write_all(s.as_bytes()).unwrap();
    f.write_all(b"}").unwrap();
}
