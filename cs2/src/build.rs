use obfstr::obfstr;

use crate::{
    CS2Handle,
    EngineBuildInfo,
    Module,
    Signature,
};

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
            revision: engine_build_info.revision()?.read_string()?,
            build_datetime: format!(
                "{} {}",
                engine_build_info.build_date()?.read_string()?,
                engine_build_info.build_time()?.read_string()?
            ),
        })
    }
}
