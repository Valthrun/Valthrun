use std::collections::BTreeMap;

use cs2_schema_provider::{
    OffsetInfo,
    SchemaProvider,
};

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct CachedOffset {
    pub module: String,
    pub class: String,
    pub member: String,
}

impl From<OffsetInfo> for CachedOffset {
    fn from(value: OffsetInfo) -> Self {
        Self {
            module: value.module.to_string(),
            class: value.class_name.to_string(),
            member: value.member.to_string(),
        }
    }
}

pub struct CachedSchemaProvider {
    offsets: BTreeMap<CachedOffset, u64>,
}

impl CachedSchemaProvider {
    pub fn new(offsets: BTreeMap<CachedOffset, u64>) -> Self {
        Self { offsets }
    }
}

impl SchemaProvider for CachedSchemaProvider {
    fn resolve_offset(&self, offset: &OffsetInfo) -> Option<u64> {
        let offset = CachedOffset::from(offset.clone());
        self.offsets.get(&offset).cloned()
    }
}
