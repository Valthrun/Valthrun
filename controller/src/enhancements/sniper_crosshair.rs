use cs2::{StateCS2Memory, StateEntityList, StateLocalPlayerController};
use cs2_schema_generated::cs2::client::{C_CSPlayerPawnBase, C_EconEntity};
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
        // AWP = 9, SSG08 = 40, SCAR-20 = 38, G3SG1 = 11
        matches!(weapon_id, 9 | 40 | 38 | 11)
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
        let local_controller = states.resolve::<StateLocalPlayerController>(())?;
        let view = states.resolve::<crate::view::ViewController>(())?;
        
        // Get local player pawn
        let local_pawn = match local_controller.instance.value_reference(memory.view_arc()) {
            Some(controller) => {
                let entities = states.resolve::<StateEntityList>(())?;
                match entities.entity_from_handle(&controller.m_hPlayerPawn()?) {
                    Some(pawn) => pawn.value_reference(memory.view_arc())
                        .ok_or_else(|| anyhow::anyhow!("pawn nullptr"))?,
                    None => return Ok(()),
                }
            }
            None => return Ok(()),
        };

        // Get active weapon
        let weapon = local_pawn.m_pClippingWeapon()?
            .value_reference(memory.view_arc())
            .ok_or_else(|| anyhow::anyhow!("weapon nullptr"))?
            .cast::<dyn C_EconEntity>();

        // Get weapon info through attribute manager
        let weapon_id = weapon
            .m_AttributeManager()?
            .m_Item()?
            .m_iItemDefinitionIndex()?;

        // Check if it's a sniper rifle
        if !self.is_sniper_weapon(weapon_id) {
            return Ok(());
        }

        let draw = ui.get_window_draw_list();
        let screen_center = [
            view.screen_bounds.x / 2.0,
            view.screen_bounds.y / 2.0,
        ];

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