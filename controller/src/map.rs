use cs2::CS2Handle;
use cs2_schema_declaration::{
    define_schema,
    Ptr,
    PtrCStr,
};
use serde::Serialize;

#[derive(Debug, PartialEq, PartialOrd, Serialize)]
pub struct MapInfo {
    pub name: String,
}

define_schema! {
    pub struct CNetworkGameClient[0x290] {
        pub map_path: PtrCStr = 0x220,
        pub map_name: PtrCStr = 0x228,
    }
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
        Ok(Some(MapInfo { name }))
    } else {
        Ok(None)
    }
}
