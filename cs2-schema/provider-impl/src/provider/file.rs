use std::{
    collections::BTreeMap,
    fs::File,
    io::BufReader,
    path::Path,
};

use anyhow::Context;
use cs2::{
    CS2Offset,
    StatePredefinedOffset,
};
use cs2_schema_definition::{
    DumpedSchema,
    SchemaScope,
};
use cs2_schema_provider::{
    OffsetInfo,
    SchemaProvider,
};
use utils_state::StateRegistry;

use super::{
    CachedOffset,
    CachedSchemaProvider,
};

pub struct FileSchemaProvider {
    inner: CachedSchemaProvider,
}

impl FileSchemaProvider {
    pub fn new(scopes: &[SchemaScope]) -> anyhow::Result<Self> {
        let mut offsets = BTreeMap::<CachedOffset, u64>::new();
        for scope in scopes {
            for class in &scope.classes {
                for member in &class.offsets {
                    offsets.insert(
                        CachedOffset {
                            module: class.schema_scope_name.to_string(),
                            class: class.class_name.to_string(),
                            member: member.field_name.to_string(),
                        },
                        member.offset,
                    );
                }
            }
        }
        Ok(Self {
            inner: CachedSchemaProvider::new(offsets),
        })
    }
}

impl SchemaProvider for FileSchemaProvider {
    fn resolve_offset(&self, offset: &OffsetInfo) -> Option<u64> {
        self.inner.resolve_offset(offset)
    }
}

pub fn setup_schema_from_file(states: &mut StateRegistry, file: &Path) -> anyhow::Result<()> {
    let file = File::open(file).context("open file")?;
    let reader = BufReader::new(file);
    let schema = serde_json::from_reader::<_, DumpedSchema>(reader).context("parse schema file")?;

    {
        let provider = FileSchemaProvider::new(&schema.scopes)?;
        cs2_schema_provider::setup_provider(Box::new(provider));
    }

    for offset in CS2Offset::available_offsets() {
        if let Some(value) = schema.resolved_offsets.get(offset.cache_name()).cloned() {
            let predefined_offset = StatePredefinedOffset::new(states, *offset, value)
                .with_context(|| format!("resolving predefined offset {}", offset.cache_name()))?;

            log::debug!(
                "Registering predefined offset {} (offset: {:X}, current address: {:X})",
                offset.cache_name(),
                value,
                predefined_offset.resolved
            );
            let _ = states.set(predefined_offset, *offset);
        }
    }

    Ok(())
}
