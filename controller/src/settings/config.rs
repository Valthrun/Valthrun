use std::{
    collections::{
        BTreeMap,
        HashMap,
    },
    fs::File,
    io::{
        BufReader,
        BufWriter,
    },
    path::PathBuf,
    sync::atomic::{
        AtomicUsize,
        Ordering,
    },
};

use anyhow::Context;
use imgui::Key;
use serde::{
    Deserialize,
    Serialize,
};
use serde_with::with_prefix;
use utils_state::{
    State,
    StateCacheType,
};

use super::{
    Color,
    EspConfig,
    EspPlayerSettings,
    EspSelector,
    HotKey,
};

fn bool_true() -> bool {
    true
}
fn bool_false() -> bool {
    false
}
fn default_u32<const V: u32>() -> u32 {
    V
}
fn default_i32<const V: i32>() -> i32 {
    V
}
fn default_usize<const V: usize>() -> usize {
    V
}
fn default_f32<const N: usize, const D: usize>() -> f32 {
    N as f32 / D as f32
}
fn default_color<const R: u8, const G: u8, const B: u8, const A: u8>() -> Color {
    Color::from_u8([R, G, B, A])
}

fn default_key_settings() -> HotKey {
    Key::Pause.into()
}
fn default_key_trigger_bot() -> Option<HotKey> {
    Some(Key::MouseMiddle.into())
}
fn default_key_none() -> Option<HotKey> {
    None
}

fn default_esp_mode() -> KeyToggleMode {
    KeyToggleMode::AlwaysOn
}

// Default configuration functions for Aimbot struct fields
fn default_aimbot_mode() -> KeyToggleMode {
    KeyToggleMode::Trigger
}

fn default_aimbot_key() -> Option<HotKey> {
    Some(Key::MouseLeft.into())
}

fn default_aim_bone() -> String {
    "head".to_string() // Default aim bone is "head"
}

fn default_aimbot_fov() -> f32 {
    5.0 // Default FOV value
}

fn default_aimbot_speed() -> f32 {
    150.0 // Default aimbot speed
}

fn default_aimbot_delay_activation() -> u32 {
    500 // Default delay in milliseconds before activation
}

fn default_aimbot_random_aim_path() -> bool {
    true // Default is to enable random aim path
}

fn default_aimbot_adaptive_aim_correction() -> bool {
    true // Default is to enable adaptive aim correction
}

fn default_aimbot_multiple_target_prioritization() -> bool {
    true // Default is to enable multiple target prioritization
}

fn default_aimbot_visibility_check() -> bool {
    true // Default is to enable visibility check
}

fn default_trigger_bot_mode() -> KeyToggleMode {
    KeyToggleMode::Trigger
}

fn default_esp_configs() -> BTreeMap<String, EspConfig> {
    let mut result: BTreeMap<String, EspConfig> = Default::default();
    result.insert(
        "player.enemy".to_string(),
        EspConfig::Player(EspPlayerSettings::new(&EspSelector::PlayerTeam {
            enemy: true,
        })),
    );
    result
}

fn default_esp_configs_enabled() -> BTreeMap<String, bool> {
    let mut result: BTreeMap<String, bool> = Default::default();
    result.insert("player.enemy".to_string(), true);
    result
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, PartialOrd)]
pub enum KeyToggleMode {
    AlwaysOn,
    Toggle,
    Trigger,
    TriggerInverted,
    Off,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum GrenadeType {
    Smoke,
    Molotov,
    Flashbang,
    Explosive,
}

impl GrenadeType {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Smoke => "Smoke",
            Self::Molotov => "Molotov",
            Self::Flashbang => "Flashbang",
            Self::Explosive => "Explosive",
        }
    }
}

static GRENADE_SPOT_ID_INDEX: AtomicUsize = AtomicUsize::new(1);
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct GrenadeSpotInfo {
    #[serde(skip, default = "GrenadeSpotInfo::new_id")]
    pub id: usize,
    pub grenade_types: Vec<GrenadeType>,

    pub name: String,
    pub description: String,

    /// The eye position of the player
    pub eye_position: [f32; 3],
    pub eye_direction: [f32; 2],
}

