use std::{
    collections::BTreeMap,
    io::{
        Cursor,
        Read,
        Seek,
        SeekFrom,
    },
    string::FromUtf8Error,
};

use bitflags::bitflags;
use byteorder::{
    LittleEndian,
    ReadBytesExt,
};
use thiserror::Error;

use crate::util::read_cstring;

#[derive(Debug, Error)]
pub enum KV3Error {
    #[error("io error")]
    IOError(#[from] std::io::Error),

    #[error("encoding error")]
    Utf8Error(#[from] FromUtf8Error),

    #[error("version {0} is not supported")]
    UnsupportedVersion(usize),

    #[error("invalid kv3 block magic")]
    InvalidMagic,

    #[error("compression method {0} not supported")]
    CompressionUnsupported(u32),

    #[error("compression parameter invalid")]
    CompressionInvalidParameter,

    #[error("decompression result length does not match exected length")]
    CompressionDecodeInvalidLength,

    #[error("invalid block trailer")]
    InvalidBlockTrailer,

    #[error("expected type but type list exceeded")]
    KV3MissingType,

    #[error("expected flag info but type list exceeded")]
    KV3MissingFlagInfo,

    #[error("kv3 flags {0} unknown")]
    KV3FlagsInvalid(u8),

    #[error("kv3 type {0} unknown")]
    KV3TypeUnknown(u8),

    #[error("kv3 byte data has been exceeded")]
    KV3BytesExceeded,

    #[error("kv3 value for type {0:?} is not supported")]
    KV3ValueNotSuported(KVType),

    #[error("missing string table entry for {0}")]
    KV3MissingStringTableEntry(u32),

    #[error("missing block index")]
    KV3MissingBlockIndex,

    #[error("the max depth of nested objects has been exceeded")]
    KV3MaxDepthExceeded,
}

struct ResourceHeader {
    _version: u32,
    _format: [u8; 16],

    compression_method: u32,
    compression_dictionary_id: u16,
    compression_frame_size: u16,

    count_of_bytes: u32,
    count_of_integers: u32,
    count_of_quads: u32,

    string_and_types_buffer_size: u32,

    uncompressed_size: u32,
    compressed_size: u32,
    block_count: u32,
    block_total_size: u32,
}

impl ResourceHeader {
    pub fn read_header<R>(reader: &mut R, version: u32) -> Result<Self, KV3Error>
    where
        R: Read,
    {
        let mut format: [u8; 16] = [0u8; 0x10];
        reader.read_exact(&mut format)?;

        let compression_method = reader.read_u32::<LittleEndian>()?;
        let compression_dictionary_id = reader.read_u16::<LittleEndian>()?;
        let compression_frame_size = reader.read_u16::<LittleEndian>()?;

        let count_of_bytes = reader.read_u32::<LittleEndian>()?;
        let count_of_integers = reader.read_u32::<LittleEndian>()?;
        let count_of_quads = reader.read_u32::<LittleEndian>()?;

        let string_and_types_buffer_size = reader.read_u32::<LittleEndian>()?;
        let _ = reader.read_u16::<LittleEndian>()?;
        let _ = reader.read_u16::<LittleEndian>()?;

        let uncompressed_size = reader.read_u32::<LittleEndian>()?;
        let compressed_size = reader.read_u32::<LittleEndian>()?;
        let block_count = reader.read_u32::<LittleEndian>()?;
        let block_total_size = reader.read_u32::<LittleEndian>()?;

        if version >= 0x05 {
            let _ = reader.read_u32::<LittleEndian>()?;
            let _ = reader.read_u32::<LittleEndian>()?;
        }

        Ok(Self {
            _version: version,
            _format: format,

            compression_method,
            compression_dictionary_id,
            compression_frame_size,

            count_of_bytes,
            count_of_integers,
            count_of_quads,

            string_and_types_buffer_size,

            uncompressed_size,
            compressed_size,
            block_count,
            block_total_size,
        })
    }
}

// Version 3 plus, payload
#[derive(Default)]
struct V3PPayload {
    offset_bytes: u64,
    offset_integers: u64,
    offset_quads: u64,

    offset_kv_data: u64,

    strings: Vec<String>,
    types: Vec<u8>,

    payload: Vec<u8>,

    block_lengths: Vec<u32>,
    block_data: Vec<u8>,
}

fn pad_bytes(length: i64, padding: i64) -> i64 {
    (length + padding - 1) / padding * padding
}

fn decompress_kv3_v3p_playload<R>(
    reader: &mut R,
    header: &ResourceHeader,
) -> Result<V3PPayload, KV3Error>
where
    R: Read,
{
    let mut compressed_buffer = Vec::new();
    compressed_buffer.resize(header.compressed_size as usize, 0u8);
    reader.read_exact(&mut compressed_buffer)?;

    let mut decompressed_reader = match header.compression_method {
        0x00 => {
            /* no compression */
            Cursor::new(compressed_buffer)
        }
        0x01 => return Err(KV3Error::CompressionUnsupported(0x01)),
        0x02 => {
            if header.compression_dictionary_id != 0 || header.compression_frame_size != 0 {
                log::warn!(
                    "zstd decompress compression_dictionary_id = {}, compression_frame_size = {}",
                    header.compression_dictionary_id,
                    header.compression_frame_size
                );
                return Err(KV3Error::CompressionInvalidParameter);
            }

            let mut reader = Cursor::new(compressed_buffer);
            let decoded = zstd::decode_all(&mut reader)?;
            if decoded.len() != (header.uncompressed_size + header.block_total_size) as usize {
                return Err(KV3Error::CompressionDecodeInvalidLength);
            }

            Cursor::new(decoded)
        }
        method => return Err(KV3Error::CompressionUnsupported(method)),
    };

    let mut payload = V3PPayload::default();
    log::debug!(
        "block_count = {}, count_of_bytes = {}, count_of_integers = {}, count_of_quads = {}",
        header.block_count,
        header.count_of_bytes,
        header.count_of_integers,
        header.count_of_quads
    );

    /* align to the next 4byte boundary */
    payload.offset_bytes = decompressed_reader.stream_position()?;
    decompressed_reader.seek(SeekFrom::Current(pad_bytes(
        header.count_of_bytes as i64,
        0x04,
    )))?;

    payload.offset_integers = decompressed_reader.stream_position()?;
    let count_of_strings = decompressed_reader.read_u32::<LittleEndian>()?; // Always the first integer
    payload.offset_kv_data = decompressed_reader.stream_position()?;
    decompressed_reader.seek(SeekFrom::Current(pad_bytes(
        (header.count_of_integers * 4 - 4) as i64,
        0x08,
    )))?;

    payload.offset_quads = decompressed_reader.stream_position()?;
    decompressed_reader.seek(SeekFrom::Current(header.count_of_quads as i64 * 0x08))?;

    let offset_strings = decompressed_reader.stream_position()?;
    payload.strings.reserve(count_of_strings as usize);
    for _ in 0..count_of_strings {
        let entry = read_cstring::<_, KV3Error>(&mut decompressed_reader)?;
        payload.strings.push(entry);
    }

    let types_length = header.string_and_types_buffer_size as usize
        - (decompressed_reader.stream_position()? - offset_strings) as usize;
    payload.types.resize(types_length, 0u8);
    decompressed_reader.read_exact(&mut payload.types)?;

    payload.block_lengths.reserve(header.block_count as usize);
    for _ in 0..header.block_count {
        payload
            .block_lengths
            .push(decompressed_reader.read_u32::<LittleEndian>()?);
    }
    if decompressed_reader.read_u32::<LittleEndian>()? != 0xFFEEDD00 {
        return Err(KV3Error::InvalidBlockTrailer);
    }

    match header.compression_method {
        0x00 => {
            let total_size = payload.block_lengths.iter().sum::<u32>() as usize;
            payload.block_data.resize(total_size, 0u8);
            reader.read_exact(&mut payload.block_data)?;
        }
        0x02 => {
            let total_size = payload.block_lengths.iter().sum::<u32>() as usize;
            payload.block_data.resize(total_size, 0u8);
            decompressed_reader.read_exact(&mut payload.block_data)?;
        }
        method => return Err(KV3Error::CompressionUnsupported(method)),
    };

    payload.payload = decompressed_reader.into_inner();
    Ok(payload)
}

bitflags! {
    #[derive(Debug)]
    pub struct KVFlag : u8 {
        const NONE = 0x00;
        const RESOURCE = 0x01;
        const RESOURCE_NAME = 0x02;
        const PANORAMA = 0x08;
        const SOUND_EVENT = 0x10;
        const SUB_CLASS = 0x20;
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum KVType {
    StringMulti,
    Null,
    Boolean,
    Int64,
    UInt64,
    Double,
    String,
    BinaryBlob,
    Array,
    Object,
    ArrayTyped,
    Int32,
    UInt32,
    BoolTrue,
    BoolFalse,
    Int64Zero,
    Int64One,
    DoubleZero,
    DoubleOne,
    Float,
    Unknown20,
    Unknown21,
    Unknown22,
    Int32AsByte,
    ArrayTypeByteLength,
}

impl KVType {
    pub fn from_value(value: u8) -> Option<KVType> {
        Some(match value {
            0x00 => Self::StringMulti,
            0x01 => Self::Null,
            0x02 => Self::Boolean,
            0x03 => Self::Int64,
            0x04 => Self::UInt64,
            0x05 => Self::Double,
            0x06 => Self::String,
            0x07 => Self::BinaryBlob,
            0x08 => Self::Object,
            0x09 => Self::Object,
            0x0A => Self::ArrayTyped,
            0x0B => Self::Int32,
            0x0C => Self::UInt32,
            0x0D => Self::BoolTrue,
            0x0E => Self::BoolFalse,
            0x0F => Self::Int64Zero,
            0x10 => Self::Int64One,
            0x11 => Self::DoubleZero,
            0x12 => Self::DoubleOne,
            0x13 => Self::Float,
            0x14 => Self::Unknown20,
            0x15 => Self::Unknown21,
            0x16 => Self::Unknown22,
            0x17 => Self::Int32AsByte,
            0x18 => Self::ArrayTypeByteLength,
            _ => return None,
        })
    }
}

#[derive(Debug, serde::Serialize)]
#[serde(untagged)]
pub enum KV3Value {
    Null,
    Bool(bool),
    Int32(u32),
    Int64(u64),
    Float(f32),
    Double(f64),
    String(String),
    Binary(Vec<u8>),
    Array(Vec<KV3Value>),
    Object(BTreeMap<String, KV3Value>),
}

impl KV3Value {
    pub fn as_bool(&self) -> Option<bool> {
        if let Self::Bool(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    pub fn as_u32(&self) -> Option<u32> {
        if let Self::Int32(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Self::Int32(value) => Some(*value as u64),
            Self::Int64(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_f32(&self) -> Option<f32> {
        if let Self::Float(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Float(value) => Some(*value as f64),
            Self::Double(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        if let Self::String(value) = self {
            Some(&*value)
        } else {
            None
        }
    }

    pub fn as_binary(&self) -> Option<&[u8]> {
        if let Self::Binary(value) = self {
            Some(&*value)
        } else {
            None
        }
    }

    pub fn as_array(&self) -> Option<&[KV3Value]> {
        if let Self::Array(value) = self {
            Some(&*value)
        } else if let Self::Object(values) = self {
            if values.is_empty() {
                /* an empty array is sometimes represented as an empty object */
                Some(&[])
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn as_vec3_f64(&self) -> Option<nalgebra::Vector3<f64>> {
        let values = self
            .as_array()?
            .iter()
            .map(|v| v.as_f64())
            .try_collect::<Vec<_>>()?;

        if values.len() != 3 {
            return None;
        }

        Some(nalgebra::Vector3::from_column_slice(&values))
    }

    pub fn get(&self, member: &str) -> Option<&KV3Value> {
        if let Self::Object(members) = self {
            members.get(member)
        } else {
            None
        }
    }
}

struct V3PParserState<'a> {
    version: u32,
    payload: &'a V3PPayload,

    payload_reader: Cursor<&'a [u8]>,
    block_reader: Cursor<&'a [u8]>,

    current_type_index: usize,
    current_bytes_offset: u64,
    current_integer_offset: u64,
    current_quad_offset: u64,
    current_block_index: usize,

    depth: usize,
}

impl V3PParserState<'_> {
    fn read_value_u8(&mut self) -> Result<u8, KV3Error> {
        let offset = self.payload.offset_bytes + self.current_bytes_offset;
        self.payload_reader.seek(SeekFrom::Start(offset))?;

        let value = self.payload_reader.read_u8()?;
        self.current_bytes_offset += 1;

        Ok(value)
    }

    fn read_value_u32(&mut self) -> Result<u32, KV3Error> {
        let offset = self.payload.offset_integers + self.current_integer_offset;
        self.payload_reader.seek(SeekFrom::Start(offset))?;

        let value = self.payload_reader.read_u32::<LittleEndian>()?;
        self.current_integer_offset += 4;

        Ok(value)
    }

    fn read_value_u64(&mut self) -> Result<u64, KV3Error> {
        let offset = self.payload.offset_quads + self.current_quad_offset;
        self.payload_reader.seek(SeekFrom::Start(offset))?;

        let value = self.payload_reader.read_u64::<LittleEndian>()?;
        self.current_quad_offset += 8;

        Ok(value)
    }

    fn read_value_string(&mut self) -> Result<String, KV3Error> {
        let id = self.read_value_u32()?;
        if id == 0xFFFFFFFF {
            Ok("".to_string())
        } else {
            let value = self
                .payload
                .strings
                .get(id as usize)
                .ok_or(KV3Error::KV3MissingStringTableEntry(id))?;

            Ok(value.to_string())
        }
    }

    fn read_value_type(&mut self) -> Result<u8, KV3Error> {
        let value = *self
            .payload
            .types
            .get(self.current_type_index)
            .ok_or(KV3Error::KV3MissingFlagInfo)?;

        self.current_type_index += 1;
        Ok(value)
    }

    pub fn parse_next_entry(&mut self) -> Result<KV3Value, KV3Error> {
        if self.depth >= 0x10 {
            return Err(KV3Error::KV3MaxDepthExceeded);
        }

        let (kv_type, _kv_flags) = self.read_type()?;
        self.parse_value(kv_type)
    }

    fn parse_value(&mut self, ktype: KVType) -> Result<KV3Value, KV3Error> {
        Ok(match ktype {
            KVType::Null => KV3Value::Null,
            KVType::Boolean => KV3Value::Bool(self.read_value_u8()? > 0),
            KVType::BoolFalse => KV3Value::Bool(false),
            KVType::BoolTrue => KV3Value::Bool(true),

            KVType::Double => KV3Value::Double(self.read_value_u64()? as f64),
            KVType::DoubleZero => KV3Value::Double(0.0),
            KVType::DoubleOne => KV3Value::Double(1.0),

            KVType::Int64 | KVType::UInt64 => KV3Value::Int64(self.read_value_u64()?),
            KVType::Int64Zero => KV3Value::Int64(0),
            KVType::Int64One => KV3Value::Int64(1),

            KVType::Int32 | KVType::UInt32 => KV3Value::Int32(self.read_value_u32()?),
            KVType::Float => KV3Value::Float(self.read_value_u32()? as f32),

            KVType::String | KVType::StringMulti => KV3Value::String(self.read_value_string()?),

            KVType::BinaryBlob => {
                let length = self
                    .payload
                    .block_lengths
                    .get(self.current_block_index)
                    .ok_or(KV3Error::KV3MissingBlockIndex)?;
                self.current_block_index += 1;

                let mut buffer = Vec::new();
                buffer.resize(*length as usize, 0u8);

                self.block_reader.read_exact(&mut buffer)?;

                KV3Value::Binary(buffer)
            }

            KVType::Array => {
                let length = self.read_value_u32()?;
                let mut elements = Vec::with_capacity(length as usize);

                self.depth += 1;
                for _ in 0..length {
                    elements.push(self.parse_next_entry()?)
                }
                self.depth -= 1;

                KV3Value::Array(elements)
            }

            KVType::Object => {
                let member_count = self.read_value_u32()?;
                let mut members = BTreeMap::<String, KV3Value>::new();

                self.depth += 1;
                for _ in 0..member_count {
                    let field = self.read_value_string()?;
                    members.insert(field, self.parse_next_entry()?);
                }
                self.depth -= 1;

                KV3Value::Object(members)
            }

            KVType::ArrayTyped | KVType::ArrayTypeByteLength => {
                let length = if ktype == KVType::ArrayTypeByteLength {
                    self.read_value_u8()? as usize
                } else {
                    self.read_value_u32()? as usize
                };

                let (kv_type, _kv_flags) = self.read_type()?;
                let mut elements = Vec::with_capacity(length);

                self.depth += 1;
                for _ in 0..length {
                    elements.push(self.parse_value(kv_type)?)
                }
                self.depth -= 1;

                KV3Value::Array(elements)
            }

            ktype => return Err(KV3Error::KV3ValueNotSuported(ktype)),
        })
    }

    fn read_type(&mut self) -> Result<(KVType, KVFlag), KV3Error> {
        let mut data = self.read_value_type()?;
        let mut flag_info = KVFlag::NONE;

        if self.version >= 0x05 {
            if data & 0x80 > 0 {
                data &= 0x3F;

                let flags = self.read_value_type()?;
                flag_info = KVFlag::from_bits(flags).ok_or(KV3Error::KV3FlagsInvalid(flags))?;
            }
        } else if data & 0x80 > 0 {
            data &= 0x7F;

            let mut flags = self.read_value_type()?;
            if flags & 0x04 > 0 {
                data = 0;
                flags &= !0x04;
            }

            flag_info = KVFlag::from_bits(flags).ok_or(KV3Error::KV3FlagsInvalid(flags))?;
        }

        let data_type = KVType::from_value(data).ok_or(KV3Error::KV3TypeUnknown(data))?;
        Ok((data_type, flag_info))
    }
}

fn read_kv3_v3p<R>(reader: &mut R, version: u32) -> Result<KV3Value, KV3Error>
where
    R: Read,
{
    let header = ResourceHeader::read_header(reader, version)?;
    let payload = decompress_kv3_v3p_playload(reader, &header)?;

    let mut parser = V3PParserState {
        version,
        payload: &payload,

        payload_reader: Cursor::new(&payload.payload),
        block_reader: Cursor::new(&payload.block_data),

        current_type_index: 0,
        current_block_index: 0,

        current_bytes_offset: 0,
        // The first integer contains the string table length
        current_integer_offset: 0x04,
        current_quad_offset: 0,

        depth: 0,
    };

    parser.parse_next_entry()
}

impl KV3Value {
    pub fn parse<R>(reader: &mut R) -> Result<KV3Value, KV3Error>
    where
        R: Read,
    {
        let magic = reader.read_u32::<LittleEndian>()?;
        match magic {
            0x03564B56 => return Err(KV3Error::UnsupportedVersion(1)),
            0x4B563301 => return Err(KV3Error::UnsupportedVersion(2)),
            0x4B563302 => return Err(KV3Error::UnsupportedVersion(3)),
            0x4B563303 => return Err(KV3Error::UnsupportedVersion(4)),
            0x4B563304 => read_kv3_v3p(reader, 0x05),
            _ => Err(KV3Error::InvalidMagic),
        }
    }
}
