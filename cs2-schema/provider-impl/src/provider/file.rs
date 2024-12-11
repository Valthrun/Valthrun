use std::{
    collections::BTreeMap,
    fs::File,
    io::BufReader,
    path::Path,
};

use anyhow::Context;
use cs2_schema_definition::SchemaScope;
use cs2_schema_provider::{
    OffsetInfo,
    SchemaProvider,
};

use super::{
    CachedOffset,
    CachedSchemaProvider,
};

pub struct FileSchemaProvider {
    inner: CachedSchemaProvider,
}

impl FileSchemaProvider {
    pub fn load_from(file: &Path) -> anyhow::Result<Self> {
        let file = File::open(file).context("open file")?;
        let reader = BufReader::new(file);
        let scopes: Vec<SchemaScope> =
            serde_json::from_reader(reader).context("parse schema file")?;

        let mut offsets = BTreeMap::<CachedOffset, u64>::new();
        for scope in scopes {
            for class in scope.classes {
                for member in class.offsets {
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
