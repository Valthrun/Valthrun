use std::{any::Any, sync::Arc};

pub trait MemoryHandle : Any  {
    fn as_any(&self) -> &dyn Any;
    fn read_slice(&self, offset: u64, slice: &mut [u8]) -> anyhow::Result<()>;

    fn reference_memory(&self, address: u64, length: Option<usize>) -> anyhow::Result<Arc<dyn MemoryHandle>>;
    fn read_memory(&self, address: u64, length: usize) -> anyhow::Result<Arc<dyn MemoryHandle>>;
}