use cs2_schema_declaration::{define_schema, PtrCStr};
use obfstr::obfstr;

use crate::{CS2Handle, Module, PCStrEx, Signature};

define_schema! {
    pub struct EngineBuildInfo[0x28] {
        pub revision: PtrCStr = 0x00,
        pub build_date: PtrCStr = 0x08,
        pub build_time: PtrCStr = 0x10,
        /* pub unknown_zero: u64 */
        pub product_name: PtrCStr = 0x20,
    }
}

#[derive(Debug)]
pub struct BuildInfo {
    pub revision: String,
    pub build_datetime: String,
}

impl BuildInfo {
    fn find_build_info(cs2: &CS2Handle) -> anyhow::Result<u64> {
        cs2.resolve_signature(
            Module::Engine,
            &Signature::relative_address(
                obfstr!("client build info"),
                obfstr!("48 8B 1D ? ? ? ? 48 85 DB 74 6B"),
                0x03,
                0x07,
            ),
        )
    }

    pub fn read_build_info(cs2: &CS2Handle) -> anyhow::Result<Self> {
        let address = Self::find_build_info(cs2)?;
        let engine_build_info = cs2.read_schema::<EngineBuildInfo>(&[address])?;
        Ok(Self {
            revision: engine_build_info.revision()?.read_string(&cs2)?,
            build_datetime: format!(
                "{} {}",
                engine_build_info.build_date()?.read_string(&cs2)?,
                engine_build_info.build_time()?.read_string(&cs2)?
            ),
        })
    }
}
