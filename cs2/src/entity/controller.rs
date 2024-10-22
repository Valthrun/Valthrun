use anyhow::{
    anyhow,
    Context,
};
use cs2_schema_generated::cs2::client::{
    CCSPlayerController,
    CEntityInstance,
};
use raw_struct::{
    builtins::Ptr64,
    FromMemoryView,
};
use utils_state::{
    State,
    StateCacheType,
    StateRegistry,
};

use super::{
    CEntityIdentityEx,
    StateEntityList,
};
use crate::{
    CS2Offset,
    StateCS2Memory,
    StateResolvedOffset,
};

pub struct StateLocalPlayerController {
    pub instance: Ptr64<dyn CCSPlayerController>,
}

impl State for StateLocalPlayerController {
    type Parameter = ();

    fn create(states: &StateRegistry, _param: Self::Parameter) -> anyhow::Result<Self> {
        let memory = states.resolve::<StateCS2Memory>(())?;
        let offset = states.resolve::<StateResolvedOffset>(CS2Offset::LocalController)?;
        Ok(Self {
            instance: Ptr64::read_object(memory.view(), offset.address).map_err(|e| anyhow!(e))?,
        })
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Volatile
    }
}

struct StatePlayerControllerClass {
    address: u64,
}

impl State for StatePlayerControllerClass {
    type Parameter = ();

    fn create(states: &StateRegistry, _param: Self::Parameter) -> anyhow::Result<Self> {
        let memory = states.resolve::<StateCS2Memory>(())?;

        let local_controller = states.resolve::<StateLocalPlayerController>(())?;
        let Some(controller) = local_controller.instance.value_reference(memory.view_arc()) else {
            anyhow::bail!("missing local player controller")
        };

        let controller_class = controller
            .m_pEntity()?
            .value_reference(memory.view_arc())
            .context("m_pEntity nullptr")?
            .entity_class_info()?
            .address;

        Ok(Self {
            address: controller_class,
        })
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Persistent
    }
}

pub struct StatePlayerControllers {
    pub instances: Vec<Ptr64<dyn CCSPlayerController>>,
}

impl State for StatePlayerControllers {
    type Parameter = ();

    fn create(states: &StateRegistry, _param: Self::Parameter) -> anyhow::Result<Self> {
        let controller_class_address = states.resolve::<StatePlayerControllerClass>(())?;
        let entities = states.resolve::<StateEntityList>(())?;

        Ok(Self {
            instances: entities
                .entities()
                .iter()
                .filter(|entity| {
                    if let Ok(ptr) = entity.entity_class_info() {
                        ptr.address == controller_class_address.address
                    } else {
                        false
                    }
                })
                .map(|entity| entity.entity_ptr())
                .collect::<anyhow::Result<Vec<_>>>()?,
        })
    }
}
