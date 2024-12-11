use cs2_schema_provider::{
    OffsetInfo,
    SchemaProvider,
};

pub struct DefaultSchemaProvider;

impl SchemaProvider for DefaultSchemaProvider {
    fn resolve_offset(&self, offset: &OffsetInfo) -> Option<u64> {
        Some(offset.default_value)
    }
}
