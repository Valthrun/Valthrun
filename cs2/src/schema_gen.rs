use std::{
    collections::{
        btree_map::Entry,
        BTreeMap,
    },
    ops::Deref,
};

use anyhow::Context;
use cs2_schema_cutl::{
    CStringUtil,
    FixedCStringUtil,
    PtrCStr,
};
use cs2_schema_definition::{
    mod_name_from_schema_name,
    ClassDefinition,
    ClassField,
    EnumDefinition,
    EnumMember,
    Metadata,
    SchemaScope,
};
use raw_struct::{
    builtins::Ptr64,
    Reference,
};
use utils_state::StateRegistry;

use crate::{
    schema::{
        AtomicCategory,
        CSchemaClassBinding,
        CSchemaEnumBinding,
        CSchemaMetadataEntry,
        CSchemaMetadataVarNames,
        CSchemaSystem,
        CSchemaType,
        CSchemaTypeAtomicCollectionOfT,
        CSchemaTypeAtomicT,
        CSchemaTypeDeclaredClass,
        CSchemaTypeDeclaredEnum,
        CSchemaTypeFixedArray,
        CSchemaTypePtr,
        TypeCategory,
    },
    CS2Offset,
    Module,
    StateCS2Handle,
    StateCS2Memory,
    StateResolvedOffset,
};

fn parse_metadata(
    states: &StateRegistry,
    metadata: &dyn CSchemaMetadataEntry,
) -> anyhow::Result<Metadata> {
    let memory = states.resolve::<StateCS2Memory>(())?;
    let name = metadata
        .name()?
        .read_string(memory.view())?
        .context("missing metadata name")?;

    let meta = match name.as_str() {
        "MNetworkEnable" => Metadata::NetworkEnable,
        "MNetworkDisable" => Metadata::NetworkDisable,
        "MNetworkChangeCallback" => {
            let name = metadata
                .metadata_value()?
                .cast::<PtrCStr>()
                .read_value(memory.view())?
                .context("missing network change callback ptr")?
                .read_string(memory.view())?
                .context("missing network change callback name")?;

            Metadata::NetworkChangeCallback { name }
        }
        "MNetworkVarNames" => {
            let meta_value = metadata
                .metadata_value()?
                .cast::<dyn CSchemaMetadataVarNames>()
                .value_copy(memory.view())?
                .context("missing network var names")?;

            Metadata::NetworkVarNames {
                var_name: meta_value
                    .var_name()?
                    .read_string(memory.view())?
                    .context("missing var name")?,
                var_type: meta_value
                    .var_type()?
                    .read_string(memory.view())?
                    .context("missing var type")?,
            }
        }
        _ => Metadata::Unknown { name },
    };

    Ok(meta)
}

