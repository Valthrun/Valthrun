use std::{
    collections::BTreeMap,
    io::Read,
};

use byteorder::{
    LittleEndian,
    ReadBytesExt,
};

use crate::{
    util::read_cstring,
    VPKError,
    VResult,
};

#[derive(Debug)]
pub struct DirectoryEntry {
    pub crc: u32,
    pub preload_bytes: Option<Vec<u8>>,

    pub archive_index: u16,
    pub entry_offset: u32,
    pub entry_length: u32,
}

impl DirectoryEntry {
    pub(crate) fn read<R>(reader: &mut R) -> VResult<Self>
    where
        R: Read,
    {
        let crc = reader.read_u32::<LittleEndian>()?;
        let preload_bytes = reader.read_u16::<LittleEndian>()? as usize;
        let archive_index = reader.read_u16::<LittleEndian>()?;
        let entry_offset = reader.read_u32::<LittleEndian>()?;
        let entry_length = reader.read_u32::<LittleEndian>()?;
        let terminator = reader.read_u16::<LittleEndian>()?;
        if terminator != 0xFFFF {
            return Err(VPKError::InvalidDirectoryEntryTerminator(terminator));
        }

        let preload_bytes = if preload_bytes > 0 {
            let mut buffer = Vec::with_capacity(preload_bytes);
            buffer.resize(preload_bytes, 0u8);
            reader.read_exact(&mut buffer)?;
            Some(buffer)
        } else {
            None
        };

        Ok(Self {
            crc,

            preload_bytes,
            archive_index,

            entry_offset,
            entry_length,
        })
    }
}

pub fn parse_directory_tree<R>(reader: &mut R) -> VResult<BTreeMap<String, DirectoryEntry>>
where
    R: Read,
{
    let mut entries: BTreeMap<String, DirectoryEntry> = Default::default();
    loop {
        let file_extension = read_cstring::<_, VPKError>(reader)?;
        if file_extension.is_empty() {
            break;
        }

        loop {
            let directory = read_cstring::<_, VPKError>(reader)?;
            if directory.is_empty() {
                break;
            }

            loop {
                let file_name = read_cstring::<_, VPKError>(reader)?;
                if file_name.is_empty() {
                    break;
                }

                let entry_path = format!(
                    "{}/{}.{}",
                    directory.trim(),
                    file_name.trim(),
                    file_extension.trim()
                );
                let entry = DirectoryEntry::read(reader)?;
                entries.insert(entry_path, entry);
            }
        }
    }

    Ok(entries)
}
