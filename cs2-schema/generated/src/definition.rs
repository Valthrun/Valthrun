use std::io::{
    self,
    Error,
    Result,
    Write,
};

use serde::{
    Deserialize,
    Serialize,
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
    pub fn emit_rust_definition(&self, output: &mut dyn std::io::Write) -> Result<()> {
        let mod_name = mod_name_from_schema_name(&self.schema_name);
        writeln!(output, "")?;
        writeln!(output, "/* {} ({}) */", mod_name, self.schema_name)?;
        writeln!(output, "pub mod {} {{", mod_name)?;

        writeln!(output, "  use super::*;")?;
        writeln!(output, "  use crate::*;")?;
        writeln!(output, "  use cs2_schema_cutl::*;")?;
        writeln!(output, "  use cs2_schema_declaration::*;")?;

        self.enums
            .iter()
            .try_for_each(|definition| definition.emit(output))?;

        self.classes
            .iter()
            .try_for_each(|definition| definition.emit(&self.schema_name, output))?;

        writeln!(output, "}}")?;
        writeln!(output, "/* {} */", mod_name)?;
        Ok(())
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct EnumDefinition {
    #[serde(default)]
    pub schema_scope_name: String,

    /// Enums public name
    pub enum_name: String,

    /// Byte size of the enum
    pub enum_size: usize,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub metadata: Vec<Metadata>,

    pub memebers: Vec<EnumMember>,
}

impl EnumDefinition {
    fn emit(&self, output: &mut dyn Write) -> Result<()> {
        let enum_name = self.enum_name.replace(":", "_");

        let (enum_type, value_mask) = match self.enum_size {
            1 => ("u8", 0xFF),
            2 => ("u16", 0xFFFF),
            4 => ("u32", 0xFFFFFFFF),
            8 => ("u64", 0xFFFFFFFFFFFFFFFF),
            _ => {
                return Err(Error::new(
                    io::ErrorKind::Other,
                    format!("invalid enum size {}", self.enum_size),
                ))
            }
        };

        writeln!(output, "  /* enum {} ({}) */", enum_name, self.enum_name)?;
        writeln!(output, "  define_schema! {{")?;
        writeln!(output, "    pub enum {} : {} {{", enum_name, enum_type)?;

        self.memebers
            .iter()
            .try_for_each(|offset| offset.emit(output, value_mask))?;

        writeln!(output, "    }}")?;
        writeln!(output, "  }}")?;
        writeln!(output, "  /* {} */", self.enum_name)?;
        writeln!(output)?;

        Ok(())
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct EnumMember {
    pub name: String,
    pub value: u64,
}

impl EnumMember {
    fn emit(&self, output: &mut dyn Write, value_mask: u64) -> Result<()> {
        writeln!(
            output,
            "      {} = 0x{:X},",
            self.name,
            self.value & value_mask
        )?;
        Ok(())
    }
}

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
    fn emit(&self, mod_name: &str, output: &mut dyn std::io::Write) -> Result<()> {
        let class_name = self.class_name.replace(":", "_");

        writeln!(output, "  /* class {} ({}) */", class_name, self.class_name)?;
        writeln!(output, "  define_schema! {{")?;
        if let Some(base_class) = &self.inherits {
            writeln!(
                output,
                "    pub struct {}[0x{:X}] : {} {{",
                class_name, self.class_size, base_class
            )?;
        } else {
            writeln!(
                output,
                "    pub struct {}[0x{:X}] {{",
                class_name, self.class_size
            )?;
        }

        writeln!(output, "      pub vtable: Ptr<()> = 0x00,")?; // Every schema class has a vtable
        self.offsets
            .iter()
            .try_for_each(|offset| offset.emit(mod_name, &self.class_name, output))?;

        writeln!(output, "    }}")?;
        writeln!(output, "  }}")?;
        writeln!(output, "  /* {} */", self.class_name)?;
        writeln!(output)?;

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
        output: &mut dyn std::io::Write,
    ) -> Result<()> {
        if let Some(field_type) = &self.field_type {
            writeln!(
                output,
                "      /// Var: {} {}  ",
                self.field_ctype, self.field_name
            )?;
            writeln!(output, "      /// Offset: 0x{:X}  ", self.offset)?;
            writeln!(
                output,
                "      pub {}: {} = RuntimeOffset::new(\"{}\", \"{}\", \"{}\"),",
                self.field_name, field_type, mod_name, class_name, self.field_name
            )?;
        } else {
            writeln!(
                output,
                "      // Var: {} {}  ",
                self.field_ctype, self.field_name
            )?;
            writeln!(output, "      // Offset: 0x{:X}  ", self.offset)?;
            writeln!(
                output,
                "      /* pub {}: {} = 0x{:X}, */",
                self.field_name, self.field_ctype, self.offset
            )?;
        }

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
