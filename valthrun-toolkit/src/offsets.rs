use std::{
    collections::BTreeMap,
    sync::Arc,
};

use anyhow::Context;
use cs2::{
    find_schema_system,
    CS2Handle,
    CSchemaSystem,
    Module,
};
use cs2_schema_generated::{
    RuntimeOffset,
    RuntimeOffsetProvider,
};

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
struct RegisteredOffset {
    module: String,
    class: String,
    member: String,
}

type Offset = u32;
struct CS2RuntimeOffsets {
    offsets: BTreeMap<RegisteredOffset, Offset>,
}

impl RuntimeOffsetProvider for CS2RuntimeOffsets {
    fn resolve(&self, offset: &RuntimeOffset) -> anyhow::Result<u64> {
        log::trace!("Try resolve {:?}", offset);

        let offset = RegisteredOffset {
            module: offset.module.to_string(),
            class: offset.class.to_string(),
            member: offset.member.to_string(),
        };
        let result = self.offsets.get(&offset).with_context(|| {
            format!(
                "unknown offset for {}::{} in {}",
                offset.class, offset.member, offset.module
            )
        })?;

        log::trace!(" -> {:X}", *result);
        Ok(*result as u64)
    }
}

fn load_runtime_offsets(
    cs2: &Arc<CS2Handle>,
) -> anyhow::Result<BTreeMap<RegisteredOffset, Offset>> {
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

    let mut result: BTreeMap<RegisteredOffset, Offset> = BTreeMap::new();
    for scope_index in 0..scope_size {
        /* scope: CSchemaSystemTypeScope */
        let scope_ptr = scopes.reference_element(scope_index)?;
        let scope = scope_ptr.read_schema()?;

        let scope_name = scope.scope_name()?.to_string_lossy()?;

        let class_bindings = scope.class_bindings()?.read_values()?;
        log::trace!(
            " {:X} {} with {} classes",
            scope_ptr.address()?,
            scope_name,
            class_bindings.len(),
        );

        for schema_class in class_bindings {
            let binding = schema_class.read_schema()?;
            let schema_name = binding
                .type_scope()?
                .read_schema()?
                .scope_name()?
                .to_string_lossy()?;

            let class_name: String = binding.name()?.read_string()?;
            log::trace!(
                "   {:X} {} -> {}",
                schema_class.address()?,
                class_name,
                schema_name
            );
            if !["client.dll", "!GlobalTypes"].contains(&schema_name.as_str()) {
                continue;
            }

            let class_member = binding
                .fields()?
                .read_entries(binding.field_size()? as usize)?;

            for class_member in class_member {
                let member_name = class_member.name()?.read_string()?;
                let member_offset = class_member.offset()?;

                result.insert(
                    RegisteredOffset {
                        module: schema_name.clone(),
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

pub fn setup_runtime_offset_provider(cs2: &Arc<CS2Handle>) -> anyhow::Result<()> {
    let offsets = load_runtime_offsets(cs2)?;
    log::debug!("Loaded {} schema offsets", offsets.len());
    cs2_schema_generated::setup_runtime_offset_provider(Box::new(CS2RuntimeOffsets { offsets }));
    Ok(())
}