impl GrenadeSpotInfo {
    pub fn new_id() -> usize {
        GRENADE_SPOT_ID_INDEX.fetch_add(1, Ordering::Relaxed)
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct GrenadeSettings {
    #[serde(default = "bool_true")]
    pub active: bool,

    #[serde(default = "default_f32::<150, 1>")]
    pub circle_distance: f32,

    #[serde(default = "default_f32::<20, 1>")]
    pub circle_radius: f32,

    #[serde(default = "default_usize::<32>")]
    pub circle_segments: usize,

    #[serde(default = "default_f32::<1, 10>")]
    pub angle_threshold_yaw: f32,

    #[serde(default = "default_f32::<5, 10>")]
    pub angle_threshold_pitch: f32,

    #[serde(default = "default_color::<255, 255, 255, 255>")]
    pub color_position: Color,

    #[serde(default = "default_color::<0, 255, 0, 255>")]
    pub color_position_active: Color,

    #[serde(default = "default_color::<255, 0, 0, 255>")]
    pub color_angle: Color,

    #[serde(default = "default_color::<0, 255, 0, 255>")]
    pub color_angle_active: Color,

    #[serde(default)]
    pub map_spots: HashMap<String, Vec<GrenadeSpotInfo>>,
}
with_prefix!(serde_prefix_grenade_helper "grenade_helper");

#[derive(Clone, Deserialize, Serialize)]
pub struct AppSettings {
    #[serde(default = "default_key_settings")]
    pub key_settings: HotKey,

    #[serde(default = "default_esp_mode")]
    pub esp_mode: KeyToggleMode,

    #[serde(default = "default_key_none")]
    pub esp_toogle: Option<HotKey>,

    #[serde(default = "default_esp_configs")]
    pub esp_settings: BTreeMap<String, EspConfig>,

    #[serde(default = "default_esp_configs_enabled")]
    pub esp_settings_enabled: BTreeMap<String, bool>,

    #[serde(default = "bool_true")]
    pub bomb_timer: bool,

    #[serde(default = "bool_false")]
    pub spectators_list: bool,

    #[serde(default = "bool_true")]
    pub valthrun_watermark: bool,

    #[serde(default = "default_i32::<16364>")]
    pub mouse_x_360: i32,

    #[serde(default = "default_trigger_bot_mode")]
    pub trigger_bot_mode: KeyToggleMode,

    #[serde(default = "default_key_trigger_bot")]
    pub key_trigger_bot: Option<HotKey>,

    #[serde(default = "default_aimbot_mode")]
    pub aimbot_mode: KeyToggleMode, // Key toggle for enabling/disabling the aimbot

    #[serde(default = "default_aimbot_key")]
    pub aimbot_key: Option<HotKey>,

    #[serde(default = "default_f32::<5, 1>")]
    pub aimbot_fov: f32, // Field of view for target acquisition

    #[serde(default = "default_f32::<5, 1>")]
    pub aimbot_smooth: f32, // Speed at which the aim moves

    #[serde(default = "bool_false")]
    pub aimbot_is_active: bool, // Indicates if the aimbot is currently active

    #[serde(default)]
    pub aimbot_current_target: Option<[f32; 3]>, // Current target coordinates (x, y, z)

    #[serde(default = "bool_false")]
    pub aimbot_is_mouse_pressed: bool, // Indicates if the mouse button is being pressed

    #[serde(default = "default_aim_bone")]
    pub aimbot_aim_bone: String, // The specific bone being targeted (e.g., "head", "chest")

    // Advanced properties
    #[serde(default = "bool_true")]
    pub aimbot_team_check: bool, // Checks if the target is on the same team

    #[serde(default = "bool_true")]
    pub aimbot_view_fov: bool, // Only targets within the aimbot's field of view

    #[serde(default = "default_f32::<5, 1>")]
    pub aimbot_flash_alpha: f32, // Threshold to ignore targets affected by flashbangs

    #[serde(default = "bool_false")]
    pub aimbot_ignore_flash: bool, // Flag to enable/disable ignoring flash effects

    #[serde(default = "bool_true")]
    pub aimbot_visibility_check: bool, // Ensures the target is visible (not behind obstacles)

    #[serde(default = "bool_true")]
    pub trigger_bot_team_check: bool,

    #[serde(default = "default_u32::<10>")]
    pub trigger_bot_delay_min: u32,

    #[serde(default = "default_u32::<20>")]
    pub trigger_bot_delay_max: u32,

    #[serde(default = "default_u32::<400>")]
    pub trigger_bot_shot_duration: u32,

    #[serde(default = "bool_false")]
    pub trigger_bot_check_target_after_delay: bool,

    #[serde(default = "bool_false")]
    pub aim_assist_recoil: bool,

    #[serde(default = "default_u32::<1>")]
    pub aim_assist_recoil_min_bullets: u32,

    #[serde(default = "bool_true")]
    pub hide_overlay_from_screen_capture: bool,

    #[serde(default = "bool_false")]
    pub render_debug_window: bool,

    #[serde(default = "bool_true")]
    pub metrics: bool,

    #[serde(default)]
    pub web_radar_url: Option<String>,

    #[serde(default = "bool_false")]
    pub web_radar_advanced_settings: bool,

    #[serde(flatten, with = "serde_prefix_grenade_helper")]
    pub grenade_helper: GrenadeSettings,

    #[serde(default)]
    pub imgui: Option<String>,
}

impl State for AppSettings {
    type Parameter = ();

    fn cache_type() -> StateCacheType {
        StateCacheType::Persistent
    }
}

pub fn get_settings_path() -> anyhow::Result<PathBuf> {
    let exe_file = std::env::current_exe().context("missing current exe path")?;
    let base_dir = exe_file.parent().context("could not get exe directory")?;

    Ok(base_dir.join("config.yaml"))
}

pub fn load_app_settings() -> anyhow::Result<AppSettings> {
    let config_path = get_settings_path()?;
    if !config_path.is_file() {
        log::info!(
            "App config file {} does not exist.",
            config_path.to_string_lossy()
        );
        log::info!("Using default config.");
        let config: AppSettings =
            serde_yaml::from_str("").context("failed to parse empty config")?;

        return Ok(config);
    }

    let config = File::open(&config_path).with_context(|| {
        format!(
            "failed to open app config at {}",
            config_path.to_string_lossy()
        )
    })?;
    let mut config = BufReader::new(config);

    let config: AppSettings =
        serde_yaml::from_reader(&mut config).context("failed to parse app config")?;

    log::info!("Loaded app config from {}", config_path.to_string_lossy());
    Ok(config)
}

pub fn save_app_settings(settings: &AppSettings) -> anyhow::Result<()> {
    let config_path = get_settings_path()?;
    let config = File::options()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&config_path)
        .with_context(|| {
            format!(
                "failed to open app config at {}",
                config_path.to_string_lossy()
            )
        })?;
    let mut config = BufWriter::new(config);

    serde_yaml::to_writer(&mut config, settings).context("failed to serialize config")?;

    log::debug!("Saved app config.");
    Ok(())
}