fn parse_type(
    states: &StateRegistry,
    schema_type: &Reference<dyn CSchemaType>,
) -> anyhow::Result<Option<String>> {
    let memory = states.resolve::<StateCS2Memory>(())?;
    let result = match schema_type.type_category()? {
        TypeCategory::Builtin => {
            let var_type = schema_type
                .var_type()?
                .read_string(memory.view())?
                .context("missing var type str")?;

            let rust_type = match var_type.as_str() {
                "bool" => "bool",
                "char" => "u8",

                "int8" => "i8",
                "uint8" => "u8",

                "int16" => "i16",
                "uint16" => "u16",

                "int32" => "i32",
                "uint32" => "u32",

                "int64" => "i64",
                "uint64" => "u64",

                "float32" => "f32",
                "float64" => "f64",

                var_type => anyhow::bail!("Unknown builtin type {}", var_type),
            }
            .to_string();

            Some(rust_type)
        }
        TypeCategory::FixedArray => {
            let fixed_array = schema_type.cast::<dyn CSchemaTypeFixedArray>();
            let length = fixed_array.array_length()?; // FIXME: This value is invalid!
            let base_type = fixed_array
                .base_type()?
                .value_reference(memory.view_arc())
                .context("missing base type")?;
            let base_type =
                parse_type(states, &base_type).context("failed to generate array base type")?;

            base_type.map(|base_type| format!("[{};0x{:X}]", base_type, length))
        }
        TypeCategory::Ptr => {
            let schema_ptr = schema_type.cast::<dyn CSchemaTypePtr>();
            let base_type = schema_ptr
                .base_type()?
                .value_reference(memory.view_arc())
                .context("missing base type")?;
            let base_type = parse_type(states, &base_type)?;

            base_type.map(|base_type| format!("Ptr64<{}>", base_type))
        }
        TypeCategory::Atomic => {
            match schema_type.atomic_category()? {
                AtomicCategory::Basic => {
                    let value = schema_type
                        .var_type()?
                        .read_string(memory.view())?
                        .context("missing var type")?;

                    Some(
                        match value.as_str() {
                            "CEntityIndex" => "CEntityIndex",

                            "CUtlStringToken" => "CUtlStringToken",
                            "CUtlSymbolLarge" => "PtrCStr",
                            "CUtlString" => "CUtlString",
                            "Vector" => "[f32; 0x03]",
                            "QAngle" => "[f32; 0x04]",

                            "Color" => "Color", // TODO: What is this (3x or 4x f32?)?

                            "CNetworkedQuantizedFloat" => "f32",

                            _ => return Ok(None),
                        }
                        .to_string(),
                    )
                }
                AtomicCategory::CollectionOfT => {
                    let value = schema_type
                        .var_type()?
                        .read_string(memory.view())?
                        .context("missing var type")?;
                    if !value.starts_with("CUtlVector<")
                        || !value.starts_with("C_NetworkUtlVectorBase<")
                    {
                        return Ok(None);
                    }

                    let atomic_collection =
                        schema_type.cast::<dyn CSchemaTypeAtomicCollectionOfT>();
                    let inner_type = atomic_collection
                        .inner_type()?
                        .value_reference(memory.view_arc())
                        .context("missing inner type")?;
                    let inner_type = parse_type(states, &inner_type)?;

                    inner_type.map(|inner_type| format!("dyn CUtlVector<{}>", inner_type))
                }
                AtomicCategory::T => {
                    let value = schema_type
                        .var_type()?
                        .read_string(memory.view())?
                        .context("missing var type")?;

                    if !value.starts_with("CHandle<") {
                        return Ok(None);
                    }

                    let atomic_t = schema_type.cast::<dyn CSchemaTypeAtomicT>();
                    let inner_type = atomic_t
                        .inner_type()?
                        .value_reference(memory.view_arc())
                        .context("missing inner type")?;
                    let inner_type = parse_type(states, &inner_type)?;

                    inner_type.map(|inner_type| format!("EntityHandle<{}>", inner_type))
                }
                _ => return Ok(None),
            }
        }
        TypeCategory::DeclaredClass => {
            let type_class = schema_type.cast::<dyn CSchemaTypeDeclaredClass>();
            let type_class = type_class
                .declaration()?
                .value_copy(memory.view())?
                .context("missing declared class declaration")?;

            //let module_name = type_class.module_name()?.read_string(cs2)?;
            let module_name = type_class
                .type_scope()?
                .value_reference(memory.view_arc())
                .context("null type scope")?
                .scope_name()?
                .to_string_lossy()
                .to_string();

            let class_name = type_class
                .name()?
                .read_string(memory.view())?
                .context("missing class name")?
                .replace(":", "_");

            Some(format!(
                "{}::{}",
                mod_name_from_schema_name(&module_name),
                class_name
            ))
        }
        TypeCategory::DeclaredEnum => {
            let enum_binding = schema_type.cast::<dyn CSchemaTypeDeclaredEnum>();
            let enum_binding = enum_binding
                .declaration()?
                .value_copy(memory.view())?
                .context("missing declared enum declaration")?;

            //let module_name = enum_binding.module_name()?.read_string(cs2)?;
            let module_name = enum_binding
                .type_scope()?
                .value_reference(memory.view_arc())
                .context("null type scope")?
                .scope_name()?
                .to_string_lossy()
                .to_string();

            let enum_name = enum_binding
                .name()?
                .read_string(memory.view())?
                .context("missing enum name")?
                .replace(":", "_");

            Some(format!(
                "{}::{}",
                mod_name_from_schema_name(&module_name),
                enum_name
            ))
        }
        _ => return Ok(None),
    };

    Ok(result)
}

