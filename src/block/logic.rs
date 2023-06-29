//! logic processors and stuff
use std::borrow::Cow;
use std::error::Error;
use std::fmt;
use std::string::FromUtf8Error;

use flate2::{
    Compress, CompressError, Compression, Decompress, DecompressError, FlushCompress,
    FlushDecompress, Status,
};

use super::State;
use crate::block::simple::{cost, state_impl, BuildCost, SimpleBlock};
use crate::block::{
    impl_block, make_register, BlockLogic, DataConvertError, DeserializeError, SerializeError,
};
use crate::data::dynamic::{DynData, DynType};
use crate::data::{self, DataRead, DataWrite, GridPos};
use crate::item::storage::Storage;

make_register! {
    // todo reinforced proc
    "reinforced-message" => MessageLogic::new(1, true, cost!(Graphite: 10, Beryllium: 5));
    "message" => MessageLogic::new(1, true, cost!(Copper: 5, Graphite: 5));
    "switch" => SwitchLogic::new(1, true, cost!(Copper: 5, Graphite: 5));
    "micro-processor" => ProcessorLogic::new(1, true, cost!(Copper: 90, Lead: 50, Silicon: 50));
    "logic-processor" => ProcessorLogic::new(2, true, cost!(Lead: 320, Graphite: 60, Thorium: 50, Silicon: 80));
    "hyper-processor" => ProcessorLogic::new(3, true, cost!(Lead: 450, Thorium: 75, Silicon: 150, SurgeAlloy: 50));
    "memory-cell" => SimpleBlock::new(1, true, cost!(Copper: 30, Graphite: 30, Silicon: 30));
    "memory-bank" => SimpleBlock::new(2, true, cost!(Copper: 30, Graphite: 80, Silicon: 80, PhaseFabric: 30));
    "logic-display" => SimpleBlock::new(3, true, cost!(Lead: 100, Metaglass: 50, Silicon: 50));
    "large-logic-display" => SimpleBlock::new(6, true, cost!(Lead: 200, Metaglass: 100, Silicon: 150, PhaseFabric: 75));
}

pub struct MessageLogic {
    size: u8,
    symmetric: bool,
    build_cost: BuildCost,
}

impl MessageLogic {
    #[must_use]
    pub const fn new(size: u8, symmetric: bool, build_cost: BuildCost) -> Self {
        assert!(size != 0, "invalid size");
        Self {
            size,
            symmetric,
            build_cost,
        }
    }

    state_impl!(pub String);
}

impl BlockLogic for MessageLogic {
    impl_block!();

    fn data_from_i32(&self, _: i32, _: GridPos) -> Result<DynData, DataConvertError> {
        Ok(DynData::Empty)
    }

    fn deserialize_state(&self, data: DynData) -> Result<Option<State>, DeserializeError> {
        match data {
            DynData::Empty | DynData::String(None) => Ok(Some(Self::create_state(String::new()))),
            DynData::String(Some(s)) => Ok(Some(Self::create_state(s))),
            _ => Err(DeserializeError::InvalidType {
                have: data.get_type(),
                expect: DynType::String,
            }),
        }
    }

    fn clone_state(&self, state: &State) -> State {
        Box::new(Self::get_state(state).clone())
    }

    fn mirror_state(&self, _: &mut State, _: bool, _: bool) {}

    fn rotate_state(&self, _: &mut State, _: bool) {}

    fn serialize_state(&self, state: &State) -> Result<DynData, SerializeError> {
        Ok(DynData::String(Some(Self::get_state(state).clone())))
    }
}

pub struct SwitchLogic {
    size: u8,
    symmetric: bool,
    build_cost: BuildCost,
}

impl SwitchLogic {
    #[must_use]
    pub const fn new(size: u8, symmetric: bool, build_cost: BuildCost) -> Self {
        assert!(size != 0, "invalid size");
        Self {
            size,
            symmetric,
            build_cost,
        }
    }

    state_impl!(pub bool);
}

impl BlockLogic for SwitchLogic {
    impl_block!();

    fn data_from_i32(&self, _: i32, _: GridPos) -> Result<DynData, DataConvertError> {
        Ok(DynData::Empty)
    }

