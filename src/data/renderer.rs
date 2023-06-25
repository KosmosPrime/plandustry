use std::io::BufReader;
use std::path::Path;

use image::codecs::png::PngDecoder;
use image::imageops::overlay;
use image::{DynamicImage, RgbaImage};

use super::schematic::Schematic;

pub fn load(category: &str, name: &str) -> Option<RgbaImage> {
    let mut p = Path::new("assets/blocks").join(category).join(name);
    p.set_extension("png");
    let f = std::fs::File::open(p).ok()?;
    let r = PngDecoder::new(BufReader::new(f)).unwrap();
    Some(DynamicImage::from_decoder(r).unwrap().into_rgba8())
}

const SUFFIXES: &[&str; 8] = &[
    "bottom", "mid", "", "-base", "-left", "-right", "-top", "-over",
];
pub fn read<S>(category: &str, name: &str, size: S) -> RgbaImage
where
    S: Into<u32> + Copy,
{
    let mut c = RgbaImage::new(size.into() * 32, size.into() * 32);
    for suffix in SUFFIXES {
        let mut p = Path::new("assets/blocks")
            .join(category)
            .join(format!("{name}{suffix}"));
        p.set_extension("png");
        if let Some(p) = load(category, &format!("{name}{suffix}")) {
            image::imageops::overlay(&mut c, &p, 0, 0);
        }
    }
    c
}

pub struct Renderer {}
impl<'l> Renderer {
    pub fn render(s: &'l Schematic<'_>) -> RgbaImage {
        let mut canvas = RgbaImage::new((s.width * 32).into(), (s.height * 32).into());
        for tile in s.block_iter() {
            let mut x = tile.pos.0 as i64;
            let mut y = tile.pos.1 as i64;
            if tile.block.get_size() != 1 && tile.block.get_size() % 2 != 0 {
                x -= 1;
                y -= 1;
            }
            overlay(&mut canvas, &tile.image(), x * 32, y * 32);
        }
        canvas
    }
}
