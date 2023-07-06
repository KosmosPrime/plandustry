//! the map module
//! ### format
//! note: utf = `len<u16>` + utf8(read(len))
//!
//! note: each section has a `u32` denoting its length
//!
//! key: `: T` and `x<T>` both mean read T, `iterate T` means iterate `read_T()` times
//!
//! ZLIB compressed stream contains:
//! - header: 4b = `MSCH`
//! - version: `u32` (should be 7)
//! - tag section `<u32>`
//!     - 1 byte of idk (skip)
//!     - string map (`u16` for map len, iterate each, read `utf`)
//! - 4 bytes of idk (skip)
//! - content header section `<u32>`:
//!     - iterate `i8` (should = `8`)'//!         - the type: `i8` (0: item, block: 1, liquid: 4, status: 5, unit: 6, weather: 7, sector: 9, planet: 13//!         - item count: `u1\'6` (item: 22, block: 412, liquid: 11, status: 21, unit: 66, weather: 6, sector: 35, planet: 7)
//!         - these types all have their own modules: [`crate::item`], [`crate::block::content`], [`crate::fluid`], [`crate::modifier`], [`crate::unit`], [`crate::data::weather`], [`crate::data::sector`], [`crate::data::planet`]
//!         - iterate `u16`
//!             - name: `utf`
//! - map section `<u32>`
//!     - width: `u16`, height: `u16`
//!     - floor and tiles:
//!         - for `i` in `w * h`
//!             - `x = i % w`, `y = i / w`
//!             - floor id: `u16`
//!             - overlay id: `u16`
//!             - consecutives: `u8`
//!             - iterate `(i + 1)..(i + 1 + consecutives)`
//!                 - `x = j % w`, `y = j / w`
//!             - i += consecutives
//!     - blocks
//!         - for `i` in `w * h`
//!             - block id: `u16`
//!             - packed?: `i8`
//!             - entity = `(packed & 1) not 0`
//!             - data = `(packed & 2) not 0`
//!             - if entity: central: `bool`
//!             - if entity:
//!                 - if central:
//!                     - chunk len: `u16`
//!                     - if block == building:
//!                         - revision: `i8`
//!                         - tile.build.readAll
//!                     - else skip `chunk len`
//!                 - or data
//!                     - data: `i8`
//!                 - else
//!                     - consecutives: `u8`
//!                     - iterate `(i + 1)..(i + 1 + consecutives)`
//!                         - same block
//!                     - i += consecutives
//! - entities section `<u32>`
//!     - entity mapping
//!         - iterate `u16`
//!             - id: `i16`, name: `utf`
//!     - team build plans
//!         - for t in `teams<u32>`
//!             - team = `team#<u32>`
//!             - iterate `plans<u32>`
//!                 - x: `u16`, y: `u16`, rot: `u16`, id: `u16`
//!                 - o: `DynData` (refer to [crate::data::dynamic::DynSerializer])
//!         - world entities
//!             - iterate `u32`
//!                 - len: `u16`
//!                 - type: `u8`
//!                 - if !mapping\[type\]
//!                     - skip(len - 1)
//!                     - continue
//!                 - id: `u32`
//!                 - entity read
use std::collections::HashMap;

use crate::block::content::Type as BlockEnum;
use crate::block::{Block, BlockRegistry, Rotation};
use crate::data::dynamic::DynSerializer;
use crate::data::renderer::*;
use crate::data::DataRead;
use crate::fluid::Type as Fluid;
use crate::item::storage::Storage;
use crate::item::Type as Item;
use crate::team::Team;

use super::schematic::{ReadError, WriteError};
use super::GridPos;
use super::Serializer;
use crate::content::Content;
use crate::utils::image::ImageUtils;

/// a tile in a map
pub struct Tile<'l> {
    pub pos: GridPos,
    pub floor: &'l Block,
    pub ore: Option<&'l Block>,
    pub build: Option<Build<'l>>,
}

pub type EntityMapping = HashMap<u8, Box<dyn Content>>;
impl<'l> Tile<'l> {
    fn set_block(&mut self, block: &'l Block) {
        self.build = Some(Build {
            block,
            items: Storage::new(),
            liquids: Storage::new(),
            rotation: Rotation::Up,
            team: crate::team::SHARDED,
            data: 0,
        });
    }

    /// check if this tile contains a building.
    pub fn has_building(&self) -> bool {
        if let Some(b) = &self.build {
            return b.block.has_building();
        }
        false
    }

    /// size of this tile
    ///
    /// ._.
    ///
    /// dont think about it too much
    pub fn size(&self) -> u8 {
        if let Some(b) = &self.build {
            return b.block.get_size();
        }
        1
    }

