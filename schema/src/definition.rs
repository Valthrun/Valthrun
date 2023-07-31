use std::io::Result;

use serde::{ Deserialize, Serialize };

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct SchemaScope {
    pub schema_name: String,
    pub classes: Vec<ClassOffsets>,
}

impl SchemaScope {
    pub fn emit_rust_definition(&self, output: &mut dyn std::io::Write) -> Result<()> {
        let mod_name = self.schema_name.replace(".dll", "");
        writeln!(output, "")?;
        writeln!(output, "/* {} ({}) */", mod_name, self.schema_name)?;
        writeln!(output, "pub mod {} {{", mod_name)?;

        self.classes.iter()
            .try_for_each(|classes| classes.emit(output))?;

        writeln!(output, "}}")?;
        writeln!(output, "/* {} */", mod_name)?;
        Ok(())
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ClassOffsets {
    pub class_name: String,
    pub offsets: Vec<Offset>,
}

impl ClassOffsets {
    fn emit(&self, output: &mut dyn std::io::Write) -> Result<()> {
        let class_name = self.class_name.replace(":", "_");
        writeln!(
            output,
            "  /* class {} ({}) */",
            class_name, self.class_name
        )?;
        writeln!(output, "  pub mod {} {{", class_name)?;

        self.offsets.iter()
            .try_for_each(|offset| offset.emit(output))?;

        writeln!(output, "  }}")?;
        writeln!(output, "  /* {} */", self.class_name)?;
        Ok(())
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Offset {
    pub field_name: String,
    pub offset: u64,
}

impl Offset {
    fn emit(&self, output: &mut dyn std::io::Write) -> Result<()> {
        writeln!(
            output,
            "    pub const {}: u64 = 0x{:X};",
            self.field_name, self.offset
        )?;

        Ok(())
    }
}