fn read_enum_binding(
    states: &StateRegistry,
    binding_ptr: &Ptr64<dyn CSchemaEnumBinding>,
) -> anyhow::Result<(String, EnumDefinition)> {
    let memory = states.resolve::<StateCS2Memory>(())?;
    let binding = binding_ptr
        .value_copy(memory.view())?
        .context("binding nullptr")?;
    let mut definition: EnumDefinition = Default::default();

    definition.enum_size = binding.size()? as usize;
    definition.enum_name = binding
        .name()?
        .read_string(memory.view())?
        .context("missing enum binding name")?;

    log::debug!("   {:X} {}", binding_ptr.address, definition.enum_name);
    definition
        .memebers
        .reserve(binding.member_count()? as usize);
    for member in binding
        .members()?
        .elements(memory.view(), 0..binding.member_count()? as usize)?
    {
        let member_name = member
            .name()?
            .read_string(memory.view())?
            .context("missing enum member name")?;

        let member_value = member.value()?;
        definition.memebers.push(EnumMember {
            name: member_name,
            value: member_value,
        });
    }

    let scope_name = binding
        .type_scope()?
        .value_reference(memory.view_arc())
        .context("missing type scope")?
        .scope_name()?
        .to_string_lossy()
        .to_string();

    definition.schema_scope_name = scope_name.clone();
    Ok((scope_name, definition))
}

pub fn read_class_scope_and_name(
    states: &StateRegistry,
    class: &dyn CSchemaClassBinding,
) -> anyhow::Result<(String, String)> {
    let memory = states.resolve::<StateCS2Memory>(())?;
    let module = class
        .type_scope()?
        .value_reference(memory.view_arc())
        .context("missing type scope")?
        .scope_name()?
        .to_string_lossy()
        .to_string();

    let name = class
        .name()?
        .read_string(memory.view())?
        .context("missing base class name")?;

    Ok((module, name))
}

fn read_class_binding(
    states: &StateRegistry,
    binding_ptr: &Ptr64<dyn CSchemaClassBinding>,
) -> anyhow::Result<(String, ClassDefinition)> {
    let memory = states.resolve::<StateCS2Memory>(())?;
    let binding = binding_ptr
        .value_copy(memory.view())?
        .context("class binding nullptr")?;

    let (class_type_scope_name, class_name) = read_class_scope_and_name(states, binding.deref())?;
    log::debug!(
        "   {:X} {} -> {}",
        binding_ptr.address,
        class_name,
        class_type_scope_name
    );

    let mut definition: ClassDefinition = Default::default();
    definition.schema_scope_name = class_type_scope_name.clone();
    definition.class_name = class_name.clone();
    definition.class_size = binding.size()? as u64;
    definition.offsets.reserve(binding.field_size()? as usize);

    let base_class = binding.base_class()?;
    if !base_class.is_null() {
        let base_class = base_class
            .value_reference(memory.view_arc())
            .context("missing value reference")?
            .class_binding()?
            .value_copy(memory.view())?
            .context("nullptr base class")?;

        let (class_type_scope_name, class_name) =
            read_class_scope_and_name(states, base_class.deref())?;

        let base_class = format!(
            "{}::{}",
            mod_name_from_schema_name(&class_type_scope_name),
            class_name.replace(":", "_")
        );

        definition.inherits = Some(base_class);
    }

    //log::debug!(" - {:X} {} ({}; {})", schema_class, class_offsets.class_name, binding.field_size, binding.size);
    for field in binding
        .fields()?
        .elements(memory.view(), 0..binding.field_size()? as usize)?
    {
        let metadata_size = field.metadata_size()? as usize;
        let mut metadata = Vec::with_capacity(metadata_size);
        for meta_entry in field
            .metadata()?
            .elements(memory.view(), 0..metadata_size)?
        {
            metadata.push(parse_metadata(states, meta_entry.deref())?);
        }

        /* needs a reference as we downcast the type later on and therefore increase the size */
        let field_type = field
            .field_type()?
            .value_reference(memory.view_arc())
            .context("missing field type")?;

        let c_type = field_type
            .var_type()?
            .read_string(memory.view())?
            .context("missing var c-type")?;

        let field_name = field
            .name()?
            .read_string(memory.view())?
            .context("missing field name")?;

        let rust_type = parse_type(states, &field_type)?;
        if rust_type.is_none() {
            /* Use debug here as warn will spam the log */
            log::debug!(
                "   Could not generate field type {} ({:?} / {:?}) for {}",
                &c_type,
                field_type.type_category()?,
                field_type.atomic_category()?,
                field_name,
            );
        }

        //log::debug!("    - {:X} {}", field.offset, field.name.read_string(cs2)?);
        definition.offsets.push(ClassField {
            field_name,

            field_type: rust_type,
            field_ctype: c_type,

            offset: field.offset()? as u64,
            metadata,
        });
    }

    {
        let metadata_size = binding.metadata_size()? as usize;
        definition.metadata.reserve(metadata_size);
        for meta_entry in binding
            .metadata()?
            .elements(memory.view(), 0..metadata_size)?
        {
            definition
                .metadata
                .push(parse_metadata(states, meta_entry.deref())?);
        }
    }

    Ok((class_type_scope_name, definition))
}

