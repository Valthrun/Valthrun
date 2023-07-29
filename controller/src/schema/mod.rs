#![allow(dead_code)]
use std::{marker::PhantomData, fs::File, io::BufWriter};

use anyhow::Context;
use serde::{Serialize, Deserialize};
use valthrun_kinterface::ByteSequencePattern;

use crate::handle::{CS2Handle, Module};

// Returns SchemaSystem_001
fn get_schema_system(cs2: &CS2Handle) -> anyhow::Result<u64> {
    let load_address = cs2.find_pattern(
        Module::Schemasystem, 
        &ByteSequencePattern::parse("48 89 05 ? ? ? ? 4C 8D 45 D0").unwrap()
    )?.context("could not find schema system by signature")?;

    log::trace!("Schema sig resolved to {:X} ({:X})", cs2.module_info.schemasystem.base_address + load_address, load_address);
    Ok(
        load_address + cs2.read::<u32>(Module::Schemasystem, &[ load_address + 0x03 ])? as u64 + 0x07
    )
}

#[repr(C)]
#[derive(Debug, Default)]
struct CUtlMemoryPool {
    block_size: u32,
    blocks_per_blob: u32,

    grow_mode: u32,
    blocks_allocated: u32,
    
    block_allocated_size: u32,
    peak_alloc: u32,
}

impl CUtlMemoryPool {
    /// Number of total ellements allocated
    pub fn count(&self) -> usize {
        self.block_allocated_size as usize
    } 
}

// struct HashUnallocatedDataT {
//     HashUnallocatedDataT* m_next_ = nullptr; // 0x0000
//     Keytype m_6114; // 0x0008
//     Keytype m_ui_key; // 0x0010
//     Keytype m_i_unk_1; // 0x0018
//     std::array<HashBucketDataT, 256> m_current_block_list; // 0x0020
// }

#[repr(C)]
struct HashBucketT {
    struct_data: u64 /* HashStructDataT* */,
    mutex_list: u64 /* HashStructDataT* */,
    allocated_data: u64 /* HashAllocatedDataT* */,
    unallocated_data: u64 /* HashUnallocatedDataT* */,
}

#[repr(C)]
struct CUtlTSHash<const N: usize> {
    memory_pool: CUtlMemoryPool,
    buckets: [HashBucketT; N]
}

#[repr(C)]
pub struct Ptr<T> {
    pub value: u64,
    _data: PhantomData<T>
}

impl<T: Sized> Ptr<T> {
    pub fn try_read(&self, cs2: &CS2Handle) -> anyhow::Result<Option<T>> {
        if self.value == 0 {
            Ok(None)
        } else {
            Ok(Some(cs2.read::<T>(Module::Absolute, &[ self.value ])?))
        }
    }

    pub fn read(&self, cs2: &CS2Handle) -> anyhow::Result<T> {
        cs2.read::<T>(Module::Absolute, &[ self.value ])
    }
}

impl Ptr<*const i8> {
    pub fn read_string(&self, cs2: &CS2Handle) -> anyhow::Result<String> {
        cs2.read_string(Module::Absolute, &[ self.value ], None)
    } 
}

type PtrCStr = Ptr<*const i8>;


#[repr(C)]
struct CSchemaField {
    name: PtrCStr,

    field_type: u64,
    offset: u32,

    metadata_size: u32,
    metadata: u64
}

#[repr(C)]
struct CSchemaClassBinding {
    parent: Ptr<CSchemaClassBinding>,
    name: PtrCStr,
    module_name: PtrCStr,

    size: u32,
    field_size: u16,
    pad_0: u16,

    static_size: u16,
    metadata_size: u16,
    pad_1: u32,

    fields: u64, /* CSchemaField* */
}