    fn deserialize_state(&self, data: DynData) -> Result<Option<State>, DeserializeError> {
        match data {
            DynData::Empty => Ok(Some(Self::create_state(true))),
            DynData::Boolean(enabled) => Ok(Some(Self::create_state(enabled))),
            _ => Err(DeserializeError::InvalidType {
                have: data.get_type(),
                expect: DynType::Boolean,
            }),
        }
    }

    fn clone_state(&self, state: &State) -> State {
        Box::new(*Self::get_state(state))
    }

    fn mirror_state(&self, _: &mut State, _: bool, _: bool) {}

    fn rotate_state(&self, _: &mut State, _: bool) {}

    fn serialize_state(&self, state: &State) -> Result<DynData, SerializeError> {
        Ok(DynData::Boolean(*Self::get_state(state)))
    }
}

pub struct ProcessorLogic {
    size: u8,
    symmetric: bool,
    build_cost: BuildCost,
}

impl ProcessorLogic {
    #[must_use]
    pub const fn new(size: u8, symmetric: bool, build_cost: BuildCost) -> Self {
        assert!(size != 0, "invalid size");
        Self {
            size,
            symmetric,
            build_cost,
        }
    }

    state_impl!(pub ProcessorState);
}

impl BlockLogic for ProcessorLogic {
    impl_block!();

    fn data_from_i32(&self, _: i32, _: GridPos) -> Result<DynData, DataConvertError> {
        Ok(DynData::Empty)
    }

    fn deserialize_state(&self, data: DynData) -> Result<Option<State>, DeserializeError> {
        match data {
            DynData::Empty => Ok(Some(Self::create_state(ProcessorState::default()))),
            DynData::ByteArray(arr) => {
                let mut input = arr.as_ref();
                let mut dec = Decompress::new(true);
                let mut raw = Vec::<u8>::new();
                raw.reserve(1024);
                loop {
                    let t_in = dec.total_in();
                    let t_out = dec.total_out();
                    let res = ProcessorDeserializeError::forward(dec.decompress_vec(
                        input,
                        &mut raw,
                        FlushDecompress::Finish,
                    ))?;
                    if dec.total_in() > t_in {
                        // we have to advance input every time, decompress_vec only knows the output position
                        input = &input[(dec.total_in() - t_in) as usize..];
                    }
                    match res {
                        // there's no more input (and the flush mode says so), we need to reserve additional space
                        Status::Ok | Status::BufError => (),
                        // input was already at the end, so this is referring to the output
                        Status::StreamEnd => break,
                    }
                    if dec.total_in() == t_in && dec.total_out() == t_out {
                        // protect against looping forever
                        return Err(DeserializeError::Custom(Box::new(
                            ProcessorDeserializeError::DecompressStall,
                        )));
                    }
                    raw.reserve(1024);
                }
                let mut buff = DataRead::new(&raw);
                let ver = ProcessorDeserializeError::forward(buff.read_u8())?;
                if ver != 1 {
                    return Err(DeserializeError::Custom(Box::new(
                        ProcessorDeserializeError::Version(ver),
                    )));
                }

                let code_len = ProcessorDeserializeError::forward(buff.read_i32())?;
                if !(0..=500 * 1024).contains(&code_len) {
                    return Err(DeserializeError::Custom(Box::new(
                        ProcessorDeserializeError::CodeLength(code_len),
                    )));
                }
                let mut code = Vec::<u8>::new();
                code.resize(code_len as usize, 0);
                ProcessorDeserializeError::forward(buff.read_bytes(&mut code))?;
                let code = ProcessorDeserializeError::forward(String::from_utf8(code))?;
                let link_cnt = ProcessorDeserializeError::forward(buff.read_i32())?;
                if link_cnt < 0 {
                    return Err(DeserializeError::Custom(Box::new(
                        ProcessorDeserializeError::LinkCount(link_cnt),
                    )));
                }
                let mut links = Vec::<ProcessorLink>::new();
                links.reserve(link_cnt as usize);
                for _ in 0..link_cnt {
                    let name = ProcessorDeserializeError::forward(buff.read_utf())?;
                    let x = ProcessorDeserializeError::forward(buff.read_i16())?;
                    let y = ProcessorDeserializeError::forward(buff.read_i16())?;
                    links.push(ProcessorLink {
                        name: String::from(name),
                        x,
                        y,
                    });
                }
                Ok(Some(Self::create_state(ProcessorState { code, links })))
            }
            _ => Err(DeserializeError::InvalidType {
                have: data.get_type(),
                expect: DynType::Boolean,
            }),
        }
    }

