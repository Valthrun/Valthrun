use std::{sync::Arc, ffi::CStr};

use anyhow::Context;
use cs2_schema::{definition::{ClassOffsets, Offset, SchemaScope}, define_schema};
use kinterface::ByteSequencePattern;
use cs2_schema::{ MemoryHandle, SchemaValue };
use crate::{CS2Handle, Module, PtrCStr, Ptr, CUtlTSHash, CUtlVector, FixedCString,};

// Returns SchemaSystem_001
fn find_schema_system(cs2: &CS2Handle) -> anyhow::Result<u64> {
    let load_address = cs2
        .find_pattern(
            Module::Schemasystem,
            &ByteSequencePattern::parse("48 89 05 ? ? ? ? 4C 8D 45").unwrap(),
        )?
        .context("could not find schema system by signature")?;

    log::trace!(
        "Schema sig resolved to {:X} ({:X})",
        cs2.module_info.schemasystem.base_address as u64 + load_address,
        load_address
    );
    Ok(load_address + cs2.read::<u32>(Module::Schemasystem, &[load_address + 0x03])? as u64 + 0x07)
}

define_schema! {
    pub struct CSchemaSystem[0x200] {
        pub scopes: CUtlVector<Ptr<CSchemaSystemTypeScope>> = 0x190,
    }

    pub struct CSchemaSystemTypeScope[0x2F00] {
        pub scope_name: FixedCString<0x100> = 0x08,
        pub class_bindings: CUtlTSHash<u64, Ptr<CSchemaClassBinding>> = 0x558,
        pub enum_bindings: CUtlTSHash<u64, Ptr<()>> = 0x2DA0,
    }

    pub struct CSchemaField[0x20] {
        pub name: PtrCStr = 0x00,

        pub field_type: u64 = 0x08,
        pub offset: u32 = 0x10,

        pub metadata_size: u32 = 0x14,
        pub metadata: u64 = 0x18,
    }

    pub struct CSchemaClassBinding[0x100] {
        pub parent: Ptr<u64> = 0x00,
        pub name: PtrCStr = 0x08,
        pub module_name: PtrCStr = 0x10,
        pub size: u32 = 0x18,
        pub field_size: u16 = 0x1C,

        pub static_size: u16 = 0x20,
        pub metadata_size: u16 = 0x22,

        pub fields: u64 = 0x28,

        // TODO: The struct itself is longer and contains more fields!
    }
}

fn read_class_binding(cs2: &CS2Handle, binding: &Ptr<CSchemaClassBinding>) -> anyhow::Result<ClassOffsets> {
    let binding = binding.read_schema(cs2)?;

    let mut class_offsets: ClassOffsets = Default::default();
    class_offsets.class_name = binding.name()?.read_string(cs2)?;
    class_offsets.offsets.reserve(binding.field_size()? as usize);

    //log::debug!(" - {:X} {} ({}; {})", schema_class, class_offsets.class_name, binding.field_size, binding.size);
    for field_index in 0..binding.field_size()? as usize {
        let field = cs2.read_schema::<CSchemaField>(&[
            binding.fields()? + (field_index * CSchemaField::value_size()) as u64
        ])?;

        //log::debug!("    - {:X} {}", field.offset, field.name.read_string(cs2)?);
        class_offsets.offsets.push(Offset {
            field_name: field.name()?.read_string(cs2)?,
            offset: field.offset()? as u64,
        });
    }

    Ok(class_offsets)
}

pub fn dump_schema(cs2: &CS2Handle) -> anyhow::Result<Vec<SchemaScope>> {
    let schema_system_offset = find_schema_system(cs2)?;
    let schema_system = cs2.reference_schema::<CSchemaSystem>(&[
        cs2.memory_address(Module::Schemasystem, schema_system_offset)?
    ])?;

    let scopes = schema_system.scopes()?;
    let scope_size = scopes.element_count()? as usize;
    log::debug!(
        "Schema system located at 0x{:X} (0x{:X}) containing 0x{:X} scopes",
        cs2.memory_address(Module::Schemasystem, schema_system_offset)?,
        schema_system_offset,
        scope_size
    );

    if scope_size > 0x20 {
        anyhow::bail!("Too many scopes ({}). Something went wrong?", scope_size);
    }

    let mut schema_scops = Vec::<SchemaScope>::with_capacity(scope_size);
    for scope_index in 0..scope_size {
        /* scope: CSchemaSystemTypeScope */
        let scope_ptr = scopes.reference_element(&cs2, scope_index)?;
        let scope = scope_ptr.read_schema(cs2)?;

        let class_bindings = scope.class_bindings()?.read_values(cs2)?;

        let mut scope_info: SchemaScope = Default::default();
        scope_info.schema_name = scope.scope_name()?.to_string_lossy()?;

        log::debug!(
            " {:X} {} ({})",
            scope_ptr.address(), scope_info.schema_name,
            class_bindings.len()
        );
        for schema_class in class_bindings {
            scope_info
                .classes
                .push(read_class_binding(cs2, &schema_class)?);
        }

        schema_scops.push(scope_info);
    }

    Ok(schema_scops)
}
