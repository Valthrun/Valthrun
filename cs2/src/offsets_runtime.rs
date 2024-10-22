use std::{
    collections::BTreeMap,
    ops::Deref,
    sync::RwLock,
};

use anyhow::Context;
use cs2_schema_cutl::{
    CStringUtil,
    FixedCStringUtil,
    OffsetInfo,
};
use raw_struct::Reference;
use utils_state::StateRegistry;

use crate::{
    read_class_scope_and_name,
    schema::{
        CSchemaSystem,
        CSchemaTypeDeclaredClass,
    },
    CS2Offset,
    Module,
    StateCS2Handle,
    StateCS2Memory,
    StateResolvedOffset,
};

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
struct RegisteredOffset {
    module: String,
    class: String,
    member: String,
}

impl From<OffsetInfo> for RegisteredOffset {
    fn from(value: OffsetInfo) -> Self {
        Self {
            module: value.module.to_string(),
            class: value.class_name.to_string(),
            member: value.member.to_string(),
        }
    }
}

type Offset = u64;
struct CS2RuntimeOffsets {
    offsets: BTreeMap<RegisteredOffset, Offset>,
}

impl CS2RuntimeOffsets {
    fn resolve(&self, offset: &OffsetInfo) -> u64 {
        log::trace!("Try resolve {:X?}", offset);
        let offset = RegisteredOffset::from(offset.clone());
        let Some(result) = self.offsets.get(&offset) else {
            panic!(
                "unknown offset for {}::{} in {}",
                offset.class, offset.member, offset.module
            )
        };

        log::trace!(" -> 0x{:X}", *result);
        *result as u64
    }
}

fn load_runtime_offsets(
    states: &StateRegistry,
) -> anyhow::Result<BTreeMap<RegisteredOffset, Offset>> {
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

    let mut result: BTreeMap<RegisteredOffset, Offset> = BTreeMap::new();
    for scope_ptr in scopes
        .data()?
        .elements(memory.view(), 0..scopes.size()? as usize)?
    {
        let scope = scope_ptr
            .value_copy(memory.view())?
            .context("scope nullptr")?;

        let scope_name = scope.scope_name()?.to_string_lossy().to_string();
        log::trace!("Name: {} @ {:X}", scope_name, scope_ptr.address);

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

            let schema_class = declared_class.declaration()?;
            let binding = schema_class
                .value_copy(memory.view())?
                .context("class declaration ptr null")?;

            let (class_type_scope_name, class_name) =
                read_class_scope_and_name(states, binding.deref())?;
            log::trace!(
                "   {:X} {} -> {}",
                schema_class.address,
                class_name,
                class_type_scope_name
            );
            if !["client.dll", "!GlobalTypes"].contains(&class_type_scope_name.as_str()) {
                continue;
            }

            for class_member in binding
                .fields()?
                .elements(memory.view(), 0..binding.field_size()? as usize)?
            {
                let member_name = class_member
                    .name()?
                    .read_string(memory.view())?
                    .context("missing class member name")?;
                let member_offset = class_member.offset()? as Offset;

                result.insert(
                    RegisteredOffset {
                        module: class_type_scope_name.clone(),
                        class: class_name.clone(),
                        member: member_name,
                    },
                    member_offset,
                );
            }
        }
    }

    Ok(result)
}

static RUNTIME_OFFSETS_INSTANCE: RwLock<Option<CS2RuntimeOffsets>> = RwLock::new(None);

fn runtime_offset_resolver(offset: &OffsetInfo) -> u64 {
    let instance = RUNTIME_OFFSETS_INSTANCE.read().unwrap();
    let Some(instance) = instance.as_ref() else {
        panic!("no runtime offsets instance set");
    };

    instance.resolve(offset)
}

pub fn setup_provider(states: &StateRegistry) -> anyhow::Result<()> {
    let offsets = load_runtime_offsets(states)?;
    log::debug!("Loaded {} schema offsets", offsets.len());

    {
        let mut offsets_instance = RUNTIME_OFFSETS_INSTANCE.write().unwrap();
        *offsets_instance = Some(CS2RuntimeOffsets { offsets });
    }
    cs2_schema_cutl::set_offset_resolver(&runtime_offset_resolver);
    Ok(())
}
