use std::{
    any::Any,
    sync::Arc,
};

use crate::SchemaValue;

pub trait MemoryDriver: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;

    fn read_slice(&self, address: u64, slice: &mut [u8]) -> anyhow::Result<()>;
    fn read_cstring(
        &self,
        address: u64,
        expected_length: Option<usize>,
        max_length: Option<usize>,
    ) -> anyhow::Result<String>;

    /* fn write_slice(&self, address: u64, slice: &[u8]) -> anyhow::Result<()>; */
}

pub struct MemoryCached {
    address: u64,
    buffer: Vec<u8>,
}

#[derive(Clone)]
pub struct MemoryHandle {
    pub driver: Arc<dyn MemoryDriver>,
    pub address: u64,

    cache: Option<Arc<MemoryCached>>,
}

impl MemoryHandle {
    pub fn from_driver(driver: &Arc<dyn MemoryDriver>, address: u64) -> Self {
        Self {
            driver: driver.clone(),
            address,

            cache: None,
        }
    }

    pub fn with_offset(self, offset: u64) -> anyhow::Result<Self> {
        Ok(Self {
            driver: self.driver,
            address: self.address + offset,
            cache: self.cache,
        })
    }

    pub fn cache(&mut self, length: usize) -> anyhow::Result<()> {
        if let Some(cache) = &self.cache {
            assert!(cache.address <= self.address);
            let cache_offset = (self.address - cache.address) as usize;
            if cache.buffer.len() >= length + cache_offset {
                /* cache does already contain the requested data */
                return Ok(());
            }
        }
        self.cache = None;

        let mut buffer = Vec::with_capacity(length);
        buffer.resize(length, 0);

        self.read_slice(0x00, &mut buffer)?;

        self.cache = Some(Arc::new(MemoryCached {
            address: self.address,
            buffer,
        }));
        Ok(())
    }

    pub fn read_slice(&self, offset: u64, slice: &mut [u8]) -> anyhow::Result<()> {
        if let Some(cache) = &self.cache {
            assert!(cache.address <= self.address);
            let cache_offset = (self.address - cache.address) as usize;
            if cache.buffer.len() < offset as usize + slice.len() + cache_offset {
                anyhow::bail!("invalid target memory address")
            }

            slice.copy_from_slice(
                &cache.buffer[(cache_offset + offset as usize)
                    ..(cache_offset + offset as usize + slice.len())],
            );
            Ok(())
        } else {
            self.driver.read_slice(self.address + offset, slice)
        }
    }

    pub fn reference_schema<T: SchemaValue>(&self, offset: u64) -> anyhow::Result<T> {
        T::from_memory(self.clone().with_offset(offset)?)
    }
}
