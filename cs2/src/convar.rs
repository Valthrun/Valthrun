use std::ops::Deref;

use cs2_schema_cutl::CStringUtil;
use raw_struct::Reference;
use utils_state::StateRegistry;

use crate::{
    schema::{
        CCVar,
        ConVar,
    },
    CS2Offset,
    StateCS2Memory,
    StateResolvedOffset,
};

pub struct ConVars {
    ccvars: Reference<dyn CCVar>,
}

impl ConVars {
    pub fn new(states: &StateRegistry) -> anyhow::Result<Self> {
        let memory = states.resolve::<StateCS2Memory>(())?;
        let ccvar_address = states.resolve::<StateResolvedOffset>(CS2Offset::CCVars)?;
        let ccvar_instance = Reference::<dyn CCVar>::new(memory.view_arc(), ccvar_address.address);
        Ok(Self {
            ccvars: ccvar_instance,
        })
    }

    pub fn find_cvar(&self, name: &str) -> anyhow::Result<Option<Reference<dyn ConVar>>> {
        let memory_view_arc = self.ccvars.reference_memory();
        let memory_view = memory_view_arc.deref();

        let entry_count = self.ccvars.entries_count()? as usize;

        let entries = self
            .ccvars
            .entries()?
            .elements_copy(memory_view, 0..entry_count)?;

        for entry in entries {
            let Some(con_var) = entry.value()?.value_reference(memory_view_arc.clone()) else {
                continue;
            };

            let Ok(Some(con_var_name)) = con_var.name()?.read_string(memory_view) else {
                continue;
            };

            if con_var_name != name {
                continue;
            }

            return Ok(Some(con_var));
        }

        Ok(None)
    }
}
