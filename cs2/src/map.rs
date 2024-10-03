use cs2_schema_declaration::{
    define_schema,
    Ptr,
    PtrCStr,
};
use utils_state::{
    State,
    StateCacheType,
    StateRegistry,
};

use crate::{
    CS2Handle,
    CS2HandleState,
    CS2Offsets,
};

define_schema! {
    pub struct CNetworkGameClient[0x290] {
        pub map_path: PtrCStr = 0x202,
        pub map_name: PtrCStr = 0x210,
    }
}

pub struct StateCurrentMap {
    pub current_map: Option<String>,
}

impl State for StateCurrentMap {
    type Parameter = ();

    fn create(states: &StateRegistry, _param: Self::Parameter) -> anyhow::Result<Self> {
        let cs2 = states.resolve::<CS2HandleState>(())?;
        let offsets = states.resolve::<CS2Offsets>(())?;

        let network_game_client = cs2
            .read_schema::<Ptr<CNetworkGameClient>>(&[offsets.network_game_client_instance])?
            .try_read_schema()?;

        let result = if let Some(instance) = network_game_client {
            if let Ok(map_name) = instance.map_name()?.read_string() {
                Self {
                    current_map: Some(map_name),
                }
            } else {
                /* Happens during connecting and disconnecting. */
                Self { current_map: None }
            }
        } else {
            Self { current_map: None }
        };
        Ok(result)
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Volatile
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
