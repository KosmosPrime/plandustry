use std::error::Error;
use std::fmt;
use std::str::Utf8Error;

pub mod base64;
pub mod command;
pub mod dynamic;
pub mod schematic;

pub struct DataRead<'d> {
    data: &'d [u8],
}

macro_rules! make_read {
    ($name:ident, $type:ty) => {
        pub fn $name(&mut self) -> Result<$type, ReadError> {
            const LEN: usize = std::mem::size_of::<$type>();
            if self.data.len() < LEN {
                return Err(ReadError::Underflow {
                    need: LEN,
                    have: self.data.len(),
                });
            }
            let mut output = [0u8; LEN];
            output.copy_from_slice(&self.data[..LEN]);
            self.data = &self.data[LEN..];
            Ok(<$type>::from_be_bytes(output))
        }
    };
}

impl<'d> DataRead<'d> {
    #[must_use] pub fn new(data: &'d [u8]) -> Self {
        Self { data }
    }

    pub fn read_bool(&mut self) -> Result<bool, ReadError> {
        Ok(self.read_u8()? != 0)
    }

    make_read!(read_u8, u8);
    make_read!(read_i8, i8);
    make_read!(read_u16, u16);
    make_read!(read_i16, i16);
    make_read!(read_u32, u32);
    make_read!(read_i32, i32);
    make_read!(read_f32, f32);
    make_read!(read_u64, u64);
    make_read!(read_i64, i64);
    make_read!(read_f64, f64);

    pub fn read_utf(&mut self) -> Result<&'d str, ReadError> {
        if self.data.len() < 2 {
            return Err(ReadError::Underflow {
                need: 2,
                have: self.data.len(),
            });
        }
        let len = u16::from_be_bytes([self.data[0], self.data[1]]);
        let end = 2 + len as usize;
        if self.data.len() < end {
            return Err(ReadError::Underflow {
                need: end,
                have: self.data.len(),
            });
        }
        let result = std::str::from_utf8(&self.data[2..end])?;
        self.data = &self.data[end..];
        Ok(result)
    }

    pub fn read_bytes(&mut self, dst: &mut [u8]) -> Result<(), ReadError> {
        if self.data.len() < dst.len() {
            return Err(ReadError::Underflow {
                need: dst.len(),
                have: self.data.len(),
            });
        }
        dst.copy_from_slice(&self.data[..dst.len()]);
        self.data = &self.data[dst.len()..];
        Ok(())
    }

    pub fn read_vec(&mut self, dst: &mut Vec<u8>, len: usize) -> Result<(), ReadError> {
        if self.data.len() < len {
            return Err(ReadError::Underflow {
                need: len,
                have: self.data.len(),
            });
        }
        dst.extend_from_slice(&self.data[..len]);
        self.data = &self.data[len..];
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReadError {
    Underflow { need: usize, have: usize },
    Utf8(Utf8Error),
}

impl From<Utf8Error> for ReadError {
    fn from(err: Utf8Error) -> Self {
        Self::Utf8(err)
    }
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Underflow { need, have } => {
                write!(f, "buffer underflow (expected {need} but got {have})")
            }
            Self::Utf8(..) => f.write_str("malformed utf-8 in string"),
        }
    }
}

impl Error for ReadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Utf8(e) => Some(e),
            _ => None,
        }
    }
}

enum WriteBuff<'d> {
    // unlike the DataRead want to access the written region after
    Ref { raw: &'d mut [u8], pos: usize },
    Vec(Vec<u8>),
}

impl<'d> WriteBuff<'d> {
    fn check_capacity(&self, need: usize) -> Result<(), WriteError> {
        match self {
            Self::Ref { raw, pos } if raw.len() - pos < need => Err(WriteError::Overflow {
                need,
                have: raw.len() - pos,
            }),
            _ => Ok(()),
        }
    }

    fn write(&mut self, data: &[u8]) {
        match self {
            Self::Ref { raw, pos } => {
                let end = *pos + data.len();
                raw[*pos..end].copy_from_slice(data);
                *pos += data.len();
            }
            Self::Vec(v) => v.extend_from_slice(data),
        }
    }
}

pub struct DataWrite<'d> {
    data: WriteBuff<'d>,
}

macro_rules! make_write {
    ($name:ident, $type:ty) => {
        pub fn $name(&mut self, val: $type) -> Result<(), WriteError> {
            const LEN: usize = std::mem::size_of::<$type>();
            self.data.check_capacity(LEN)?;
            self.data.write(&<$type>::to_be_bytes(val));
            Ok(())
        }
    };
}

impl<'d> DataWrite<'d> {
    pub fn write_bool(&mut self, val: bool) -> Result<(), WriteError> {
        self.write_u8(u8::from(val))
    }