    pub fn image(&self) -> ImageHolder {
        // building covers floore
        let i = if let Some(b) = &self.build {
            b.image()
        } else {
            let mut i = self.floor.image(None).own();
            if let Some(ore) = self.ore {
                i.overlay(ore.image(None).borrow(), 0, 0);
            }
            ImageHolder::from(i)
        };
        i
    }
}

impl std::fmt::Debug for Tile<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Tile<{},{}>@{}+{}{}",
            self.pos.0,
            self.pos.1,
            self.floor.name(),
            if let Some(ore) = &self.ore {
                ore.name()
            } else {
                ""
            },
            if let Some(build) = &self.build {
                format!(":{}", build.block.name())
            } else {
                "".to_string()
            }
        )
    }
}

/// a build on a tile in a map
#[derive(Debug)]
pub struct Build<'l> {
    pub block: &'l Block,
    pub items: Storage<Item>,
    pub liquids: Storage<Fluid>,
    // pub health: f32,
    pub rotation: Rotation,
    pub team: Team,
    pub data: i8,
}

impl Build<'_> {
    pub fn image(&self) -> ImageHolder {
        self.block.image(None)
    }

    pub fn read(
        &mut self,
        buff: &mut DataRead<'_>,
        reg: &BlockRegistry,
        map: &EntityMapping,
    ) -> Result<(), ReadError> {
        // health
        let _ = buff.read_f32()?; // 4
        let rot = dbg!(buff.read_u8()?); // 5
        self.rotation = Rotation::try_from(rot & 127).unwrap_or(Rotation::Up);
        if (rot & 128) == 0 {
            return Err(ReadError::Version(rot & 128));
        }

        let _t = dbg!(buff.read_u8()?); // 6
        let _v = dbg!(buff.read_u8()?); // 7
        let mask = dbg!(buff.read_u8()?); // 8
        if dbg!((mask & 1) != 0) {
            self.items.clear();
            // 10
            for _ in 0..dbg!(buff.read_u16()?) {
                let item = buff.read_u16()?;
                let amount = buff.read_u32()?;
                if let Ok(item) = Item::try_from(item) {
                    self.items.set(item, amount);
                }
            }
        }
        if mask & 2 == 0 {
            let n = buff.read_u16()? as usize;
            buff.skip((n * 4) + 1)?;
        }
        if mask & 4 == 0 {
            self.liquids.clear();
            for _ in 0..buff.read_u16()? {
                let fluid = buff.read_u16()?;
                let amount = buff.read_f32()?;
                if let Ok(fluid) = Fluid::try_from(fluid) {
                    self.liquids.set(fluid, (amount * 100.0) as u32);
                }
            }
        }
        // "efficiency"?
        let _ = buff.read_u8()?;
        let _ = buff.read_u8()?;
        // visible flags
        let _ = buff.read_i64()?;
        // "overriden by subclasses"
        self.block.read(buff, reg, map)?;
        Ok(())
    }
}

/// a map
#[derive(Debug)]
pub struct Map<'l> {
    pub width: u32,
    pub height: u32,
    pub tags: HashMap<String, String>,
    pub tiles: Vec<Tile<'l>>,
}

const MAP_HEADER: u32 = 0x4d534156;

