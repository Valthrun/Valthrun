use std::io::Read;

use byteorder::{
    LittleEndian,
    ReadBytesExt,
};

use crate::{
    VPKError,
    VResult,
};

#[derive(Debug)]
pub struct VPKHeaderV1 {
    pub tree_size: usize,
}

#[derive(Debug)]
pub struct VPKHeaderV2 {
    pub tree_size: usize,
    pub file_data_section_size: usize,
    pub archive_md5_section_size: usize,
    pub other_md5_section_size: usize,
    pub signature_section_size: usize,
}

const VPK_HEADER_SIGNATURE: u32 = 0x55aa1234;

#[derive(Debug)]
pub enum VPKHeader {
    V1(VPKHeaderV1),
    V2(VPKHeaderV2),
}

impl VPKHeader {
    pub(crate) fn parse_header<R>(reader: &mut R) -> VResult<Self>
    where
        R: Read,
    {
        let signature = reader.read_u32::<LittleEndian>()?;
        if signature != VPK_HEADER_SIGNATURE {
            log::debug!(
                "File signature is 0x{:X}, expected 0x{:X}",
                signature,
                VPK_HEADER_SIGNATURE
            );
            return Err(VPKError::InvalidFileSignature);
        }

        let version = reader.read_u32::<LittleEndian>()?;
        match version {
            1 => {
                let tree_size = reader.read_u32::<LittleEndian>()? as usize;
                Ok(VPKHeader::V1(VPKHeaderV1 { tree_size }))
            }
            2 => {
                let tree_size = reader.read_u32::<LittleEndian>()? as usize;
                let file_data_section_size = reader.read_u32::<LittleEndian>()? as usize;
                let archive_md5_section_size = reader.read_u32::<LittleEndian>()? as usize;
                let other_md5_section_size = reader.read_u32::<LittleEndian>()? as usize;
                let signature_section_size = reader.read_u32::<LittleEndian>()? as usize;

                Ok(VPKHeader::V2(VPKHeaderV2 {
                    tree_size,
                    file_data_section_size,
                    archive_md5_section_size,
                    other_md5_section_size,
                    signature_section_size,
                }))
            }
            version => Err(VPKError::UnsupportedArchiveVersion { version }),
        }
    }
}
