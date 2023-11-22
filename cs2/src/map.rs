use cs2_schema_declaration::{
    define_schema,
    Ptr,
    PtrCStr,
};

use crate::CS2Handle;

define_schema! {
    pub struct CNetworkGameClient[0x290] {
        pub map_path: PtrCStr = 0x220,
        pub map_name: PtrCStr = 0x228,
    }
}

pub fn get_current_map(
    cs2: &CS2Handle,
    network_game_client_instance: u64,
) -> anyhow::Result<Option<String>> {
    let network_game_client = cs2
        .read_schema::<Ptr<CNetworkGameClient>>(&[network_game_client_instance])?
        .try_read_schema()?;

    if let Some(instance) = network_game_client {
        let name = if let Ok(map_name) = instance.map_name()?.read_string() {
            map_name
        } else {
            /* Happens during connecting and disconnecting. */
            return Ok(None);
        };

        Ok(Some(name))
    } else {
        Ok(None)
    }
}