fn cutl_tshash_elements<T: Sized>(cs2: &CS2Handle, address: u64) -> anyhow::Result<Vec<T>> {
    let scope_class_table = cs2.read::<CUtlTSHash::<1>>(Module::Absolute, &[ address ])?;

    let mut result = Vec::with_capacity(scope_class_table.memory_pool.count());
    
    let mut current_blob = scope_class_table.buckets[0].unallocated_data;
    let mut num_elem_remaining = scope_class_table.memory_pool.count();
    while current_blob > 0 && num_elem_remaining > 0 {
        let blob_element_count = (scope_class_table.memory_pool.blocks_per_blob as usize).min(num_elem_remaining);
        for block_index in 0..blob_element_count {
            let data = cs2.read::<T>(Module::Absolute, &[
                current_blob 
                    + 0x20 // blob header
                    + (scope_class_table.memory_pool.block_size as u64 * block_index as u64) // block index
                    + 0x00 // data index
            ])?;

            result.push(data);
        }

        num_elem_remaining -= blob_element_count;
        current_blob = cs2.read::<u64>(Module::Absolute, &[ current_blob ])?;
    }

    if num_elem_remaining != 0 {
        anyhow::bail!("failed to read all elements")
    }

    Ok(result)
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SchemaScope {
    schema_name: String,
    classes: Vec<ClassOffsets>
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ClassOffsets {
    class_name: String,
    offsets: Vec<Offset>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Offset {
    field_name: String,
    offset: u64
}

fn read_class_binding(cs2: &CS2Handle, address: u64) -> anyhow::Result<ClassOffsets> {
    let binding = cs2.read::<CSchemaClassBinding>(Module::Absolute, &[ address ])?;
    let mut class_offsets: ClassOffsets = Default::default();
    class_offsets.class_name = binding.name.read_string(cs2)?;
    class_offsets.offsets.reserve(binding.field_size as usize);

    //log::debug!(" - {:X} {} ({}; {})", schema_class, class_offsets.class_name, binding.field_size, binding.size);
    for field_index in 0..binding.field_size {
        let field = cs2.read::<CSchemaField>(Module::Absolute, &[ 
            binding.fields + (field_index * 0x20) as u64
        ])?;

        //log::debug!("    - {:X} {}", field.offset, field.name.read_string(cs2)?);
        class_offsets.offsets.push(
            Offset { field_name: field.name.read_string(cs2)?, offset: field.offset as u64 }
        );
    }

    Ok(class_offsets)
}

const OFFSET_SCHEMA_SYSTEM_SCOPE_SIZE: u64 = 0x190;
const OFFSET_SCHEMA_SYSTEM_SCOPE_ARRAY: u64 = 0x198;
const OFFSET_SCHEMA_SCOPE_CLASS_BINDINGS: u64 = 0x558;
const OFFSET_SCHEMA_SCOPE_ENUM_BINDINGS: u64 = 0x2DA0;
pub fn dump_schema(cs2: &CS2Handle) -> anyhow::Result<()> {
    log::info!("Dumping schema!");

    let schema_system = get_schema_system(cs2)?;
    let scope_size = cs2.read::<u64>(Module::Schemasystem, &[ schema_system + OFFSET_SCHEMA_SYSTEM_SCOPE_SIZE ])?;
    log::debug!("Schema system located at 0x{:X} (0x{:X}) containing 0x{:X} scopes", cs2.memory_address(Module::Schemasystem, schema_system)?, schema_system, scope_size);
    
    let mut schema_scops = Vec::<SchemaScope>::with_capacity(scope_size as usize);
    for scope_index in 0..scope_size {
        /* scope: CSchemaSystemTypeScope */
        let scope = cs2.read::<u64>(Module::Schemasystem, &[
            schema_system + OFFSET_SCHEMA_SYSTEM_SCOPE_ARRAY, // PTR to scope array
            scope_index * 8 // entry in array
        ])?;

        let mut scope_info: SchemaScope = Default::default();
        scope_info.schema_name = cs2.read_string(Module::Absolute, &[ scope + 0x08 ], Some(0x100))?;

        let class_bindings = cutl_tshash_elements::<u64>(cs2, scope + OFFSET_SCHEMA_SCOPE_CLASS_BINDINGS)?;
        scope_info.classes.reserve(class_bindings.len());
        // let enum_bindings = cutl_tshash_elements::<u64>(cs2, scope + OFFSET_SCHEMA_SCOPE_ENUM_BINDINGS)?;

        log::debug!(" {:X} -> {} ({})", scope, scope_info.schema_name, class_bindings.len());
        for schema_class in class_bindings {
            scope_info.classes.push(
                read_class_binding(cs2, schema_class)?
            );
        }

        schema_scops.push(scope_info);
    }    


    let output = File::options()
        .create(true)
        .truncate(true)
        .write(true)
        .open("cs2_schema.json")?;

    let mut output = BufWriter::new(output);
    serde_json::to_writer_pretty(&mut output, &schema_scops)?;
    log::info!("Schema dumped");
    Ok(())
}