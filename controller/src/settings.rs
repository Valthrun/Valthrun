use std::{path::PathBuf, fs::File, io::{BufReader, BufWriter}};
use serde::{ Deserialize, Serialize };
use anyhow::Context;


#[derive(Clone, Deserialize, Serialize)]
pub struct AppSettings {
    pub player_list: bool,
    pub player_pos_dot: bool,
    
    pub esp_skeleton: bool,
    pub esp_boxes: bool,

    pub imgui: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self { 
            player_list: false,

            player_pos_dot: false, 
            esp_skeleton: true,
            esp_boxes: false,

            imgui: None,
        }
    }
}

pub fn get_settings_path() -> anyhow::Result<PathBuf> {
    let exe_file = std::env::current_exe()
        .context("missing current exe path")?;
    let base_dir = exe_file.parent()
        .context("could not get exe directory")?;

    Ok(base_dir.join("config.yaml"))
}

pub fn load_app_settings() -> anyhow::Result<AppSettings> {
    let config_path = get_settings_path()?;
    if !config_path.is_file() {
        log::info!("App config file {} does not exist.", config_path.to_string_lossy());
        log::info!("Using default config.");
        return Ok(AppSettings::default());
    }

    let config = File::open(&config_path)
        .with_context(|| format!("failed to open app config at {}", config_path.to_string_lossy()))?;
    let mut config = BufReader::new(config);

    let config: AppSettings = serde_yaml::from_reader(&mut config)
        .context("failed to parse app config")?;

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
        .with_context(|| format!("failed to open app config at {}", config_path.to_string_lossy()))?;
    let mut config = BufWriter::new(config);

    serde_yaml::to_writer(&mut config, settings)
        .context("failed to serialize config")?;

    log::debug!("Saved app config.");
    Ok(())
}