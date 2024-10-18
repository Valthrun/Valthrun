use anyhow::anyhow;
use cs2_schema_cutl::CStringUtil;
use raw_struct::{
    Copy,
    FromMemoryView,
};
use utils_state::{
    State,
    StateCacheType,
    StateRegistry,
};

use crate::{
    schema::EngineBuildInfo,
    CS2Offset,
    StateCS2Memory,
    StateResolvedOffset,
};

#[derive(Debug)]
pub struct StateBuildInfo {
    pub revision: String,
    pub build_datetime: String,
}

impl State for StateBuildInfo {
    type Parameter = ();

    fn create(states: &StateRegistry, _params: Self::Parameter) -> anyhow::Result<Self> {
        let memory = states.resolve::<StateCS2Memory>(())?;
        let offset = states.resolve::<StateResolvedOffset>(CS2Offset::BuildInfo)?;

        let engine_build_info =
            Copy::<dyn EngineBuildInfo>::read_object(memory.view(), offset.address)
                .map_err(|e| anyhow!(e))?;

        Ok(Self {
            revision: engine_build_info
                .revision()?
                .read_string(memory.view())?
                .unwrap_or_default(),
            build_datetime: format!(
                "{} {}",
                engine_build_info
                    .build_date()?
                    .read_string(memory.view())?
                    .unwrap_or_default(),
                engine_build_info
                    .build_time()?
                    .read_string(memory.view())?
                    .unwrap_or_default()
            ),
        })
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Persistent
    }
}
