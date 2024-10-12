use std::time::{Instant};
use cs2::{EntitySystem, CEntityIdentityEx};
use nalgebra::Vector3;
use obfstr::obfstr;
use cs2::BoneFlags;
use cs2::CS2Model;
use cs2::ClassNameCache;
use cs2::PlayerPawnState;
use crate::UnicodeTextRenderer;

use crate::settings::AppSettings;
use crate::view::{KeyToggle, ViewController};
use crate::UpdateContext;
use super::Enhancement;
use valthrun_kernel_interface::MouseState;


pub struct Aimbot {
    toggle: KeyToggle,
    fov: f32,                          // FOV fetched from settings
    aim_speed: f32,                    // Aim speed fetched from settings
    is_active: bool,
    last_mouse_move: Instant,          // Track the time for constant mouse down movement
    current_target: Option<[f32; 2]>,  // Store the current target
    is_mouse_pressed: bool,            // Track if the mouse is pressed
    aim_bone: String,
    aimbot_team_check: bool,
}

impl Aimbot {
    pub fn new() -> Self {
        Self {
            toggle: KeyToggle::new(),
            fov: 3.0,                   // Default FOV, updated dynamically from settings
            aim_speed: 2.5,              // Default aim speed, updated dynamically from settings
            is_active: false,
            last_mouse_move: Instant::now(), // Initialize the timer
            current_target: None,
            is_mouse_pressed: false,
            aim_bone: "head".to_string(),
            aimbot_team_check: true,
        }
    }

    fn world_to_screen(&self, view: &ViewController, world_position: &Vector3<f32>) -> Option<[f32; 2]> {
        view.world_to_screen(world_position, true).map(|vec| [vec.x, vec.y])
    }

    fn find_best_target(&mut self, ctx: &UpdateContext) -> Option<[f32; 2]> {
        if self.is_mouse_pressed && self.current_target.is_some() {
            return self.current_target;
        }

        let settings = ctx.states.resolve::<AppSettings>(()).ok()?;

        let entities = ctx.states.resolve::<EntitySystem>(()).ok()?;
        let view = ctx.states.resolve::<ViewController>(()).ok()?;
        let class_name_cache = ctx.states.resolve::<ClassNameCache>(()).ok()?; // Store the result here

        let local_player_position = view.get_camera_world_position()?; // Get local player position
        let get_local_player_controller = entities.get_local_player_controller().ok()?;
        let local_player_controller = get_local_player_controller.reference_schema().ok()?;
        let crosshair_pos = [view.screen_bounds.x / 2.0, view.screen_bounds.y / 2.0]; // Center of the screen
        let mut best_target: Option<[f32; 2]> = None;
        let mut lowest_distance_from_crosshair = f32::MAX; // Track the closest target in FOV
        const UNITS_TO_METERS: f32 = 0.01905;

        for entity_identity in entities.all_identities() {
            let entity_class = class_name_cache.lookup(&entity_identity.entity_class_info().ok()?).ok()?; // Use the stored value
            if entity_class.map(|name| *name == "C_CSPlayerPawn").unwrap_or(false) {
                let entry = ctx.states.resolve::<PlayerPawnState>(entity_identity.handle::<()>().ok()?.get_entity_index()).ok()?;
                if let PlayerPawnState::Alive(player_info) = &*entry {
                    let entry_model = ctx.states.resolve::<CS2Model>(player_info.model_address).ok()?;

                    // ignore flash

                    //player alive check

                    //player in gui check

                    // respect wall check

                    //team_mate check
                    if settings.aimbot_team_check && local_player_controller.m_iTeamNum().unwrap() == player_info.team_id {
                        continue;
                    }

                    // Calculate distance between the local player and the target
                    let distance = (player_info.position - local_player_position).norm() * UNITS_TO_METERS;

                    // Skip self (local player) if distance is < 3.0
                    if distance < 2.0 {
                        continue;
                    }

                    for (bone, state) in entry_model.bones.iter().zip(player_info.bone_states.iter()) {
                        if (bone.flags & BoneFlags::FlagHitbox as u32) == 0 {
                            continue;
                        }

                        if bone.name.to_lowercase().contains(&self.aim_bone) {
                            // Convert the selected bone position to screen space
                            if let Some(screen_position) = self.world_to_screen(&view, &state.position) {
                                // Calculate the distance from the crosshair (X and Y axes)
                                let dx = screen_position[0] - crosshair_pos[0];
                                let dy = screen_position[1] - crosshair_pos[1];
                                let distance_from_crosshair = (dx * dx + dy * dy).sqrt();

                                // Calculate the angle between the crosshair and the target
                                let angle_to_target = distance_from_crosshair.atan2(view.screen_bounds.x / 2.0).to_degrees();

                                // Check if the target is within the FOV
                                if angle_to_target <= self.fov / 2.0 {
                                    // If the bone is closer than the previous best target, update the best target
                                    if distance_from_crosshair < lowest_distance_from_crosshair {
                                        lowest_distance_from_crosshair = distance_from_crosshair;
                                        best_target = Some(screen_position);  // Update the target to the selected bone
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Update current target when the mouse is pressed and a new best target is found
        if self.is_mouse_pressed {
            self.current_target = best_target;
        }

        best_target
    }

    // Function to simulate mouse press and release
    pub fn on_mouse_pressed(&mut self) {
        self.is_mouse_pressed = true;
    }

    pub fn on_mouse_released(&mut self) {
        self.is_mouse_pressed = false;
        self.current_target = None; // Clear the current target when the mouse is released
    }

    fn aim_at_target(&self, ctx: &UpdateContext, target_screen_position: [f32; 2]) -> anyhow::Result<bool> {
        let view = ctx.states.resolve::<ViewController>(())?;
        let crosshair_pos = [view.screen_bounds.x / 2.0, view.screen_bounds.y / 2.0];
        let aim_adjustment = [
            (target_screen_position[0] - crosshair_pos[0]) / self.aim_speed,
            (target_screen_position[1] - crosshair_pos[1]) / self.aim_speed,
        ];

        ctx.cs2.send_mouse_state(&[MouseState {
            last_x: aim_adjustment[0] as i32,
            last_y: aim_adjustment[1] as i32,
            ..Default::default()
        }])?;
        Ok(true)
    }
}

impl Enhancement for Aimbot {
    fn update(&mut self, ctx: &UpdateContext) -> anyhow::Result<()> {
        let settings = ctx.states.resolve::<AppSettings>(())?;

        // Update the aimbot settings from the configuration
        self.fov = settings.aimbot_fov;         // Fetch FOV from the config
        self.aim_speed = settings.aimbot_speed; // Fetch aim speed from the config
        self.aim_bone = settings.aim_bone.to_lowercase();
        self.aimbot_team_check = settings.aimbot_team_check;

        // Other aimbot logic
        if self.toggle.update(&settings.aimbot_mode, ctx.input, &settings.key_aimbot) {
            ctx.cs2.add_metrics_record(
                obfstr!("feature-aimbot-toggle"),
                &format!("enabled: {}, mode: {:?}", self.toggle.enabled, settings.aimbot_mode),
            );
        }

        if self.toggle.enabled {
            if let Some(target_screen_position) = self.find_best_target(ctx) {
                self.aim_at_target(ctx, target_screen_position)?;
            }
        }

        Ok(())
    }

    fn render(&self, _states: &utils_state::StateRegistry, _ui: &imgui::Ui, _unicode_text: &UnicodeTextRenderer) -> anyhow::Result<()> {
        Ok(())
    }
}
