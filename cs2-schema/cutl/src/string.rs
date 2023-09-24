use cs2_schema_declaration::define_schema;

use super::CUtlMemory;

define_schema! {
    pub struct CUtlString[0x14] {
        pub memory: CUtlMemory<u8> = 0x00,
        pub actual_length: u32 = 0x10,
    }

    pub struct CUtlStringToken[0x04] {
        pub hash_code: u32 = 0x00,

        // only present if compiled with DEBUG_STRINGTOKENS
        /* pub m_pDebugName: PtrCStr = 0x08 */
    }
}

impl CUtlString {
    pub fn read_string(&self) -> anyhow::Result<String> {
        let buffer = self.memory()?.buffer()?;
        let buffer = buffer.read_entries(self.actual_length()? as usize)?;

        Ok(String::from_utf8(buffer)?)
    }
}
