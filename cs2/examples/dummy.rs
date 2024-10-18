use std::{
    borrow::Cow,
    ops::Deref,
};

use anyhow::{
    anyhow,
    Context,
};
use cs2::{
    CEntityIdentityEx,
    CS2Handle,
    CS2Offset,
    ClassNameCache,
    ConVars,
    StateBuildInfo,
    StateCS2Handle,
    StateCS2Memory,
    StateCurrentMap,
    StateEntityList,
    StateGlobals,
    StateLocalPlayerController,
    StateResolvedOffset,
};
use cs2_schema_cutl::CStringUtil;
use cs2_schema_generated::cs2::client::{
    CBasePlayerController,
    CSkeletonInstance,
    C_BaseEntity,
};
use raw_struct::{
    builtins::Ptr64,
    Copy,
    FromMemoryView,
};
use utils_state::StateRegistry;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let handle = CS2Handle::create(false)?;
    let memory_view = handle.create_memory_view();

    let mut state = StateRegistry::new(0xFF);
    let _ = state.set(StateCS2Handle::new(handle.clone()), ());
    let _ = state.set(StateCS2Memory::new(memory_view.clone()), ());
    state.invalidate_states();

    {
        let globals = state.resolve::<StateGlobals>(())?;
        let build_info = state.resolve::<StateBuildInfo>(())?;
        log::info!("Frame time: {:X}", globals.frame_count_1()?);
        log::info!("Build info: {:?}", build_info);
    }

    {
        let cvars = ConVars::new(&state)?;
        let cvar_sensitivity = cvars
            .find_cvar("sensitivity")?
            .context("missing sensitivity")?;

        log::info!("Sensitivity: {}", cvar_sensitivity.fl_value()?);
    }

    {
        let current_map = state.resolve::<StateCurrentMap>(())?;
        log::info!("Current map: {:?}", current_map.current_map);
    }

    {
        let entities = state.resolve::<StateEntityList>(())?;
        let class_names = state.resolve::<ClassNameCache>(())?;

        log::info!("Entities: {}", entities.entities().len());
        for entity in entities.entities() {
            let class_name = class_names.lookup(&entity.entity_class_info()?)?;
            log::info!(
                " - {} @ {:X} {}",
                entity.handle::<()>()?.get_entity_index(),
                entity.entity_ptr::<()>()?.address,
                class_name.map_or("<unknown>".into(), Cow::from)
            );
        }

        let local_controller = state.resolve::<StateLocalPlayerController>(())?;
        log::info!("Local controller: {:X}", local_controller.instance.address);

        if let Some(controller) = local_controller
            .instance
            .value_reference(memory_view.clone())
        {
            if let Some(pawn) = entities.entity_from_handle(&controller.m_hPawn()?) {
                log::info!("Local pawn: {:X}", pawn.address);
                //let skeleton = pawn.value_reference(memory_view.clone())?.m_pGameSceneNode()?.value_reference(memory_view.clone())?.cast::<dyn CSkeletonInstance>().m_modelState()?.m
            } else {
                log::info!("No local pawn");
            }
        }
    }
    Ok(())
}