/// serde map
pub struct MapSerializer<'l>(pub &'l BlockRegistry<'l>);
impl<'l> Serializer<Map<'l>> for MapSerializer<'l> {
    type ReadError = ReadError;
    type WriteError = WriteError;
    fn deserialize(&mut self, buff: &mut DataRead<'_>) -> Result<Map<'l>, Self::ReadError> {
        let buff = buff.deflate()?;
        let mut buff = DataRead::new(&buff);
        let hdr = buff.read_u32()?;
        if hdr != MAP_HEADER {
            return Err(ReadError::Header(hdr));
        }
        let version = buff.read_u32()?;
        if version != 7 {
            return Err(ReadError::Version(version.try_into().unwrap_or(0)));
        }
        let mut tags = HashMap::new();
        buff.read_chunk(|buff| {
            buff.skip(1)?;
            for _ in 0..buff.read_u8()? {
                let key = buff.read_utf()?;
                let value = buff.read_utf()?;
                tags.insert(key.to_owned(), value.to_owned());
            }
            Ok::<(), super::ReadError>(())
        })?;
        buff.read_chunk(|buff| {
            // we skip these (just keep the respective modules updated)
            for _ in 0..buff.read_i8()? {
                // let _ty = buff.read_u8()?;
                // for _ in 0..buff.read_i16()? {
                //     let name = dbg!(buff.read_utf()?);
                // }
                buff.skip(1)?;
                for _ in 0..buff.read_u16()? {
                    let n = buff.read_u16()?;
                    buff.skip(n as usize)?;
                }
            }
            Ok::<(), super::ReadError>(())
        })?;

        //  map section
        let mut w = 0;
        let mut h = 0;
        let mut tiles = vec![];
        buff.read_chunk(|buff| {
            w = buff.read_u16()? as u32;
            h = buff.read_u16()? as u32;
            let count = w * h;
            let mut i = 0;
            while i < count {
                let x = (i % w) as u16;
                let y = (i / w) as u16;
                let floor_id = buff.read_u16()?;
                let overlay_id = buff.read_u16()?;
                let floor = BlockEnum::try_from(floor_id)
                    .unwrap_or(BlockEnum::Stone)
                    .to(self.0)
                    .unwrap_or(&crate::block::environment::STONE);
                let ore = BlockEnum::try_from(overlay_id)
                    .unwrap_or(BlockEnum::Air)
                    .to(self.0);
                debug_assert!(
                    x < w as u16 && y < h as u16,
                    "{x} or {y} out of bounds ({floor:?} {ore:?})"
                );
                tiles.push(Tile {
                    floor,
                    ore,
                    pos: GridPos(x, y),
                    build: None,
                });
                let consecutives = buff.read_u8()? as u32;
                if consecutives > 0 {
                    for i in (i + 1)..(i + 1 + consecutives) {
                        let x = (i % w) as u16;
                        let y = (i / w) as u16;
                        tiles.push(Tile {
                            floor,
                            ore,
                            pos: GridPos(x, y),
                            build: None,
                        })
                    }
                    i += consecutives;
                }
                i += 1;
            }
            let mut i = 0usize;
            while i < count as usize {
                let block_id = buff.read_u16()?;
                let packed = buff.read_u8()?;
                let entity = (packed & 1) != 0;
                let data = (packed & 2) != 0;
                let central = if entity { buff.read_bool()? } else { false };
                let block = BlockEnum::try_from(block_id)
                    .map_err(|_| ReadError::NoSuchBlock(block_id.to_string()))?;
                let block = if block != BlockEnum::Air {
                    Some(
                        self.0
                            .get(block.get_name())
                            .ok_or(ReadError::NoSuchBlock(block.to_string()))?,
                    )
                } else {
                    None
                };
                if central {
                    if let Some(block) = block {
                        tiles[i].set_block(block);
                    }
                }
                if entity {
                    if central {
                        // we dont bother skipping if not building - mindustry supports
                        // all kinds of broken saves, but i dont have to
                        let n = buff.read_u16()?;
                        buff.skip(n as usize)?;
                        // let _ = buff.read_i8()?;
                        // tiles[i]
                        //     .build
                        //     .as_mut()
                        //     .unwrap()
                        //     // map not initialized yet
                        //     .read(&mut buff, self.0, &HashMap::new())?;
                    }
                } else if data {
                    if let Some(block) = block {
                        tiles[i].set_block(block);
                    }
                    tiles[i].build.as_mut().unwrap().data = buff.read_i8()?;
                } else {
                    let consecutives = buff.read_u8()? as usize;
                    for tile in tiles.iter_mut().take(consecutives).skip(i + 1) {
                        if let Some(block) = block {
                            tile.set_block(block);
                        }
                    }
                    i += consecutives;
                }
                i += 1
            }
            Ok::<(), ReadError>(())
        })?;
        let mut mapping = EntityMapping::new();
        buff.read_chunk(|buff| {
            for _ in 0..buff.read_u16()? {
                let id = buff.read_i16()? as u8;
                let nam = buff.read_utf()?;
                dbg!(nam);
                mapping.insert(id, Box::new(Item::Copper));
                // mapping.push(content::Type::get_name(nam));
            }
            for _ in 0..buff.read_u32()? {
                buff.skip(4)?;
                for _ in 0..buff.read_u32()? {
                    buff.skip(8usize)?;
                    let _ = DynSerializer::deserialize(&mut DynSerializer, buff)?;
                }
            }
            for _ in 0..buff.read_u32()? {
                let len = buff.read_u16()? as usize;
                let ty = buff.read_u8()?;
                if !mapping.contains_key(&ty) {
                    buff.skip(len - 1)?;
                    continue;
                }
                let _id = buff.read_u32()?;
                // TODO
            }
            Ok::<(), ReadError>(())
        })?;
        // skip custom chunks
        buff.skip_chunk()?;
        println!("remaining bytes: {}", buff.data.len());
        // println!("{:?}", tiles);
        Ok(Map {
            width: w,
            height: h,
            tags,
            tiles,
        })
    }

    fn serialize(
        &mut self,
        _: &mut super::DataWrite<'_>,
        _: &Map<'_>,
    ) -> Result<(), Self::WriteError> {
        todo!()
    }
}
