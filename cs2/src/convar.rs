use std::sync::Arc;

use cs2_schema_declaration::{
    define_schema,
    Ptr,
    PtrCStr,
};
use obfstr::obfstr;

use crate::{
    CS2Handle,
    Module,
    Signature,
};

pub struct ConVars {
    ccvars: CCVar,
}

define_schema! {
    pub struct ConVar[0x48] {
        pub name: PtrCStr = 0x00,
        pub description: PtrCStr = 0x20,

        pub n_change_count: u32 = 0x2C,

        pub n_value: u32 = 0x40,
        pub n_value_min: u32 = 0x48,
        pub n_value_default: u32 = 0x50,

        pub fl_value: f32 = 0x40,
        pub fl_value_min: f32 = 0x48,
        pub fl_value_default: f32 = 0x50,
    }

    pub struct CCVarEntry[0x10] {
        pub con_var: Ptr<ConVar> = 0x00,
    }

    pub struct CCVar[0xFF] {
        pub entries: Ptr<[CCVarEntry]> = 0x40,
        pub entries_capacity: u64 = 0x48,
        pub entries_count: u16 = 0x52,
    }
}

impl ConVars {
    pub fn new(handle: Arc<CS2Handle>) -> anyhow::Result<Self> {
        let ccvar_instance = handle.resolve_signature(
            Module::Tier0,
            &Signature::relative_address(
                obfstr!("CCVars"),
                obfstr!("4C 8D 3D ? ? ? ? 0F 28"),
                0x03,
                0x07,
            ),
        )?;

        let ccvars = handle.reference_schema::<CCVar>(&[ccvar_instance])?;
        let result = Self { ccvars };

        Ok(result)
    }

    pub fn find_cvar(&self, name: &str) -> anyhow::Result<Option<ConVar>> {
        let entry_count = self.ccvars.entries_count()? as usize;

        let entries = self.ccvars.entries()?.read_entries(entry_count)?;
        for entry in entries {
            let con_var = entry.con_var()?.read_schema()?;
            let con_var_name = match con_var.name()?.read_string() {
                Ok(name) => name,
                Err(_) => continue,
            };

            if con_var_name != name {
                continue;
            }

            return Ok(Some(con_var));
        }

        Ok(None)
    }
}
