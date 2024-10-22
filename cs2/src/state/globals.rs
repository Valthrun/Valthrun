use std::ops::Deref;

use anyhow::{
    anyhow,
    Context,
};
use raw_struct::{
    builtins::Ptr64,
    Copy,
    FromMemoryView,
};
use utils_state::{
    State,
    StateCacheType,
    StateRegistry,
};

use crate::{
    schema::Globals,
    CS2Offset,
    StateCS2Memory,
    StateResolvedOffset,
};

pub struct StateGlobals(Copy<dyn Globals>);
impl State for StateGlobals {
    type Parameter = ();

    fn create(states: &StateRegistry, _param: Self::Parameter) -> anyhow::Result<Self> {
        let memory_view = states.resolve::<StateCS2Memory>(())?;
        let offset_globals = states.resolve::<StateResolvedOffset>(CS2Offset::Globals)?;

        let globals = Ptr64::<dyn Globals>::read_object(memory_view.view(), offset_globals.address)
            .map_err(|e| anyhow!(e))?;

        let globals = globals
            .value_copy(memory_view.view())?
            .context("CS2 globals nullptr")?;
        Ok(Self(globals))
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Volatile
    }
}

impl Deref for StateGlobals {
    type Target = dyn Globals;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}
