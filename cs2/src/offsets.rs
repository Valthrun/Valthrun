use anyhow::Context;
use obfstr::obfstr;
use utils_state::{
    State,
    StateCacheType,
    StateRegistry,
};

use crate::{
    CS2Handle,
    CS2HandleState,
    Module,
    Signature,
};

/// Offsets which needs to be scaned for on runtime.
/// Mostly global variables.
#[derive(Debug, Clone)]
pub struct CS2Offsets {
    /// Address of the client globals
    pub globals: u64,

    /// Address for the local player controller ptr
    pub local_controller: u64,

    /// Address for the global entity list ptr
    pub global_entity_list: u64,

    /// Address for the global world to screen view matrix
    pub view_matrix: u64,

    /// Offset of the CNetworkGameClient
    pub network_game_client_instance: u64,
}

impl State for CS2Offsets {
    type Parameter = ();

    fn create(states: &StateRegistry, _param: Self::Parameter) -> anyhow::Result<Self> {
        let cs2 = states.resolve::<CS2HandleState>(())?;
        let cs2 = &*cs2;

        Ok(Self {
            globals: Self::find_globals(cs2).with_context(|| obfstr!("cs2 globals").to_string())?,
            local_controller: Self::find_local_player_controller_ptr(cs2)
                .with_context(|| obfstr!("local player controller ptr").to_string())?,
            global_entity_list: Self::find_entity_list(cs2)
                .with_context(|| obfstr!("global entity list").to_string())?,
            view_matrix: Self::find_view_matrix(cs2)
                .with_context(|| obfstr!("view matrix").to_string())?,
            network_game_client_instance: Self::find_network_game_client_instance(cs2)
                .with_context(|| obfstr!("network game client instance").to_string())?,
        })
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Persistent
    }
}

impl CS2Offsets {
    fn find_globals(cs2: &CS2Handle) -> anyhow::Result<u64> {
        cs2.resolve_signature(
            Module::Client,
            &Signature::relative_address(
                obfstr!("client globals"),
                obfstr!("48 8B 05 ? ? ? ? 8B 48 04 FF C1"),
                0x03,
                0x07,
            ),
        )
    }

    fn find_local_player_controller_ptr(cs2: &CS2Handle) -> anyhow::Result<u64> {
        // 48 83 3D ? ? ? ? ? 0F 95 -> IsLocalPlayerControllerValid
        cs2.resolve_signature(
            Module::Client,
            &Signature::relative_address(
                obfstr!("local player controller ptr"),
                obfstr!("48 83 3D ? ? ? ? ? 0F 95"),
                0x03,
                0x08,
            ),
        )
    }

    fn find_entity_list(cs2: &CS2Handle) -> anyhow::Result<u64> {
        // 4C 8B 0D ? ? ? ? 48 89 5C 24 ? 8B -> Global entity list
        cs2.resolve_signature(
            Module::Client,
            &Signature::relative_address(
                obfstr!("global entity list"),
                obfstr!("4C 8B 0D ? ? ? ? 48 89 5C 24 ? 8B"),
                0x03,
                0x07,
            ),
        )
    }

    fn find_view_matrix(cs2: &CS2Handle) -> anyhow::Result<u64> {
        cs2.resolve_signature(
            Module::Client,
            &Signature::relative_address(
                obfstr!("world view matrix"),
                obfstr!("48 8D 0D ? ? ? ? 48 C1 E0 06"),
                0x03,
                0x07,
            ),
        )
    }

    fn find_network_game_client_instance(cs2: &CS2Handle) -> anyhow::Result<u64> {
        cs2.resolve_signature(
            Module::Engine,
            &Signature::relative_address(
                obfstr!("network game client instance"),
                obfstr!("48 83 3D ? ? ? ? ? 48 8B D9 8B 0D"),
                0x03,
                0x08,
            ),
        )
    }
}
