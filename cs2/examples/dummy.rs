use std::borrow::Cow;

use anyhow::Context;
use cs2::{
    CEntityIdentityEx,
    CS2Handle,
    ClassNameCache,
    ConVars,
    StateBuildInfo,
    StateCS2Handle,
    StateCS2Memory,
    StateCurrentMap,
    StateEntityList,
    StateGlobals,
    StateLocalPlayerController,
    StatePlayerControllers,
};
use cs2_schema_cutl::FixedCStringUtil;
use cs2_schema_generated::cs2::client::CBasePlayerController;
use utils_state::StateRegistry;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let handle = CS2Handle::create(false)?;

    let mut state = StateRegistry::new(0xFF);
    let _ = state.set(StateCS2Handle::new(handle.clone()), ());
    let _ = state.set(StateCS2Memory::new(handle.create_memory_view()), ());
    state.invalidate_states();

    let memory = state.resolve::<StateCS2Memory>(())?;
    {
        let globals = state.resolve::<StateGlobals>(())?;
        let build_info = state.resolve::<StateBuildInfo>(())?;
        log::info!("Frame time: {:X}", globals.frame_count_1()?);
        log::info!("Build info: {:?}", build_info);
    }

    {
        let player_controllers = state.resolve::<StatePlayerControllers>(())?;
        log::info!("Player controllers: {}", player_controllers.instances.len());
        for controller in player_controllers.instances.iter() {
            let controller = controller
                .value_reference(memory.view_arc())
                .context("player controller nullptr")?;

            let player_name = controller.m_iszPlayerName()?.to_string_lossy().to_string();
            log::info!(" - {} ({})", player_name,);
        }
        return Ok(());
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
    }

    {
        let entities = state.resolve::<StateEntityList>(())?;
        let local_controller = state.resolve::<StateLocalPlayerController>(())?;
        log::info!("Local controller: {:X}", local_controller.instance.address);

        if let Some(controller) = local_controller.instance.value_reference(memory.view_arc()) {
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
