mod definition;
use std::{
    collections::BTreeMap,
    fs,
    path::Path,
};

use anyhow::Context;
pub use definition::*;

mod definition_enum;
pub use definition_enum::*;

mod definition_class;
pub use definition_class::*;

mod inheritage;
pub use inheritage::*;

mod writer;
use serde::{
    Deserialize,
    Serialize,
};
pub use writer::*;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct DumpedSchema {
    pub cs2_revision: String,
    pub cs2_build_datetime: String,

    pub resolved_offsets: BTreeMap<String, u64>,
    pub scopes: Vec<SchemaScope>,
}

pub fn emit_to_dir(target: impl AsRef<Path>, scopes: &[SchemaScope]) -> anyhow::Result<()> {
    let target = target.as_ref();
    fs::create_dir_all(target).context("mkdirs")?;

    let inheritage = InheritageMap::build(scopes);
    for scope in scopes.iter() {
        let mut writer = FileEmitter::new(target.join(format!(
            "{}.rs",
            mod_name_from_schema_name(&scope.schema_name)
        )))?;

        scope.emit_rust_definition(&mut writer, &inheritage)?;
    }

    /* create the mod.rs */
    {
        let mut writer = FileEmitter::new(target.join("lib.rs"))?;
        for scope in scopes.iter() {
            let name = mod_name_from_schema_name(&scope.schema_name);
            writer.emit_line(&format!("pub mod {};", name))?;
        }
    }

    Ok(())
}
