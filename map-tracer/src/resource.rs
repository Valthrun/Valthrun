use std::io::{
    Read,
    Seek,
    SeekFrom,
};

use byteorder::{
    LittleEndian,
    ReadBytesExt,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ResourceError {
    #[error("io error")]
    IOError(#[from] std::io::Error),

    #[error("Use ValvePak library to parse VPK files.\nSee https://github.com/ValveResourceFormat/ValvePak")]
    ResourceIsValvePack,

    #[error("header contains unknown version ({0})")]
    HeaderUnknownVersion(u16),

    #[error("block index unknown")]
    UnknownBlockIndex,
}

#[derive(Debug)]
pub struct ResourceBlock {
    pub block_type: String,
    pub block_length: u32,
    block_offset: u64,
}

pub struct Resource<R> {
    version: u16,
    blocks: Vec<ResourceBlock>,

    reader: R,
}

impl<R> Resource<R>
where
    R: Read + Seek,
{
    pub fn new(mut reader: R) -> Result<Self, ResourceError> {
        let file_size = reader.read_u32::<LittleEndian>()?;
        if file_size == 0x55AA1234 {
            return Err(ResourceError::ResourceIsValvePack);
        }

        let header_version = reader.read_u16::<LittleEndian>()?;
        if header_version != 12 {
            return Err(ResourceError::HeaderUnknownVersion(header_version));
        }

        let version = reader.read_u16::<LittleEndian>()?;
        let block_offset = reader.read_u32::<LittleEndian>()?;
        let block_count = reader.read_u32::<LittleEndian>()?;

        let mut blocks: Vec<ResourceBlock> = Default::default();
        reader.seek(SeekFrom::Current(block_offset as i64 - 0x08))?;

        for _ in 0..block_count {
            let block_type = {
                let mut type_buffer = [0u8; 4];
                reader.read_exact(&mut type_buffer)?;
                String::from_utf8_lossy(&type_buffer).to_string()
            };
            let block_base = reader.stream_position()?;
            let block_offset = reader.read_u32::<LittleEndian>()?;
            let block_length = reader.read_u32::<LittleEndian>()?;

            blocks.push(ResourceBlock {
                block_type,
                block_offset: block_base + block_offset as u64,
                block_length,
            });
        }

        Ok(Self {
            version,
            blocks,

            reader,
        })
    }

    pub fn version(&self) -> u16 {
        self.version
    }

    pub fn blocks(&self) -> &[ResourceBlock] {
        &self.blocks
    }

    pub fn read_block(&mut self, index: usize) -> Result<Vec<u8>, ResourceError> {
        let block = self
            .blocks
            .get(index)
            .ok_or(ResourceError::UnknownBlockIndex)?;
        self.reader.seek(SeekFrom::Start(block.block_offset))?;

        let mut buffer = Vec::new();
        buffer.resize(block.block_length as usize, 0u8);
        self.reader.read_exact(&mut buffer)?;

        Ok(buffer)
    }
}