    fn clone_state(&self, state: &State) -> State {
        Box::new(Self::get_state(state).clone())
    }

    fn mirror_state(&self, state: &mut State, horizontally: bool, vertically: bool) {
        for link in &mut Self::get_state_mut(state).links {
            if horizontally {
                link.x = -link.x;
            }
            if vertically {
                link.y = -link.y;
            }
        }
    }

    fn rotate_state(&self, state: &mut State, clockwise: bool) {
        for link in &mut Self::get_state_mut(state).links {
            let (cdx, cdy) = link.get_pos();
            link.x = if clockwise { cdy } else { -cdy };
            link.y = if clockwise { -cdx } else { cdx };
        }
    }

    fn serialize_state(&self, state: &State) -> Result<DynData, SerializeError> {
        let state = Self::get_state(state);
        let mut rbuff = DataWrite::default();
        ProcessorSerializeError::forward(rbuff.write_u8(1))?;
        assert!(state.code.len() < 500 * 1024);
        ProcessorSerializeError::forward(rbuff.write_i32(state.code.len() as i32))?;
        ProcessorSerializeError::forward(rbuff.write_bytes(state.code.as_bytes()))?;
        assert!(state.links.len() < i32::MAX as usize);
        ProcessorSerializeError::forward(rbuff.write_i32(state.links.len() as i32))?;
        for link in &state.links {
            ProcessorSerializeError::forward(rbuff.write_utf(&link.name))?;
            ProcessorSerializeError::forward(rbuff.write_i16(link.x))?;
            ProcessorSerializeError::forward(rbuff.write_i16(link.y))?;
        }
        let mut input = rbuff.get_written();
        let mut comp = Compress::new(Compression::default(), true);
        let mut dst = Vec::<u8>::new();
        dst.reserve(1024);
        loop {
            let t_in = comp.total_in();
            let t_out = comp.total_out();
            let res = ProcessorSerializeError::forward(comp.compress_vec(
                input,
                &mut dst,
                FlushCompress::Finish,
            ))?;
            if comp.total_in() > t_in {
                // we have to advance input every time, compress_vec only knows the output position
                input = &input[(comp.total_in() - t_in) as usize..];
            }
            match res {
                // there's no more input (and the flush mode says so), we need to reserve additional space
                Status::Ok | Status::BufError => (),
                // input was already at the end, so this is referring to the output
                Status::StreamEnd => break,
            }
            if comp.total_in() == t_in && comp.total_out() == t_out {
                // protect against looping forever
                return Err(SerializeError::Custom(Box::new(
                    ProcessorSerializeError::CompressStall,
                )));
            }
            dst.reserve(1024);
        }
        Ok(DynData::ByteArray(dst))
    }
}

#[derive(Debug)]
pub enum ProcessorDeserializeError {
    Read(data::ReadError),
    Decompress(DecompressError),
    DecompressStall,
    FromUtf8(FromUtf8Error),
    Version(u8),
    CodeLength(i32),
    LinkCount(i32),
}

impl ProcessorDeserializeError {
    pub fn forward<T, E: Into<Self>>(result: Result<T, E>) -> Result<T, DeserializeError> {
        match result {
            Ok(v) => Ok(v),
            Err(e) => Err(DeserializeError::Custom(Box::new(e.into()))),
        }
    }
}

impl From<data::ReadError> for ProcessorDeserializeError {
    fn from(value: data::ReadError) -> Self {
        Self::Read(value)
    }
}

impl From<DecompressError> for ProcessorDeserializeError {
    fn from(value: DecompressError) -> Self {
        Self::Decompress(value)
    }
}

impl From<FromUtf8Error> for ProcessorDeserializeError {
    fn from(value: FromUtf8Error) -> Self {
        Self::FromUtf8(value)
    }
}

impl fmt::Display for ProcessorDeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read(..) => f.write_str("failed to read state data"),
            Self::Decompress(..) => f.write_str("zlib decompression failed"),
            Self::DecompressStall => f.write_str("decompressor stalled before completion"),
            Self::FromUtf8(..) => f.write_str("malformed utf-8 in processor code"),
            Self::Version(ver) => write!(f, "unsupported version ({ver})"),
            Self::CodeLength(len) => write!(f, "invalid code length ({len})"),
            Self::LinkCount(cnt) => write!(f, "invalid link count ({cnt})"),
        }
    }
}

