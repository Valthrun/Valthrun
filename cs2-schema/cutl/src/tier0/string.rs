use raw_struct::{
    raw_struct,
    AccessError,
    Copy,
    MemoryView,
};

use super::CUtlMemory;

#[raw_struct(size = 0x14)]
pub struct CUtlString {
    #[field(offset = 0x00)]
    pub memory: Copy<dyn CUtlMemory<u8>>,

    #[field(offset = 0x10)]
    pub actual_length: u32,
}

impl dyn CUtlString {
    pub fn read_string(&self, memory: &dyn MemoryView) -> Result<String, AccessError> {
        let buffer = self
            .memory()?
            .buffer()?
            .elements(memory, 0..self.actual_length()? as usize)?;

        Ok(String::from_utf8_lossy(&buffer).to_string())
    }
}

#[raw_struct(size = 0x04)]
pub struct CUtlStringToken {
    #[field(offset = 0x00)]
    pub hash_code: u32,
    // only present if compiled with DEBUG_STRINGTOKENS
    /* pub m_pDebugName: PtrCStr = 0x08 */
}
