use anyhow::Context;
use obfstr::obfstr;
use utils_state::{
    State,
    StateCacheType,
    StateRegistry,
};

use crate::{
    Module,
    Signature,
    StateCS2Handle,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CS2Offset {
    Globals,
    BuildInfo,

    LocalController,
    GlobalEntityList,

    ViewMatrix,
    NetworkGameClientInstance,

    CCVars,
    SchemaSystem,
}

impl CS2Offset {
    pub fn signature(&self) -> (Module, Signature) {
        match *self {
            Self::Globals => (
                Module::Client,
                Signature::relative_address(
                    obfstr!("client globals"),
                    obfstr!("48 8B 05 ? ? ? ? 8B 48 04 FF C1"),
                    0x03,
                    0x07,
                ),
            ),
            Self::BuildInfo => (
                Module::Engine,
                Signature::relative_address(
                    obfstr!("client build info"),
                    obfstr!("48 8B 1D ? ? ? ? 48 85 DB 74 6B"),
                    0x03,
                    0x07,
                ),
            ),
            Self::LocalController => (
                Module::Client,
                Signature::relative_address(
                    obfstr!("local player controller ptr"),
                    obfstr!("48 83 3D ? ? ? ? ? 0F 95"),
                    0x03,
                    0x08,
                ),
            ),
            Self::GlobalEntityList => (
                Module::Client,
                Signature::relative_address(
                    obfstr!("global entity list"),
                    obfstr!("4C 8B 0D ? ? ? ? 48 89 5C 24 ? 8B"),
                    0x03,
                    0x07,
                ),
            ),
            Self::ViewMatrix => (
                Module::Client,
                Signature::relative_address(
                    obfstr!("world view matrix"),
                    obfstr!("48 8D 0D ? ? ? ? 48 C1 E0 06"),
                    0x03,
                    0x07,
                ),
            ),
            Self::NetworkGameClientInstance => (
                Module::Engine,
                Signature::relative_address(
                    obfstr!("network game client instance"),
                    obfstr!("48 83 3D ? ? ? ? ? 48 8B D9 8B 0D"),
                    0x03,
                    0x08,
                ),
            ),
            Self::CCVars => (
                Module::Tier0,
                Signature::relative_address(
                    obfstr!("CCVars"),
                    obfstr!("4C 8D 3D ? ? ? ? 0F 28"),
                    0x03,
                    0x07,
                ),
            ),
            Self::SchemaSystem => (
                Module::Schemasystem,
                Signature::relative_address(
                    obfstr!("schema system instance"),
                    obfstr!("48 8B 0D ? ? ? ? 48 8B 55 A0"),
                    0x03,
                    0x07,
                ),
            ),
        }
    }
}

pub struct StateResolvedOffset {
    pub address: u64,
}

impl State for StateResolvedOffset {
    type Parameter = CS2Offset;

    fn create(states: &StateRegistry, offset: Self::Parameter) -> anyhow::Result<Self> {
        let cs2 = states.resolve::<StateCS2Handle>(())?;
        let (module, signature) = offset.signature();

        let address = cs2
            .resolve_signature(module, &signature)
            .with_context(|| format!("offset {:?}", offset))?;
        Ok(Self { address })
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Persistent
    }
}