impl Error for ProcessorDeserializeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Decompress(e) => Some(e),
            Self::FromUtf8(e) => Some(e),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum ProcessorSerializeError {
    Write(data::WriteError),
    Compress(CompressError),
    CompressEof(usize),
    CompressStall,
}

impl ProcessorSerializeError {
    pub fn forward<T, E: Into<Self>>(result: Result<T, E>) -> Result<T, SerializeError> {
        match result {
            Ok(v) => Ok(v),
            Err(e) => Err(SerializeError::Custom(Box::new(e.into()))),
        }
    }
}

impl From<data::WriteError> for ProcessorSerializeError {
    fn from(value: data::WriteError) -> Self {
        Self::Write(value)
    }
}

impl From<CompressError> for ProcessorSerializeError {
    fn from(value: CompressError) -> Self {
        Self::Compress(value)
    }
}

impl fmt::Display for ProcessorSerializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Write(..) => f.write_str("failed to write state data"),
            Self::Compress(..) => f.write_str("zlib compression failed"),
            Self::CompressEof(remain) => write!(
                f,
                "compression overflow with {remain} bytes of input remaining"
            ),
            Self::CompressStall => f.write_str("compressor stalled before completion"),
        }
    }
}

impl Error for ProcessorSerializeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Compress(e) => Some(e),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ProcessorLink {
    name: String,
    x: i16,
    y: i16,
}

impl ProcessorLink {
    #[must_use]
    pub fn new(name: Cow<'_, str>, x: i16, y: i16) -> Self {
        assert!(
            u16::try_from(name.len()).is_ok(),
            "name too long ({})",
            name.len()
        );
        Self {
            name: name.into_owned(),
            x,
            y,
        }
    }

    #[must_use]
    pub fn get_name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn get_pos(&self) -> (i16, i16) {
        (self.x, self.y)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ProcessorState {
    code: String,
    links: Vec<ProcessorLink>,
}

impl ProcessorState {
    #[must_use]
    pub fn get_code(&self) -> &str {
        &self.code
    }

    pub fn set_code(&mut self, code: Cow<'_, str>) -> Result<(), CodeError> {
        let as_str = &code as &str;
        if as_str.len() > 500 * 1024 {
            return Err(CodeError::TooLong(as_str.len()));
        }
        match code {
            Cow::Borrowed(s) => {
                self.code.clear();
                self.code.push_str(s);
            }
            Cow::Owned(s) => self.code = s,
        }
        Ok(())
    }

    #[must_use]
    pub fn get_links(&self) -> &[ProcessorLink] {
        &self.links
    }

    pub fn create_link(
        &mut self,
        mut name: String,
        x: i16,
        y: i16,
    ) -> Result<&ProcessorLink, CreateError> {
        if name.len() > u16::MAX as usize {
            return Err(CreateError::NameLength(name.len()));
        }
        for curr in &self.links {
            if name == curr.name {
                return Err(CreateError::DuplicateName(name));
            }
            if x == curr.x && y == curr.y {
                name.clear();
                name.push_str(&curr.name);
                return Err(CreateError::DuplicatePos { name, x, y });
            }
        }
        let idx = self.links.len();
        self.links.push(ProcessorLink { name, x, y });
        Ok(&self.links[idx])
    }

    pub fn add_link(&mut self, link: ProcessorLink) -> Result<&ProcessorLink, CreateError> {
        self.create_link(link.name, link.x, link.y)
    }

    pub fn remove_link(&mut self, idx: usize) -> Option<ProcessorLink> {
        if idx < self.links.len() {
            Some(self.links.remove(idx))
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CodeError {
    TooLong(usize),
}

impl fmt::Display for CodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLong(len) => write!(f, "code too long ({len} bytes)"),
        }
    }
}

impl Error for CodeError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CreateError {
    NameLength(usize),
    DuplicateName(String),
    DuplicatePos { name: String, x: i16, y: i16 },
}

impl fmt::Display for CreateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NameLength(len) => write!(f, "link name too long ({len} bytes)"),
            Self::DuplicateName(name) => write!(f, "there already is a link named {name}"),
            Self::DuplicatePos { name, x, y } => {
                write!(f, "link {name} already points to {x} / {y}")
            }
        }
    }
}

impl Error for CreateError {}