    make_write!(write_u8, u8);
    make_write!(write_i8, i8);
    make_write!(write_u16, u16);
    make_write!(write_i16, i16);
    make_write!(write_u32, u32);
    make_write!(write_i32, i32);
    make_write!(write_f32, f32);
    make_write!(write_u64, u64);
    make_write!(write_i64, i64);
    make_write!(write_f64, f64);

    pub fn write_utf(&mut self, val: &str) -> Result<(), WriteError> {
        if val.len() > u16::MAX as usize {
            return Err(WriteError::TooLong { len: val.len() });
        }
        self.data.check_capacity(2 + val.len())?;
        self.data.write(&u16::to_be_bytes(val.len() as u16));
        self.data.write(val.as_bytes());
        Ok(())
    }

    pub fn write_bytes(&mut self, val: &[u8]) -> Result<(), WriteError> {
        self.data.check_capacity(val.len())?;
        self.data.write(val);
        Ok(())
    }

    #[must_use] pub fn is_owned(&self) -> bool {
        matches!(self.data, WriteBuff::Vec(..))
    }

    #[must_use] pub fn get_written(&self) -> &[u8] {
        match &self.data {
            WriteBuff::Ref { raw, pos } => &raw[..*pos],
            WriteBuff::Vec(v) => v,
        }
    }
}

impl Default for DataWrite<'static> {
    fn default() -> Self {
        Self {
            data: WriteBuff::Vec(Vec::new()),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WriteError {
    Overflow { need: usize, have: usize },
    TooLong { len: usize },
}

impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Overflow { need, have } => {
                write!(f, "buffer overflow (expected {need} but got {have})")
            }
            Self::TooLong { len } => write!(f, "string too long ({len} bytes of {})", u16::MAX),
        }
    }
}

impl Error for WriteError {}

impl<'d> From<&'d mut [u8]> for DataWrite<'d> {
    fn from(value: &'d mut [u8]) -> Self {
        Self {
            data: WriteBuff::Ref { raw: value, pos: 0 },
        }
    }
}

impl From<Vec<u8>> for DataWrite<'static> {
    fn from(value: Vec<u8>) -> Self {
        Self {
            data: WriteBuff::Vec(value),
        }
    }
}

impl<'d> TryFrom<DataWrite<'d>> for Vec<u8> {
    type Error = ();

    fn try_from(value: DataWrite<'d>) -> Result<Self, Self::Error> {
        match value.data {
            WriteBuff::Vec(v) => Ok(v),
            _ => Err(()),
        }
    }
}

pub trait Serializer<D> {
    type ReadError;
    type WriteError;

    fn deserialize(&mut self, buff: &mut DataRead<'_>) -> Result<D, Self::ReadError>;

    fn serialize(&mut self, buff: &mut DataWrite<'_>, data: &D) -> Result<(), Self::WriteError>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GridPos(pub u16, pub u16);

impl From<u32> for GridPos {
    fn from(value: u32) -> Self {
        GridPos((value >> 16) as u16, value as u16)
    }
}

impl From<GridPos> for u32 {
    fn from(value: GridPos) -> Self {
        (u32::from(value.0) << 16) | u32::from(value.1)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn read() {
        let mut read = DataRead::new("Thé qûick ઉrown fox 🦘 over\0\rthe lazy dog.".as_bytes());
        assert_eq!(read.read_u8(), Ok(84));
        assert_eq!(read.read_i8(), Ok(104));
        assert_eq!(read.read_i8(), Ok(-61));
        assert_eq!(read.read_u16(), Ok(43296));
        assert_eq!(read.read_i16(), Ok(29123));
        assert_eq!(read.read_i16(), Ok(-17559));
        assert_eq!(read.read_i32(), Ok(1_667_965_152));
        assert_eq!(read.read_i32(), Ok(-1_433_832_849));
        assert_eq!(read.read_i64(), Ok(8_605_851_562_280_493_296));
        assert_eq!(read.read_i64(), Ok(-6_942_694_510_468_635_278));
        assert_eq!(read.read_utf(), Ok("the lazy dog."));
    }

    #[test]
    fn write() {
        let mut write = DataWrite::default();
        assert_eq!(write.write_u8(84), Ok(()));
        assert_eq!(write.write_i8(104), Ok(()));
        assert_eq!(write.write_i8(-61), Ok(()));
        assert_eq!(write.write_u16(43296), Ok(()));
        assert_eq!(write.write_i16(29123), Ok(()));
        assert_eq!(write.write_i16(-17559), Ok(()));
        assert_eq!(write.write_i32(1_667_965_152), Ok(()));
        assert_eq!(write.write_i32(-1_433_832_849), Ok(()));
        assert_eq!(write.write_i64(8_605_851_562_280_493_296), Ok(()));
        assert_eq!(write.write_i64(-6_942_694_510_468_635_278), Ok(()));
        assert_eq!(write.write_utf("the lazy dog."), Ok(()));
        assert_eq!(
            write.get_written(),
            "Thé qûick ઉrown fox 🦘 over\0\rthe lazy dog.".as_bytes()
        );
    }
}
