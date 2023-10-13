#![feature(cursor_remaining)]
#![feature(seek_stream_len)]
#![feature(iterator_try_collect)]

use std::{
    collections::BTreeMap,
    io::{
        Cursor,
        Read,
        Seek,
    },
};

use crc::{
    Crc,
    CRC_32_ISO_HDLC,
};

mod error;
pub use error::*;

mod vpk_header;
pub use vpk_header::*;

mod vpk_tree;
pub use vpk_tree::*;

mod resource;
pub use resource::*;

mod kv3;
pub use kv3::*;

mod util;

pub struct VPKArchiveReader<R> {
    _header: VPKHeaderV2,
    entries: BTreeMap<String, DirectoryEntry>,

    reader: R,
    data_offset: u64,
}

pub const VALVE_CRC32: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);
impl<R> VPKArchiveReader<R>
where
    R: Read + Seek,
{
    pub fn new(mut reader: R) -> VResult<Self> {
        let header = VPKHeader::parse_header(&mut reader)?;
        let header_v2 = match header {
            VPKHeader::V1(_) => return Err(VPKError::UnsupportedArchiveVersion { version: 1 }),
            VPKHeader::V2(header) => header,
        };

        let tree = {
            let mut tree_buffer = Vec::with_capacity(header_v2.tree_size);
            tree_buffer.resize(header_v2.tree_size, 0u8);
            reader.read_exact(&mut tree_buffer)?;

            let mut tree_cursor = Cursor::new(tree_buffer);
            let tree = parse_directory_tree(&mut tree_cursor)?;
            if !tree_cursor.is_empty() {
                return Err(VPKError::UnconsumedData {
                    step: "directory tree".to_string(),
                });
            }

            tree
        };

        let data_offset = reader.stream_position()?;
        Ok(Self {
            _header: header_v2,
            entries: tree,

            data_offset,
            reader,
        })
    }

    pub fn entries(&self) -> &BTreeMap<String, DirectoryEntry> {
        &self.entries
    }

    pub fn into_inner(self) -> R {
        self.reader
    }

    pub fn read_entry(&mut self, name: &str) -> VResult<Vec<u8>> {
        let entry = self.entries.get(name).ok_or(VPKError::EntryUnknown)?;

        if entry.archive_index != 0x7FFF {
            return Err(VPKError::EntryNotContainedInThisArchive);
        }

        let mut buffer = Vec::new();
        buffer.resize(entry.entry_length as usize, 0u8);
        if buffer.is_empty() {
            return Ok(buffer);
        }

        let position = self.data_offset + entry.entry_offset as u64;
        self.reader.seek(std::io::SeekFrom::Start(position))?;
        self.reader.read_exact(&mut buffer)?;

        let buffer_crc = VALVE_CRC32.checksum(&buffer);
        if buffer_crc != entry.crc {
            return Err(VPKError::EntryCrcMissmatch {
                expected: entry.crc,
                calculated: buffer_crc,
            });
        }
        Ok(buffer)
    }
}
