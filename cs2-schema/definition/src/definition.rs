use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    ClassDefinition,
    EmitOutput,
    EnumDefinition,
    InheritageMap,
};
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct SchemaScope {
    pub schema_name: String,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub classes: Vec<ClassDefinition>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub enums: Vec<EnumDefinition>,
}

pub fn mod_name_from_schema_name(name: &str) -> &str {
    if name.ends_with(".dll") {
        &name[0..name.len() - 4]
    } else if name == "!GlobalTypes" {
        "globals"
    } else {
        name
    }
}

impl SchemaScope {
    pub fn emit_rust_definition(
        &self,
        output: &mut dyn EmitOutput,
        inheritage: &InheritageMap,
    ) -> anyhow::Result<()> {
        output.emit_line("use super::*;")?;
        output.emit_line("use crate::*;")?;
        output.emit_line("use cs2_schema_cutl::*;")?;
        output.emit_line("use raw_struct::builtins::*;")?;
        output.emit_line("use raw_struct::Copy;")?;
        output.emit_line("")?;

        self.enums
            .iter()
            .try_for_each(|definition| -> anyhow::Result<()> {
                definition.emit(output)?;
                output.emit_line("")?;
                Ok(())
            })?;

        self.classes
            .iter()
            .try_for_each(|definition| -> anyhow::Result<()> {
                definition.emit(&self.schema_name, output, inheritage)?;
                output.emit_line("")?;
                Ok(())
            })?;

        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Metadata {
    NetworkEnable,
    NetworkDisable,
    NetworkChangeCallback { name: String },
    NetworkVarNames { var_name: String, var_type: String },
    Unknown { name: String },
}
