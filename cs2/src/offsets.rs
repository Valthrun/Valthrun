use anyhow::Context;
use obfstr::obfstr;

use crate::{
    CS2Handle,
    Module,
    Signature,
};

/// Offsets which needs to be scaned for on runtime.
/// Mostly global variables.
pub struct CS2Offsets {
    /// Address of the client globals
    pub globals: u64,

    /// Address for the local player controller ptr
    pub local_controller: u64,

    /// Address for the global entity list ptr
    pub global_entity_list: u64,

    /// Address for the global world to screen view matrix
    pub view_matrix: u64,

    /// Offset for the crosshair entity id in C_CSPlayerPawn
    pub offset_crosshair_id: u64,
}

impl CS2Offsets {
    pub fn resolve_offsets(cs2: &CS2Handle) -> anyhow::Result<Self> {
        Ok(Self {
            globals: Self::find_globals(cs2).with_context(|| obfstr!("cs2 globals").to_string())?,
            local_controller: Self::find_local_player_controller_ptr(cs2)
                .with_context(|| obfstr!("local player controller ptr").to_string())?,
            global_entity_list: Self::find_entity_list(cs2)
                .with_context(|| obfstr!("global entity list").to_string())?,
            view_matrix: Self::find_view_matrix(cs2)
                .with_context(|| obfstr!("view matrix").to_string())?,
            offset_crosshair_id: Self::find_offset_crosshair_id(cs2)
                .with_context(|| obfstr!("crosshair id").to_string())?,
        })
    }

    fn find_globals(cs2: &CS2Handle) -> anyhow::Result<u64> {
        cs2.resolve_signature(
            Module::Client,
            &Signature::relative_address(
                obfstr!("client globals"),
                obfstr!("48 89 15 ?? ?? ?? ?? 48 8D 05 ?? ?? ?? ?? 48 85 D2"),
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

    fn find_offset_crosshair_id(cs2: &CS2Handle) -> anyhow::Result<u64> {
        cs2.resolve_signature(
            Module::Client,
            &Signature::offset(
                obfstr!("C_CSPlayerPawn crosshair id"),
                obfstr!("41 89 86 ? ? ? ? 41 89 86"),
                0x03,
            ),
        )
    }
}
