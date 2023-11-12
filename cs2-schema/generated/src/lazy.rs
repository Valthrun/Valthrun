use std::cell::SyncUnsafeCell;

use anyhow::Context;
use cs2_schema_declaration::LazyOffset;

#[derive(Debug, Clone, Copy)]
pub struct RuntimeOffset {
    pub module: &'static str,
    pub class: &'static str,
    pub member: &'static str,
}

impl RuntimeOffset {
    pub const fn new(module: &'static str, class: &'static str, member: &'static str) -> Self {
        Self {
            module,
            class,
            member,
        }
    }
}

pub trait RuntimeOffsetProvider: Sync {
    fn resolve(&self, offset: &RuntimeOffset) -> anyhow::Result<u64>;
}

static OFFSET_PROVIDER: SyncUnsafeCell<Option<Box<dyn RuntimeOffsetProvider>>> =
    SyncUnsafeCell::new(None);
pub fn setup_runtime_offset_provider(provider: Box<dyn RuntimeOffsetProvider>) {
    let container = unsafe { &mut *OFFSET_PROVIDER.get() };
    *container = Some(provider);
}

impl LazyOffset for RuntimeOffset {
    fn offset(self) -> anyhow::Result<u64> {
        let provider = unsafe { &*OFFSET_PROVIDER.get() }
            .as_ref()
            .context("missing runtime offset provider")?;

        provider.resolve(&self)
    }
}
