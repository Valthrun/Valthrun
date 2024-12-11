use std::sync::RwLock;

#[derive(Debug, Clone, Copy)]
pub struct OffsetInfo {
    pub default_value: u64,
    pub module: &'static str,
    pub class_name: &'static str,
    pub member: &'static str,
}

pub trait SchemaProvider: Send + Sync {
    fn resolve_offset(&self, offset: &OffsetInfo) -> Option<u64>;
}

pub(crate) static PROVIDER_INSTANCE: RwLock<Option<Box<dyn SchemaProvider>>> = RwLock::new(None);

pub fn setup_provider(provider: Box<dyn SchemaProvider>) {
    let mut instance = PROVIDER_INSTANCE.write().unwrap();
    *instance = Some(provider);
}