pub fn dump_schema(
    states: &StateRegistry,
    scope_filter: Option<&[&str]>,
) -> anyhow::Result<Vec<SchemaScope>> {
    let cs2 = states.resolve::<StateCS2Handle>(())?;
    let memory = states.resolve::<StateCS2Memory>(())?;

    let schema_system = states.resolve::<StateResolvedOffset>(CS2Offset::SchemaSystem)?;
    let system_instance =
        Reference::<dyn CSchemaSystem>::new(memory.view_arc(), schema_system.address);

    let scopes = system_instance.scopes()?;
    let scope_size = scopes.size()? as usize;
    log::debug!(
        "Schema system located at 0x{:X} (0x{:X}) containing 0x{:X} scopes",
        schema_system.address,
        cs2.module_address(Module::Schemasystem, schema_system.address)
            .context("invalid schema system address")?,
        scope_size
    );

    if scope_size > 0x20 {
        anyhow::bail!("Too many scopes ({}). Something went wrong?", scope_size);
    }

    let mut schema_scops = BTreeMap::<String, SchemaScope>::new();
    for scope_ptr in scopes
        .data()?
        .elements(memory.view(), 0..scopes.size()? as usize)?
    {
        let scope = scope_ptr
            .value_copy(memory.view())?
            .context("scope nullptr")?;

        let scope_name = scope.scope_name()?.to_string_lossy().to_string();
        log::trace!("Dumping scope {} @ {:X}", scope_name, scope_ptr.address);

        let declared_classes = scope.type_declared_class()?;
        let declared_classes = declared_classes.elements()?.elements_copy(
            memory.view(),
            0..declared_classes.highest_entry()?.wrapping_add(1) as usize,
        )?;

        for rb_node in declared_classes {
            let declared_class = rb_node
                .value()?
                .value
                .cast::<dyn CSchemaTypeDeclaredClass>()
                .value_reference(memory.view_arc())
                .context("tree null entry")?;

            let (class_scope_name, definition) =
                read_class_binding(states, &declared_class.declaration()?).context(format!(
                    "class binding {:X}",
                    declared_class.declaration()?.address
                ))?;

            if let Some(filter) = &scope_filter {
                if !filter.contains(&class_scope_name.as_str()) {
                    continue;
                }
            }

            let schema_scope = match schema_scops.entry(scope_name.clone()) {
                Entry::Occupied(entry) => entry.into_mut(),
                Entry::Vacant(entry) => {
                    let schema_name = entry.key().clone();
                    entry.insert(SchemaScope {
                        schema_name,

                        classes: Default::default(),
                        enums: Default::default(),
                    })
                }
            };

            schema_scope.classes.push(definition);
        }

        let declared_enums = scope.type_declared_enum()?;
        let declared_enums = declared_enums.elements()?.elements_copy(
            memory.view(),
            0..declared_enums.highest_entry()?.wrapping_add(1) as usize,
        )?;

        for declared_enum in declared_enums {
            let declared_enum = declared_enum
                .value()?
                .value
                .cast::<dyn CSchemaTypeDeclaredEnum>()
                .value_reference(memory.view_arc())
                .context("tree null entry")?;

            let (enum_scope_name, definition) =
                read_enum_binding(states, &declared_enum.declaration()?)?;
            if let Some(filter) = &scope_filter {
                if !filter.contains(&enum_scope_name.as_str()) {
                    continue;
                }
            }

            let schema_scope = match schema_scops.entry(scope_name.clone()) {
                Entry::Occupied(entry) => entry.into_mut(),
                Entry::Vacant(entry) => {
                    let schema_name = entry.key().clone();
                    entry.insert(SchemaScope {
                        schema_name,

                        classes: Default::default(),
                        enums: Default::default(),
                    })
                }
            };
            schema_scope.enums.push(definition);
        }
    }

    Ok(schema_scops.into_values().collect())
}
