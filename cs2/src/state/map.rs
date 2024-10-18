use anyhow::{
    anyhow,
    Context,
};
use cs2_schema_cutl::CStringUtil;
use raw_struct::{
    builtins::Ptr64,
    FromMemoryView,
};
use utils_state::{
    State,
    StateCacheType,
    StateRegistry,
};

use crate::{
    schema::CNetworkGameClient,
    CS2Offset,
    StateCS2Memory,
    StateResolvedOffset,
};

pub struct StateCurrentMap {
    pub current_map: Option<String>,
}

impl State for StateCurrentMap {
    type Parameter = ();

    fn create(states: &StateRegistry, _param: Self::Parameter) -> anyhow::Result<Self> {
        let memory_view = states.resolve::<StateCS2Memory>(())?;
        let offset_network_game_client_instance =
            states.resolve::<StateResolvedOffset>(CS2Offset::NetworkGameClientInstance)?;

        let instance = Ptr64::<dyn CNetworkGameClient>::read_object(
            memory_view.view(),
            offset_network_game_client_instance.address,
        )
        .map_err(|e| anyhow!(e))?
        .value_reference(memory_view.view_arc())
        .context("network game client nullptr")?;

        Ok(Self {
            current_map: instance
                .map_name()
                .ok()
                .map(|v| v.read_string(memory_view.view()).ok().flatten())
                .flatten(),
        })
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Volatile
    }
}
