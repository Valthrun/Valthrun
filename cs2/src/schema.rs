use std::collections::{
    btree_map::Entry,
    BTreeMap,
};

use anyhow::Context;
use cs2_schema_cutl::{
    CUtlTSHash,
    CUtlVector,
};
use cs2_schema_declaration::{
    define_schema,
    FixedCString,
    Ptr,
    PtrCStr,
};
use cs2_schema_generated::definition::{
    mod_name_from_schema_name,
    ClassDefinition,
    ClassField,
    EnumDefinition,
    EnumMember,
    Metadata,
    SchemaScope,
};
use obfstr::obfstr;

use crate::{
    CS2Handle,
    Module,
    Signature,
};

// Returns SchemaSystem_001
pub fn find_schema_system(cs2: &CS2Handle) -> anyhow::Result<u64> {
    cs2.resolve_signature(
        Module::Schemasystem,
        &Signature::relative_address(
            obfstr!("schema system instance"),
            obfstr!("48 89 05 ? ? ? ? 4C 8D 45"),
            0x03,
            0x07,
        ),
    )
}

define_schema! {
    pub enum AtomicCategory : u8 {
        Basic = 0,
        T = 1,
        CollectionOfT = 2,
        TT = 3,
        I = 4,
        Unknown = 5,
        None = 6,
    }

    pub enum TypeCategory : u8 {
        Builtin = 0,
        Ptr = 1,
        Bitfield = 2,
        FixedArray = 3,
        Atomic = 4,
        DeclaredClass = 5,
        DeclaredEnum = 6,
        None = 7,
    }

    pub struct CSchemaSystem[0x200] {
        pub scopes: CUtlVector<Ptr<CSchemaSystemTypeScope>> = 0x190,
    }

    pub struct CSchemaSystemTypeScope[0x2F00] {
        pub scope_name: FixedCString<0x100> = 0x08,
        pub class_bindings: CUtlTSHash<u64, Ptr<CSchemaClassBinding>> = 0x0588,
        pub enum_bindings: CUtlTSHash<u64, Ptr<CSchemaEnumBinding>> = 0x2DD0,
    }

    pub struct CSchemaType[0x20] {
        pub vtable: u64 = 0x00,
        pub var_type: PtrCStr = 0x08,
        pub var_type_scope: Ptr<CSchemaSystemTypeScope> = 0x10,

        pub type_category: TypeCategory = 0x18,
        pub atomic_category: AtomicCategory = 0x19,
    }

    pub struct CSchemaTypeDeclaredEnum[0x28] : CSchemaType {
        pub declaration: Ptr<CSchemaEnumBinding> = 0x20,
    }

    pub struct CSchemaTypeDeclaredClass[0x28] : CSchemaType {
        pub declaration: Ptr<CSchemaClassBinding> = 0x20,
    }

    pub struct CSchemaTypePtr[0x30] : CSchemaType {
        pub base_type: Ptr<CSchemaType> = 0x20,
        /* unknown value which is sometimes 1 (maybe "*" count?) */
    }

    pub struct CSchemaTypeFixedArray[0x30] : CSchemaType {
        pub array_length: u32 = 0x20,
        /* rest is unknown */
        pub base_type: Ptr<CSchemaType> = 0x28,
    }

    pub struct CSchemaTypeAtomic[0x20] : CSchemaType {
        /* I'm unaware of "special fields" */
    }

    pub struct CSchemaTypeAtomicT[0x40] : CSchemaTypeAtomic {
        /* FIXME: Where is the "handle type" field? */
        pub inner_type: Ptr<CSchemaType> = 0x30,
    }

    pub struct CSchemaTypeAtomicCollectionOfT[0x40] : CSchemaTypeAtomicT { }

    pub struct CSchemaMetadataEntry[0x10] {
        pub name: PtrCStr = 0x00,

        // ptr to metadata value
        // - const char*
        // - float
        // - int
        // - void*
        pub metadata_value: Ptr<u64> = 0x08,
    }

    pub struct CSchemaMetadataVarNames[0x10] {
        pub var_name: PtrCStr = 0x00,
        pub var_type: PtrCStr = 0x08,
    }

    pub struct CSchemaClassField[0x20] {
        pub name: PtrCStr = 0x00,

        pub field_type: Ptr<CSchemaType> = 0x08,
        pub offset: u32 = 0x10,

        pub metadata_size: u32 = 0x14,
        pub metadata: Ptr<[CSchemaMetadataEntry]> = 0x18,
    }

    pub struct CSchemaClassBinding[0x68] {
        pub parent: Ptr<CSchemaClassBinding> = 0x00,
        pub name: PtrCStr = 0x08,
        pub module_name: PtrCStr = 0x10,

        pub size: u32 = 0x18, // Size of own struct

        pub field_size: u16 = 0x1C,
        pub static_size: u16 = 0x1E,
        pub metadata_size: u16 = 0x20,

        pub fields: Ptr<[CSchemaClassField]> = 0x28,
        /* pub static_fields: Ptr<[CSchemaStaticField]> = 0x30, */
        pub base_class: Ptr<CSchemaClassInheritance> = 0x38,
        pub metadata: Ptr<[CSchemaMetadataEntry]> = 0x48,

        pub type_scope: Ptr<CSchemaSystemTypeScope> = 0x50,
        pub schema_type: Ptr<CSchemaType> = 0x58,

        pub flags: u64 = 0x60,
    }

    pub struct CSchemaClassInheritance[0x10] {
        pub class_binding: Ptr<CSchemaClassBinding> = 0x08,
    }

    pub struct CSchemaEnumBinding[0x40] {
        pub name: PtrCStr = 0x08,
        pub module_name: PtrCStr = 0x10,

        pub size: u8 = 0x18, // Size of own struct
        pub member_count: u16 = 0x1C,
        pub flags: u16 = 0x1E,

        pub members: Ptr<[CSchemaEnumMember]> = 0x20,
        pub type_scope: Ptr<CSchemaSystemTypeScope> = 0x30,
    }

    pub struct CSchemaEnumMember[0x20] {
        pub name: PtrCStr = 0x00,
        pub value: u64 = 0x08,
    }
}

