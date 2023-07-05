//! schematic drawing
use std::io::{BufReader, Cursor};
use std::path::Path;

use image::codecs::png::PngDecoder;
use image::imageops::overlay;
use image::{DynamicImage, RgbaImage};
use zip::ZipArchive;

use crate::block::environment::METAL_FLOOR;
use crate::data::map::Tile;
use crate::team::SHARDED;
use crate::utils::ImageUtils;
use crate::Map;

use super::schematic::Schematic;

pub(crate) fn load(category: &str, name: &str) -> Option<RgbaImage> {
    let mut p = Path::new("blocks").join(category).join(name);
    p.set_extension("png");
    load_raw(p)
}

pub(crate) fn load_raw(f: impl AsRef<Path>) -> Option<RgbaImage> {
    let f = std::fs::File::open(Path::new("target/out").join(f)).ok()?;
    let r = PngDecoder::new(BufReader::new(f)).unwrap();
    Some(DynamicImage::from_decoder(r).unwrap().into_rgba8())
}

fn load_zip() {
    if !Path::new("target/out").exists() {
        let mut zip = ZipArchive::new(Cursor::new(
            include_bytes!(concat!(env!("OUT_DIR"), "/asset")).to_vec(),
        ))
        .unwrap();
        zip.extract("target/out").unwrap();
    }
}
pub const TOP: &str = "-top";
const SUFFIXES: &[&str; 9] = &[
    "-bottom", "-mid", "", "-base", "-left", "-right", TOP, "-over", "-team",
];
pub(crate) fn read<S>(category: &str, name: &str, size: S) -> RgbaImage
where
    S: Into<u32> + Copy,
{
    read_with(category, name, SUFFIXES, size)
}

pub(crate) fn read_with<S>(
    category: &str,
    name: &str,
    suffixes: &'static [&'static str],
    size: S,
) -> RgbaImage
where
    S: Into<u32> + Copy,
{
    let mut c = RgbaImage::new(size.into() * 32, size.into() * 32);
    for suffix in suffixes {
        if let Some(mut p) = load(category, &format!("{name}{suffix}")) {
            if suffix == &"-team" {
                p.tint(SHARDED.color());
            }
            image::imageops::overlay(&mut c, &p, 0, 0);
        }
    }
    c
}

/// renderer for creating images of schematics
pub struct Renderer {}
impl<'l> Renderer {
    /// creates a picture of a schematic. Bridges and node connections are not drawn, and there is no background.
    /// conveyors, conduits, and ducts currently do not render.
    /// ```
    /// use mindus::*;
    /// let mut s = Schematic::new(2, 3);
    /// s.put(0, 0, &block::distribution::DISTRIBUTOR);
    /// s.put(0, 3, &block::distribution::ROUTER);
    /// s.put(1, 3, &block::walls::COPPER_WALL);
    /// let output /*: RgbaImage */ = Renderer::render(&s);
    /// ```
    pub fn render_schematic(s: &'l Schematic<'_>) -> RgbaImage {
        load_zip();
        let mut canvas = RgbaImage::new((s.width * 32).into(), (s.height * 32).into());
        // fill background
        canvas.repeat(&METAL_FLOOR.image(None));
        for tile in s.block_iter() {
            let x = (tile.pos.0 - ((tile.block.get_size() - 1) / 2) as u16) as i64;
            let y = (s.height - tile.pos.1 - ((tile.block.get_size() / 2) + 1) as u16) as i64;
            canvas.overlay(&tile.image(), x * 32, y * 32);
        }
        canvas
    }

    pub fn render_map(m: &'l Map<'_>) -> RgbaImage {
        load_zip();
        let mut canvas = RgbaImage::new((m.width * 8).into(), (m.height * 8).into());
        const VEC: Vec<&Tile<'_>> = vec![];
        let mut layers = [VEC; 2];
        for tile in m.tiles.iter() {
            if tile.has_building() {
                layers[1].push(tile)
            } else {
                layers[0].push(tile)
            }
        }
        for tiles in layers {
            for tile in tiles {
                let s = if let Some(build) = &tile.build {
                    build.block.get_size()
                } else {
                    1
                };
                let x = (tile.pos.0 - ((s - 1) / 2) as u16) as i64;
                let y = (m.height as u16 - tile.pos.1 - ((s / 2) + 1) as u16) as i64;
                overlay(
                    &mut canvas,
                    tile.image().scale(tile.size() as u32 * 8),
                    x * 8,
                    y * 8,
                );
            }
        }
        canvas
    }
}
