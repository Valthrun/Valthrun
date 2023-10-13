use std::path::PathBuf;

use anyhow::Context;
use cs2::CS2Handle;
use cs2_schema_cutl::CUtlVector;
use cs2_schema_declaration::{
    define_schema,
    MemoryHandle,
    Ptr,
    PtrCStr,
    SchemaValue,
};

define_schema! {
    pub struct CNetworkGameClient[0x290] {
        pub map_path: PtrCStr = 0x220,
        pub map_name: PtrCStr = 0x228,
    }

    #[allow(non_camel_case_types)]
    pub struct CFileSystem_Stdio[0xC0] {
        pub defined_directories: CUtlVector<Ptr<SymbolEntry>> = 0x98,
    }
}

// 0x00 -> flags or whatever
// 0x04 -> hash
// 0x08 -> ptr to string
pub struct SymbolEntry {
    memory: MemoryHandle,
}

impl SymbolEntry {
    pub fn value(&self) -> anyhow::Result<PtrCStr> {
        self.memory.reference_schema(0x08)
    }
}

impl SchemaValue for SymbolEntry {
    fn value_size() -> Option<u64> {
        Some(0x10)
    }

    fn from_memory(memory: MemoryHandle) -> anyhow::Result<Self> {
        Ok(Self { memory })
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct MapInfo {
    pub name: String,
    pub file: PathBuf,
}

pub fn get_current_map(
    cs2: &CS2Handle,
    network_game_client_instance: u64,
) -> anyhow::Result<Option<MapInfo>> {
    let network_game_client = cs2
        .read_schema::<Ptr<CNetworkGameClient>>(&[network_game_client_instance])?
        .try_read_schema()?;

    if let Some(instance) = network_game_client {
        let name = instance.map_name()?.read_string()?;
        let path = instance.map_path()?.read_string()?;
        Ok(Some(MapInfo {
            name,
            file: PathBuf::from(&path),
        }))
    } else {
        Ok(None)
    }
}

/*
 * Hacky way of getting the current game directory by inspecting all defined symbols.
 */
pub fn get_game_directory(cs2: &CS2Handle, file_system: u64) -> anyhow::Result<PathBuf> {
    let file_system = cs2
        .read_schema::<Ptr<CFileSystem_Stdio>>(&[file_system])?
        .try_read_schema()?
        .context("missing file system")?;

    let mut value_address = file_system
        .defined_directories()?
        .read_element(0)?
        .read_schema()?
        .value()?
        .address()?;

    let mut value: String;
    loop {
        value = cs2.read_string(&[value_address], None)?;
        //log::trace!("Found symbol value {}.", value);
        if value.contains("Counter-Strike Global Offensive\\game\\csgo") {
            break;
        }

        value_address += value.as_bytes().len() as u64 + 0x01;
    }

    log::trace!(
        "Found symbol value which is a path to the games directory: {}",
        value
    );
    let mut path = PathBuf::from(&value);
    loop {
        let directory_name = path.file_name().context("expected a file name")?;
        if directory_name.to_string_lossy() == "csgo" {
            break;
        }

        path = path
            .parent()
            .context("expected the path to have a parent")?
            .to_path_buf();
    }

    Ok(path)
}
