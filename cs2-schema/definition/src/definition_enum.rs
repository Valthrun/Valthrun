use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    EmitOutput,
    Metadata,
};

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
    pub fn emit(&self, output: &mut dyn EmitOutput) -> anyhow::Result<()> {
        let enum_name = self.enum_name.replace(":", "_");

        let (enum_type, value_mask) = match self.enum_size {
            1 => ("u8", 0xFF),
            2 => ("u16", 0xFFFF),
            4 => ("u32", 0xFFFFFFFF),
            8 => ("u64", 0xFFFFFFFFFFFFFFFF),
            _ => anyhow::bail!("invalid enum size {}", self.enum_size),
        };

        output.emit_line(&format!("/* enum {} ({}) */", enum_name, self.enum_name))?;
        output.emit_line(&format!("#[repr(transparent)]"))?;
        output.emit_line(&format!("#[derive(Copy, Clone, Debug)]"))?;
        output.emit_line(&format!("pub struct {enum_name}({enum_type});"))?;
        output.emit_line(&format!("impl {enum_name} {{"))?;
        output.push_ident();

        self.memebers
            .iter()
            .try_for_each(|offset| offset.emit(output, &enum_type, value_mask))?;

        output.pop_ident();
        output.emit_line(&format!("}}"))?;
        output.emit_line(&format!("/* {} */", self.enum_name))?;

        Ok(())
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct EnumMember {
    pub name: String,
    pub value: u64,
}

impl EnumMember {
    fn emit(
        &self,
        output: &mut dyn EmitOutput,
        rtype: &str,
        value_mask: u64,
    ) -> anyhow::Result<()> {
        let member_name = &self.name;
        let member_value = self.value & value_mask;

        output.emit_line(&format!(
            "const {member_name}: {rtype} = 0x{member_value:X};"
        ))?;
        Ok(())
    }
}
