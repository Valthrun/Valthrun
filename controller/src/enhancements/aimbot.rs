use super::Enhancement;
use crate::{
    settings::AppSettings,
    view::{
        KeyToggle,
        ViewController,
    },
    UnicodeTextRenderer,
    UpdateContext,
};
use cs2::{
    BoneFlags,
    CEntityIdentityEx,
    CS2Model,
    ClassNameCache,
    EntitySystem,
    PlayerPawnState,
};
use cs2_schema_generated::{
    cs2::client::C_CSPlayerPawn,
    EntityHandle,
};
use nalgebra::Vector3;
use obfstr::obfstr;
use std::time::Instant;
use valthrun_kernel_interface::MouseState;
pub struct Aimbot {
    toggle: KeyToggle,
    fov: f32, // FOV fetched from settings
    aim_speed: f32, // Aim speed fetched from settings
    is_active: bool,
    last_mouse_move: Instant, // Track the time for constant mouse down movement
    current_target: Option<[f32; 2]>, // Store the current target
    is_mouse_pressed: bool, // Track if the mouse is pressed
    aim_bone: String,
    aimbot_team_check: bool,
    aimbot_view_fov: bool,
}

impl Aimbot {
    pub fn new() -> Self {
        Self {
            toggle: KeyToggle::new(),
            fov: 3.0, // Default FOV, updated dynamically from settings
            aim_speed: 2.5, // Default aim speed, updated dynamically from settings
            is_active: false,
            last_mouse_move: Instant::now(), // Initialize the timer
            current_target: None,
            is_mouse_pressed: false,
            aim_bone: "head".to_string(),
            aimbot_team_check: true,
            aimbot_view_fov: true,
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

        let local_pawn_index = entities.get_local_player_controller().ok()?.reference_schema().ok()?.m_hPlayerPawn().ok()?.value;
        let local_pawn = match entities.get_by_handle::<C_CSPlayerPawn>(&EntityHandle::from_index(local_pawn_index)).ok()? {
            Some(identity) => identity.entity().ok()?.read_schema().ok()?,
            None => return None
        };

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
                    if local_pawn.m_flFlashBangTime().unwrap() > 0.0 { // ignore flash
                        continue;
                    } //player in gui check// respect wall check//team_mate check
                    if settings.aimbot_team_check && local_pawn.m_iTeamNum().unwrap() == player_info.team_id {
                        continue;
                    } // Calculate distance between the local player and the target
                    let distance = (player_info.position - local_player_position).norm() * UNITS_TO_METERS; // Skip self (local player) if distance is < 3.0
                    if distance < 2.0 {
                        continue;
                    }

                    for (bone, state) in entry_model.bones.iter().zip(player_info.bone_states.iter()) {
                        if (bone.flags & BoneFlags::FlagHitbox as u32) == 0 {
                            continue;
                        }

                        if bone.name.to_lowercase().contains(&self.aim_bone) { // Convert the selected bone position to screen space
                            if let Some(screen_position) = self.world_to_screen(&view, &state.position) { // Calculate the distance from the crosshair (X and Y axes)
                                let dx = screen_position[0] - crosshair_pos[0];
                                let dy = screen_position[1] - crosshair_pos[1];
                                let distance_from_crosshair = (dx * dx + dy * dy).sqrt(); // Calculate the angle between the crosshair and the target
                                let angle_to_target = distance_from_crosshair.atan2(view.screen_bounds.x / 2.0).to_degrees(); // Check if the target is within the FOV
                                if angle_to_target <= self.fov / 2.0 { // If the bone is closer than the previous best target, update the best target
                                    if distance_from_crosshair < lowest_distance_from_crosshair {
                                        lowest_distance_from_crosshair = distance_from_crosshair;
                                        best_target = Some(screen_position); // Update the target to the selected bone
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } // Update current target when the mouse is pressed and a new best target is found
        if self.is_mouse_pressed {
            self.current_target = best_target;
        }

        best_target
    } // Function to simulate mouse press and release
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
        let settings = ctx.states.resolve::<AppSettings>(())?; // Update the aimbot settings from the configuration
        self.fov = settings.aimbot_fov; // Fetch FOV from the config
        self.aim_speed = settings.aimbot_speed; // Fetch aim speed from the config
        self.aim_bone = settings.aim_bone.to_lowercase();
        self.aimbot_team_check = settings.aimbot_team_check; // Other aimbot logic
        if self.toggle.update(&settings.aimbot_mode, ctx.input, &settings.key_aimbot) {
            ctx.cs2.add_metrics_record(
                obfstr!("feature-aimbot-toggle"),
                &format!("enabled: {}, mode: {:?}", self.toggle.enabled, settings.aimbot_mode),
            );
        } else if self.toggle.update_dual(
            &settings.aimbot_mode,
            ctx.input,
            &settings.key_aimbot,
            &settings.key_aimbot_secondary,
        ) {
            ctx.cs2.add_metrics_record(
                obfstr!("feature-aimbot-toggle"),
                &format!("enabled: {}, mode: {:?}", self.toggle.enabled, settings.aimbot_mode),
            )
        }

        if self.toggle.enabled {
            if let Some(target_screen_position) = self.find_best_target(ctx) {
                self.aim_at_target(ctx, target_screen_position)?;
            }
        }

        Ok(())
    }

    fn render(&self, _states: &utils_state::StateRegistry, _ui: &imgui::Ui, _unicode_text: &UnicodeTextRenderer) -> anyhow::Result<()> {
        let settings = _states.resolve::<AppSettings>(())?;
        let view = _states.resolve::<ViewController>(())?;
        let draw_list = _ui.get_window_draw_list();
        let cursor_pos = [view.screen_bounds.x / 2.0, view.screen_bounds.y / 2.0];
        fn fov_to_radius(fov: f32, screen_width: f32) -> f32 {
            // Calculate the radius of the FOV circle based on the FOV and screen width
            let fov_in_radians = fov.to_radians(); // Convert to radians
            let half_fov = fov_in_radians / 2.0; // Half FOV angle
            let radius = (screen_width / 2.0) * (half_fov.tan()); // Calculate radius based on half FOV
            radius
        }

        // assume that fov is enabled
        if settings.aimbot_view_fov {
            draw_list
                .add_circle(
                    cursor_pos,
                    fov_to_radius(settings.aimbot_fov, view.screen_bounds.x),
                    (1.0, 1.0, 1.0, 1.0))
                .filled(false)
                .build();
        }
        Ok(())
    }
}
