use std::{
    self,
    borrow::Cow,
};

use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    mod_name_from_schema_name,
    ClassReference,
    EmitOutput,
    InheritageMap,
    Metadata,
};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ClassDefinition {
    #[serde(default)]
    pub schema_scope_name: String,
    pub class_name: String,
    pub class_size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub inherits: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub metadata: Vec<Metadata>,
    pub offsets: Vec<ClassField>,
}

impl ClassDefinition {
    pub fn emit(
        &self,
        mod_name: &str,
        output: &mut dyn EmitOutput,
        inheritage: &InheritageMap,
    ) -> anyhow::Result<()> {
        let class_name = self.class_name.replace(":", "_");

        output.emit_line(&format!("/* class {} ({}) */", class_name, self.class_name))?;
        output.emit_line(&format!(
            "#[raw_struct::raw_struct(size = 0x{:X})]",
            self.class_size
        ))?;
        output.emit_line(&format!("pub struct {class_name} {{"))?;
        output.push_ident();

        output.emit_line(&format!("#[field(offset = 0x00)]"))?;
        output.emit_line(&format!("pub vtable: Ptr64<()>,"))?;

        self.offsets
            .iter()
            .try_for_each(|offset| offset.emit(mod_name, &self.class_name, output))?;

        output.pop_ident();
        output.emit_line(&format!("}}"))?;

        for class in inheritage.get_inherited_classes(&ClassReference {
            class_name: self.class_name.clone(),
            module_name: mod_name_from_schema_name(&mod_name).to_string(),
        }) {
            output.emit_line(&format!(
                "impl {}::{} for dyn {class_name} {{ }}",
                class.module_name, class.class_name
            ))?;
        }

        output.emit_line(&format!("/* {} */", self.class_name))?;

        Ok(())
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ClassField {
    pub field_name: String,

    /// Rust mapped field type.
    /// If none the type isn't yet supported.
    pub field_type: Option<String>,

    /// The engines field type
    pub field_ctype: String,

    pub offset: u64,
    pub metadata: Vec<Metadata>,
}

impl ClassField {
    fn emit(
        &self,
        mod_name: &str,
        class_name: &str,
        output: &mut dyn EmitOutput,
    ) -> anyhow::Result<()> {
        if let Some(field_type) = &self.field_type {
            output.emit_line(&format!(
                "/// Var: {} {}  ",
                self.field_ctype, self.field_name
            ))?;

            output.emit_line(&format!("/// Offset: 0x{:X}  ", self.offset))?;
            output.emit_line(&format!(
                "#[field(offset = r#\"cs2_schema_cutl::runtime_offset!({}, \"{}\", \"{}\", \"{}\")\"#)]",
                self.offset, mod_name, class_name, self.field_name
            ))?;
            output.emit_line(&format!(
                "pub {}: {},",
                self.field_name,
                if field_type.starts_with("dyn ") {
                    Cow::from(format!("Copy<{}>", field_type))
                } else {
                    Cow::from(field_type)
                }
            ))?;
        } else {
            output.emit_line(&format!(
                "// Var: {} {}  ",
                self.field_ctype, self.field_name
            ))?;
            output.emit_line(&format!("// Offset: 0x{:X}  ", self.offset))?;
            output.emit_line(&format!(
                "// pub {}: {} = 0x{:X},",
                self.field_name, self.field_ctype, self.offset
            ))?;
        }

        Ok(())
    }
}
