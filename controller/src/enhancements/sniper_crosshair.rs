use anyhow::Context;
use cs2::{
    CEntityIdentityEx,
    ClassNameCache,
    LocalCameraControllerTarget,
    StateCS2Memory,
    StateEntityList,
    WeaponId,
};
use cs2_schema_generated::cs2::client::{
    CBasePlayerController,
    C_CSObserverPawn,
    C_CSPlayerPawnBase,
    C_EconEntity,
};
use overlay::UnicodeTextRenderer;
use utils_state::StateRegistry;

use super::Enhancement;
use crate::settings::AppSettings;

pub struct SniperCrosshair;

impl SniperCrosshair {
    pub fn new() -> Self {
        Self
    }

    fn is_sniper_weapon(&self, weapon_id: u16) -> bool {
        matches!(
            WeaponId::from_id(weapon_id).unwrap_or(WeaponId::Unknown),
            WeaponId::AWP | WeaponId::Ssg08 | WeaponId::Scar20 | WeaponId::G3SG1
        )
    }

    fn get_active_weapon(
        &self,
        entities: &StateEntityList,
        memory: &StateCS2Memory,
        class_name_cache: &ClassNameCache,
        target_entity_id: u32,
    ) -> anyhow::Result<Option<u16>> {
        let entity_identity = entities
            .identity_from_index(target_entity_id)
            .context("missing entity identity")?;

        let entity_class = class_name_cache
            .lookup(&entity_identity.entity_class_info()?)?
            .context("failed to resolve entity class")?;

        match entity_class.as_str() {
            "C_CSPlayerPawn" => {
                // Handle normal player pawn
                let player_pawn = entity_identity
                    .entity_ptr::<dyn C_CSPlayerPawnBase>()?
                    .value_reference(memory.view_arc())
                    .context("player pawn nullptr")?;

                let weapon_ref = match player_pawn
                    .m_pClippingWeapon()?
                    .value_reference(memory.view_arc())
                {
                    Some(weapon) => weapon,
                    None => return Ok(None),
                };

                let weapon = weapon_ref.cast::<dyn C_EconEntity>();
                Ok(Some(
                    weapon
                        .m_AttributeManager()?
                        .m_Item()?
                        .m_iItemDefinitionIndex()?,
                ))
            }
            "C_CSObserverPawn" => {
                // Handle observer pawn
                let observer_pawn = entity_identity
                    .entity_ptr::<dyn C_CSObserverPawn>()?
                    .value_reference(memory.view_arc())
                    .context("observer pawn nullptr")?;

                let observer_controller_handle = observer_pawn.m_hOriginalController()?;
                let current_player_controller = entities
                    .entity_from_handle(&observer_controller_handle)
                    .context("missing observer controller")?
                    .value_reference(memory.view_arc())
                    .context("nullptr")?
                    .cast::<dyn CBasePlayerController>();

                // Get the player pawn from the controller
                let player_pawn_handle = current_player_controller.m_hPawn()?;
                let player_pawn = entities
                    .entity_from_handle(&player_pawn_handle)
                    .context("missing player pawn")?
                    .value_reference(memory.view_arc())
                    .context("player pawn nullptr")?
                    .cast::<dyn C_CSPlayerPawnBase>();

                let weapon_ref = match player_pawn
                    .m_pClippingWeapon()?
                    .value_reference(memory.view_arc())
                {
                    Some(weapon) => weapon,
                    None => return Ok(None),
                };

                let weapon = weapon_ref.cast::<dyn C_EconEntity>();
                Ok(Some(
                    weapon
                        .m_AttributeManager()?
                        .m_Item()?
                        .m_iItemDefinitionIndex()?,
                ))
            }
            _ => Ok(None),
        }
    }
}

impl Enhancement for SniperCrosshair {
    fn update(&mut self, _ctx: &crate::UpdateContext) -> anyhow::Result<()> {
        Ok(())
    }

    fn render(
        &self,
        states: &StateRegistry,
        ui: &imgui::Ui,
        _unicode_text: &UnicodeTextRenderer,
    ) -> anyhow::Result<()> {
        let settings = states.resolve::<AppSettings>(())?;
        if !settings.sniper_crosshair {
            return Ok(());
        }

        let memory = states.resolve::<StateCS2Memory>(())?;
        let entities = states.resolve::<StateEntityList>(())?;
        let view = states.resolve::<crate::view::ViewController>(())?;
        let class_name_cache = states.resolve::<ClassNameCache>(())?;
        let view_target = states.resolve::<LocalCameraControllerTarget>(())?;

        // Get the current target entity ID (whether local player or being spectated)
        let target_entity_id = match view_target.target_entity_id {
            Some(id) => id,
            None => return Ok(()),
        };

        // Get weapon ID from either player pawn or observer pawn
        let weapon_id = match self.get_active_weapon(
            &entities,
            &memory,
            &class_name_cache,
            target_entity_id,
        )? {
            Some(id) => id,
            None => return Ok(()),
        };

        // Check if it's a sniper rifle
        if !self.is_sniper_weapon(weapon_id) {
            return Ok(());
        }

        let draw = ui.get_window_draw_list();
        let screen_center = [view.screen_bounds.x / 2.0, view.screen_bounds.y / 2.0];

        // Draw shadow (black outline)
        draw.add_circle(screen_center, 3.5, [0.0, 0.0, 0.0, 0.8])
            .filled(true)
            .build();

        // Draw white dot
        draw.add_circle(screen_center, 2.0, [1.0, 1.0, 1.0, 0.8])
            .filled(true)
            .build();

        Ok(())
    }

    fn render_debug_window(
        &mut self,
        _states: &StateRegistry,
        _ui: &imgui::Ui,
        _unicode_text: &UnicodeTextRenderer,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
