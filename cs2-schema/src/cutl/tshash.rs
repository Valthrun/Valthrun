use std::{sync::Arc, marker::PhantomData};

use anyhow::Context;

use crate::{define_schema, MemoryHandle, SchemaValue, Ptr};


define_schema! {
    pub struct CUtlMemoryPool[0x18] {
        pub block_size: u32 = 0x00,
        pub blocks_per_blob: u32 = 0x04,

        pub grow_mode: u32 = 0x08,
        pub blocks_allocated: u32 = 0x0C,

        // Number of total ellements allocated
        pub block_allocated_size: u32 = 0x10,
        pub peak_alloc: u32 = 0x14,
    }
}

pub struct HashBucketData<K, V> {
    memory: Arc<dyn MemoryHandle>,
    offset: u64,

    _data: PhantomData<(K, V)>,
}

impl<K: SchemaValue, V: SchemaValue> HashBucketData<K, V> {
    pub fn value(&self) -> anyhow::Result<V> {
        SchemaValue::from_memory(&self.memory, self.offset + 0x00)
    }

    pub fn key(&self) -> anyhow::Result<K> {
        SchemaValue::from_memory(&self.memory, self.offset + V::value_size().context("value must have a size")? as u64 + 0x08)
    }
}

impl<K: SchemaValue, V: SchemaValue> SchemaValue for HashBucketData<K, V> {
    fn value_size() -> Option<usize> {
        Some(K::value_size()? + V::value_size()? + 0x08)
    }

    fn from_memory(memory: &Arc<dyn MemoryHandle>, offset: u64) -> anyhow::Result<Self> {
        Ok(Self {
            memory: memory.clone(),
            offset,

            _data: Default::default()
        })
    }
}

pub  struct HashUnallocatedData<K, V> {
    memory: Arc<dyn MemoryHandle>,
    offset: u64,

    _data: PhantomData<(K, V)>,
}


impl<K: SchemaValue, V: SchemaValue> HashUnallocatedData<K, V> {
    pub fn next_data(&self) -> anyhow::Result<Ptr<HashUnallocatedData<K, V>>> {
        SchemaValue::from_memory(&self.memory, self.offset + 0x00)
    }

    pub fn bucket_entry(&self, index: usize) -> anyhow::Result<HashBucketData<K, V>> {
        SchemaValue::from_memory(&self.memory, self.offset + 0x20 + (HashBucketData::<K, V>::value_size().context("hash bucket must have a size")? * index) as u64)
    }
}

impl<K: SchemaValue, V: SchemaValue> SchemaValue for HashUnallocatedData<K, V> {
    fn value_size() -> Option<usize> {
        // FIXME: HashunallocatedData length is determined by m_blocks_per_blob_!
        //        Pass as template parameter and not define this here.
        Some(0x20 + HashBucketData::<K, V>::value_size()? * 256)
    }

    fn from_memory(memory: &Arc<dyn MemoryHandle>, offset: u64) -> anyhow::Result<Self> {
        Ok(Self {
            memory: memory.clone(),
            offset,

            _data: Default::default()
        })
    }
}

pub  struct HashBucket<K, V> {
    memory: Arc<dyn MemoryHandle>,
    offset: u64,

    _data: PhantomData<(K, V)>,
}

impl<K: SchemaValue, V: SchemaValue> HashBucket<K, V> {
    pub fn unallocated_data(&self) -> anyhow::Result<Ptr<HashUnallocatedData<K, V>>> {
        SchemaValue::from_memory(&self.memory, self.offset + 0x18)
    }
}

impl<K, V> SchemaValue for HashBucket<K, V> {
    fn value_size() -> Option<usize> {
        Some(0x20)
    }

    fn from_memory(memory: &Arc<dyn MemoryHandle>, offset: u64) -> anyhow::Result<Self> {
        Ok(Self {
            memory: memory.clone(),
            offset: offset,
            _data: Default::default()
        })
    }
}

/// CUtlTSHash has the following layout:
/// struct CUtlTSHash<K, V, N> {
///     memory_pool: CUtlMemoryPool // 0x00
///     buckets: [HashBucket<K, V>, N] // 0x18
/// }
pub struct CUtlTSHash<K, V, const N: usize = 1> {
    memory: Arc<dyn MemoryHandle>,
    offset: u64,

    pub memory_pool: CUtlMemoryPool,
    _data: PhantomData<(K, V)>,
}

impl<K: SchemaValue, V: SchemaValue, const N: usize> CUtlTSHash<K, V, N> {
    pub fn bucket_count(&self) -> usize { N }

    pub fn bucket(&self, index: usize) -> anyhow::Result<HashBucket<K, V>> {
        let memory_bool_size = CUtlMemoryPool::value_size().context("memory pool must have a size")?;
        let bucket_size = HashBucket::<K, V>::value_size().context("hash bucket must have a size")?;
        SchemaValue::from_memory(&self.memory, self.offset + (memory_bool_size + index * bucket_size) as u64)
    }

    pub fn read_values(&self) -> anyhow::Result<Vec<V>> {
        let num_entries = self.memory_pool.block_allocated_size()? as usize;
        let mut num_entries_remaining = num_entries;

        let mut result = Vec::with_capacity(num_entries);
        let mut current_data = self.bucket(0)?.unallocated_data()?;
        while current_data.address()? > 0 && num_entries_remaining > 0 {
            let data_array = current_data.read_schema()?;
            let data_array_elements = (self.memory_pool.blocks_per_blob()? as usize).min(num_entries_remaining);
            for data_index in 0..data_array_elements {
                let value = data_array.bucket_entry(data_index)?.value()?;
                result.push(value);
            }
    
            num_entries_remaining -= data_array_elements;
            current_data = data_array.next_data()?;
        }
    
        if num_entries_remaining != 0 {
            anyhow::bail!("failed to read all elements")
        }
    
        Ok(result)
    }
}

impl<K: SchemaValue, V: SchemaValue, const N: usize> SchemaValue for CUtlTSHash<K, V, N> {
    fn value_size() -> Option<usize> {
        Some(CUtlMemoryPool::value_size()? + N * HashBucket::<K, V>::value_size()?)
    }

    fn from_memory(memory: &Arc<dyn MemoryHandle>, offset: u64) -> anyhow::Result<Self> {
        Ok(Self {
            memory_pool: SchemaValue::from_memory(memory, offset + 0x00)?,

            offset,
            memory: memory.clone(),

            _data: Default::default()
        })
    }
}