fn parse_metadata(metadata: &CSchemaMetadataEntry) -> anyhow::Result<Metadata> {
    let name = metadata.name()?.read_string()?;

    let meta = match name.as_str() {
        "MNetworkEnable" => Metadata::NetworkEnable,
        "MNetworkDisable" => Metadata::NetworkDisable,
        "MNetworkChangeCallback" => {
            let name = metadata
                .metadata_value()?
                .cast::<PtrCStr>()
                .read_schema()?
                .read_string()?;

            Metadata::NetworkChangeCallback { name }
        }
        "MNetworkVarNames" => {
            let meta_value = metadata
                .metadata_value()?
                .cast::<CSchemaMetadataVarNames>()
                .read_schema()?;

            Metadata::NetworkVarNames {
                var_name: meta_value.var_name()?.read_string()?,
                var_type: meta_value.var_type()?.read_string()?,
            }
        }
        _ => Metadata::Unknown { name },
    };

    Ok(meta)
}

fn parse_type(cs2: &CS2Handle, schema_type: &CSchemaType) -> anyhow::Result<Option<String>> {
    let result = match schema_type.type_category()? {
        TypeCategory::Builtin => {
            let var_type = schema_type.var_type()?.read_string()?;
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

                var_type => anyhow::bail!("Unknown build in type {}", var_type),
            }
            .to_string();

            Some(rust_type)
        }
        TypeCategory::FixedArray => {
            let fixed_array = schema_type.as_schema::<CSchemaTypeFixedArray>()?;
            let length = fixed_array.array_length()?; // FIXME: This value is invalid!
            let base_type = fixed_array.base_type()?.reference_schema()?;
            let base_type =
                parse_type(cs2, &base_type).context("failed to generate array base type")?;

            base_type.map(|base_type| format!("[{};0x{:X}]", base_type, length))
        }
        TypeCategory::Ptr => {
            let schema_ptr = schema_type.as_schema::<CSchemaTypePtr>()?;
            let base_type = schema_ptr.base_type()?.reference_schema()?;
            let base_type = parse_type(cs2, &base_type)?;

            base_type.map(|base_type| format!("Ptr<{}>", base_type))
        }
        TypeCategory::Atomic => {
            match schema_type.atomic_category()? {
                AtomicCategory::Basic => {
                    let value = schema_type.var_type()?.read_string()?;
                    Some(
                        match value.as_str() {
                            "CEntityIndex" => "CEntityIndex",

                            "CUtlStringToken" => "CUtlStringToken",
                            "CUtlSymbolLarge" => "PtrCStr",
                            "CUtlString" => "CUtlString",
                            "Vector" => "[f32; 0x03]",
                            "QAngle" => "[f32; 0x04]",

                            "Color" => "Color", // TODO: What is this (3x or 4x f32?)?

                            _ => return Ok(None),
                        }
                        .to_string(),
                    )
                }
                AtomicCategory::CollectionOfT => {
                    let value = schema_type.var_type()?.read_string()?;
                    if !value.starts_with("CUtlVector<") {
                        return Ok(None);
                    }

                    let atomic_collection =
                        schema_type.as_schema::<CSchemaTypeAtomicCollectionOfT>()?;
                    let inner_type = atomic_collection.inner_type()?.reference_schema()?;
                    let inner_type = parse_type(cs2, &inner_type)?;

                    inner_type.map(|inner_type| format!("CUtlVector<{}>", inner_type))
                }
                AtomicCategory::T => {
                    let value = schema_type.var_type()?.read_string()?;
                    if !value.starts_with("CHandle<") {
                        return Ok(None);
                    }

                    let atomic_t = schema_type.as_schema::<CSchemaTypeAtomicT>()?;
                    let inner_type = atomic_t.inner_type()?.reference_schema()?;
                    let inner_type = parse_type(cs2, &inner_type)?;

                    inner_type.map(|inner_type| format!("EntityHandle<{}>", inner_type))
                }
                _ => return Ok(None),
            }
        }
        TypeCategory::DeclaredClass => {
            let type_class = schema_type.as_schema::<CSchemaTypeDeclaredClass>()?;
            let type_class = type_class.declaration()?.read_schema()?;

            //let module_name = type_class.module_name()?.read_string(cs2)?;
            let module_name = type_class
                .type_scope()?
                .read_schema()?
                .scope_name()?
                .to_string_lossy()?;
            let class_name = type_class.name()?.read_string()?.replace(":", "_");
            Some(format!(
                "{}::{}",
                mod_name_from_schema_name(&module_name),
                class_name
            ))
        }
        TypeCategory::DeclaredEnum => {
            let enum_binding = schema_type.as_schema::<CSchemaTypeDeclaredEnum>()?;
            let enum_binding = enum_binding.declaration()?.read_schema()?;

            //let module_name = enum_binding.module_name()?.read_string(cs2)?;
            let module_name = enum_binding
                .type_scope()?
                .read_schema()?
                .scope_name()?
                .to_string_lossy()?;
            let enum_name = enum_binding.name()?.read_string()?.replace(":", "_");
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
    binding_ptr: &Ptr<CSchemaEnumBinding>,
) -> anyhow::Result<(String, EnumDefinition)> {
    let binding = binding_ptr.read_schema()?;
    let mut definition: EnumDefinition = Default::default();

    definition.enum_size = binding.size()? as usize;
    definition.enum_name = binding.name()?.read_string()?;

    log::debug!("   {:X} {}", binding_ptr.address()?, definition.enum_name);
    definition
        .memebers
        .reserve(binding.member_count()? as usize);
    for index in 0..binding.member_count()? as usize {
        let member = binding.members()?.reference_element(index)?;
        let member_name = member.name()?.read_string()?;
        let member_value = member.value()?;
        definition.memebers.push(EnumMember {
            name: member_name,
            value: member_value,
        });
    }

    Ok((
        binding
            .type_scope()?
            .reference_schema()?
            .scope_name()?
            .to_string_lossy()?,
        definition,
    ))
}

fn read_class_binding(
    cs2: &CS2Handle,
    binding_ptr: &Ptr<CSchemaClassBinding>,
) -> anyhow::Result<(String, ClassDefinition)> {
    let binding = binding_ptr.read_schema()?;
    log::debug!(
        "   {:X} {} -> {}",
        binding_ptr.address()?,
        binding.name()?.read_string()?,
        binding
            .type_scope()?
            .read_schema()?
            .scope_name()?
            .to_string_lossy()?
    );

    let mut definition: ClassDefinition = Default::default();
    definition.class_name = binding.name()?.read_string()?;
    definition.class_size = binding.size()? as u64;
    definition.offsets.reserve(binding.field_size()? as usize);

    let base_class = binding.base_class()?;
    if !base_class.is_null()? {
        let base_class = base_class
            .reference_schema()?
            .class_binding()?
            .read_schema()?;

        let class_module = base_class
            .type_scope()?
            .reference_schema()?
            .scope_name()?
            .to_string_lossy()?;
        let base_class = format!(
            "{}::{}",
            mod_name_from_schema_name(&class_module),
            base_class.name()?.read_string()?.replace(":", "_")
        );

        definition.inherits = Some(base_class);
    }

    //log::debug!(" - {:X} {} ({}; {})", schema_class, class_offsets.class_name, binding.field_size, binding.size);
    for field_index in 0..binding.field_size()? as usize {
        let field = binding.fields()?.read_element(field_index)?;
        /* needs a reference as we downcast the type later on and therefore increase the size */
        let field_type = field.field_type()?.reference_schema()?;

        let mut metadata = Vec::with_capacity(field.metadata_size()? as usize);
        for index in 0..field.metadata_size()? as usize {
            let meta_entry = field.metadata()?.read_element(index)?;
            metadata.push(parse_metadata(&meta_entry)?);
        }

        let c_type = field_type.var_type()?.read_string()?;
        let rust_type = parse_type(cs2, &field_type)?;
        if rust_type.is_none() {
            /* Use debug here as warn will spam the log */
            log::debug!(
                "   Could not generate field type {} ({:?} / {:?}) for {}",
                &c_type,
                field_type.type_category()?,
                field_type.atomic_category()?,
                field.name()?.read_string()?,
            );
        }

        //log::debug!("    - {:X} {}", field.offset, field.name.read_string(cs2)?);
        definition.offsets.push(ClassField {
            field_name: field.name()?.read_string()?,

            field_type: rust_type,
            field_ctype: c_type,

            offset: field.offset()? as u64,
            metadata,
        });
    }

    definition
        .metadata
        .reserve(binding.metadata_size()? as usize);
    for index in 0..binding.metadata_size()? as usize {
        let metadata = &binding.metadata()?.read_element(index)?;
        definition
            .metadata
            .push(parse_metadata(metadata).context("metadata parse")?);
    }

    Ok((
        binding
            .type_scope()?
            .reference_schema()?
            .scope_name()?
            .to_string_lossy()?,
        definition,
    ))
}

pub fn dump_schema(cs2: &CS2Handle, client_only: bool) -> anyhow::Result<Vec<SchemaScope>> {
    let schema_system_address = find_schema_system(cs2)?;
    let schema_system = cs2.reference_schema::<CSchemaSystem>(&[schema_system_address])?;

    let scopes = schema_system.scopes()?;
    let scope_size = scopes.element_count()? as usize;
    log::debug!(
        "Schema system located at 0x{:X} (0x{:X}) containing 0x{:X} scopes",
        schema_system_address,
        cs2.module_address(Module::Schemasystem, schema_system_address)
            .context("invalid schema system address")?,
        scope_size
    );

    if scope_size > 0x20 {
        anyhow::bail!("Too many scopes ({}). Something went wrong?", scope_size);
    }

    let mut schema_scops = BTreeMap::<String, SchemaScope>::new();
    for scope_index in 0..scope_size {
        /* scope: CSchemaSystemTypeScope */
        let scope_ptr = scopes.reference_element(scope_index)?;
        let scope = scope_ptr.read_schema()?;

        let scope_name = scope.scope_name()?.to_string_lossy()?;
        if client_only && (scope_name != "client.dll" && scope_name != "!GlobalTypes") {
            //continue;
        }

        let class_bindings = scope.class_bindings()?.read_values()?;
        let enum_bindings = scope.enum_bindings()?.read_values()?;
        log::debug!(
            " {:X} {} ({} classes, {} enums)",
            scope_ptr.address()?,
            scope_name,
            class_bindings.len(),
            enum_bindings.len(),
        );
        for schema_class in class_bindings {
            let (scope_name, definition) = read_class_binding(cs2, &schema_class)?;
            let schema_scope = match schema_scops.entry(scope_name) {
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

        for enum_binding in enum_bindings {
            let (scope_name, definition) = read_enum_binding(&enum_binding)?;
            let schema_scope = match schema_scops.entry(scope_name) {
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
