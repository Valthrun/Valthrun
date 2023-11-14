#![allow(dead_code)]
#![feature(const_fn_floating_point_arithmetic)]

use std::{
    cell::{
        RefCell,
        RefMut,
    },
    error::Error,
    fmt::Debug,
    fs::File,
    io::BufWriter,
    path::PathBuf,
    rc::Rc,
    sync::{
        atomic::{
            AtomicBool,
            Ordering,
        },
        Arc,
    },
    time::{
        Duration,
        Instant,
    },
};

use anyhow::Context;
use cache::EntryCache;
use clap::{
    Args,
    Parser,
    Subcommand,
};
use class_name_cache::ClassNameCache;
use cs2::{
    BuildInfo,
    CS2Handle,
    CS2Model,
    CS2Offsets,
    EntitySystem,
    Globals,
};
use cs2_schema_generated::{
    definition::SchemaScope,
    RuntimeOffset,
    RuntimeOffsetProvider,
};
use enhancements::Enhancement;
use imgui::{
    Condition,
    FontConfig,
    FontId,
    FontSource,
    Ui,
};
use map::{
    get_current_map,
    MapInfo,
};
use obfstr::obfstr;
use overlay::{
    LoadingError,
    OverlayError,
    OverlayOptions,
    OverlayTarget,
    SystemRuntimeController,
};
use settings::{
    load_app_settings,
    AppSettings,
    SettingsUI,
};
use valthrun_kernel_interface::KInterfaceError;
use view::ViewController;
use windows::Win32::{
    System::Console::GetConsoleProcessList,
    UI::Shell::IsUserAnAdmin,
};

use crate::{
    enhancements::{
        AntiAimPunsh,
        BombInfo,
        PlayerESP,
        SpectatorsList,
        TriggerBot,
        WebRadar,
    },
    offsets::setup_runtime_offset_provider,
    settings::save_app_settings,
    view::LocalCrosshair,
    web_radar_server::{
        MessageData,
        CLIENTS,
    },
    winver::version_info,
};

mod cache;
mod class_name_cache;
mod enhancements;
mod map;
mod offsets;
mod settings;
mod utils;
mod view;
mod weapon;
mod web_radar_server;
mod winver;

pub trait MetricsClient {
    fn add_metrics_record(&self, record_type: &str, record_payload: &str);
}

impl MetricsClient for CS2Handle {
    fn add_metrics_record(&self, record_type: &str, record_payload: &str) {
        self.add_metrics_record(record_type, record_payload)
    }
}

pub trait KeyboardInput {
    fn is_key_down(&self, key: imgui::Key) -> bool;
    fn is_key_pressed(&self, key: imgui::Key, repeating: bool) -> bool;
}

impl KeyboardInput for imgui::Ui {
    fn is_key_down(&self, key: imgui::Key) -> bool {
        Ui::is_key_down(self, key)
    }

    fn is_key_pressed(&self, key: imgui::Key, repeating: bool) -> bool {
        if repeating {
            Ui::is_key_pressed(self, key)
        } else {
            Ui::is_key_pressed_no_repeat(self, key)
        }
    }
}

pub struct UpdateContext<'a> {
    pub settings: &'a AppSettings,
    pub input: &'a dyn KeyboardInput,

    pub current_map: &'a Option<MapInfo>,
    pub current_map_changed: &'a bool,

    pub cs2: &'a Arc<CS2Handle>,
    pub cs2_entities: &'a EntitySystem,

    pub model_cache: &'a EntryCache<u64, CS2Model>,
    pub class_name_cache: &'a ClassNameCache,
    pub view_controller: &'a ViewController,

    pub globals: Globals,
}

pub struct AppFonts {
    valthrun: FontId,
}

pub struct Application {
    pub fonts: AppFonts,

    pub cs2: Arc<CS2Handle>,
    pub cs2_offsets: Arc<CS2Offsets>,
    pub cs2_entities: EntitySystem,
    pub cs2_globals: Option<Globals>,
    pub cs2_build_info: BuildInfo,

    pub current_map: Option<MapInfo>,
    pub current_map_changed: bool,

    pub model_cache: EntryCache<u64, CS2Model>,
    pub class_name_cache: ClassNameCache,
    pub view_controller: ViewController,

    pub enhancements: Vec<Rc<RefCell<dyn Enhancement>>>,

    pub frame_read_calls: usize,
    pub last_total_read_calls: usize,

    pub settings: Rc<RefCell<AppSettings>>,
    pub settings_visible: bool,
    pub settings_dirty: bool,
    pub settings_ui: RefCell<SettingsUI>,
    pub settings_screen_capture_changed: AtomicBool,
    pub settings_render_debug_window_changed: AtomicBool,
}

impl Application {
    pub fn settings(&self) -> std::cell::Ref<'_, AppSettings> {
        self.settings.borrow()
    }

    pub fn settings_mut(&self) -> RefMut<'_, AppSettings> {
        self.settings.borrow_mut()
    }

    pub fn pre_update(&mut self, controller: &mut SystemRuntimeController) -> anyhow::Result<()> {
        if self.settings_dirty {
            self.settings_dirty = false;
            let mut settings = self.settings.borrow_mut();

            settings.imgui = None;
            if let Ok(value) = serde_json::to_string(&*settings) {
                self.cs2.add_metrics_record("settings-updated", &value);
            }

            let mut imgui_settings = String::new();
            controller.imgui.save_ini_settings(&mut imgui_settings);
            settings.imgui = Some(imgui_settings);

            if let Err(error) = save_app_settings(&*settings) {
                log::warn!("Failed to save user settings: {}", error);
            };
        }

        if self
            .settings_screen_capture_changed
            .swap(false, Ordering::Relaxed)
        {
            let settings = self.settings.borrow();
            controller.toggle_screen_capture_visibility(!settings.hide_overlay_from_screen_capture);
            log::debug!(
                "Updating screen capture visibility to {}",
                !settings.hide_overlay_from_screen_capture
            );
        }

        if self
            .settings_render_debug_window_changed
            .swap(false, Ordering::Relaxed)
        {
            let settings = self.settings.borrow();
            controller.toggle_debug_overlay(settings.render_debug_window);
        }

        Ok(())
    }

    pub fn update(&mut self, ui: &imgui::Ui) -> anyhow::Result<()> {
        {
            let mut settings = self.settings.borrow_mut();
            for enhancement in self.enhancements.iter() {
                let mut hack = enhancement.borrow_mut();
                if hack.update_settings(ui, &mut *settings)? {
                    self.settings_dirty = true;
                }
            }
        }

        let settings = self.settings.borrow();
        if ui.is_key_pressed_no_repeat(settings.key_settings.0) {
            log::debug!("Toogle settings");
            self.settings_visible = !self.settings_visible;
            self.cs2.add_metrics_record(
                "settings-toggled",
                &format!("visible: {}", self.settings_visible),
            );

            if !self.settings_visible {
                /* overlay has just been closed */
                self.settings_dirty = true;
            }
        }

        self.view_controller
            .update_screen_bounds(mint::Vector2::from_slice(&ui.io().display_size));
        self.view_controller.update_view_matrix(&self.cs2)?;

        let globals = self
            .cs2
            .reference_schema::<Globals>(&[self.cs2_offsets.globals, 0])?
            .cached()
            .with_context(|| obfstr!("failed to read globals").to_string())?;

        let new_map_info =
            get_current_map(&self.cs2, self.cs2_offsets.network_game_client_instance)?;

        if let Some(new_map) = &new_map_info {
            self.current_map_changed = self.current_map != new_map_info;
            if self.current_map_changed {
                let mut data = web_radar_server::CURRENT_MAP.write().unwrap();
                *data = new_map.clone();
                match serde_json::to_string(new_map) {
                    Ok(data) => {
                        for client in CLIENTS.lock().unwrap().iter() {
                            client.do_send(MessageData { data: data.clone() });
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to create json with error: {}", e);
                    }
                };
                self.current_map = new_map_info;
            }
        };

        self.cs2_entities
            .read_entities()
            .with_context(|| obfstr!("failed to read global entity list").to_string())?;

        self.class_name_cache
            .update_cache(self.cs2_entities.all_identities())
            .with_context(|| obfstr!("failed to update class name cache").to_string())?;

        let update_context = UpdateContext {
            cs2: &self.cs2,
            cs2_entities: &self.cs2_entities,

            current_map: &self.current_map,
            current_map_changed: &self.current_map_changed,

            settings: &*settings,
            input: ui,

            globals,
            class_name_cache: &self.class_name_cache,
            view_controller: &self.view_controller,
            model_cache: &self.model_cache,
        };

        for enhancement in self.enhancements.iter() {
            let mut hack = enhancement.borrow_mut();
            hack.update(&update_context)?;
        }

        let read_calls = self.cs2.ke_interface.total_read_calls();
        self.frame_read_calls = read_calls - self.last_total_read_calls;
        self.last_total_read_calls = read_calls;

        Ok(())
    }

    pub fn render(&self, ui: &imgui::Ui) {
        ui.window("overlay")
            .draw_background(false)
            .no_decoration()
            .no_inputs()
            .size(ui.io().display_size, Condition::Always)
            .position([0.0, 0.0], Condition::Always)
            .build(|| self.render_overlay(ui));

        {
            let mut settings = self.settings.borrow_mut();
            for enhancement in self.enhancements.iter() {
                let mut enhancement = enhancement.borrow_mut();
                enhancement.render_debug_window(&mut *settings, ui);
            }
        }

        if self.settings_visible {
            let mut settings_ui = self.settings_ui.borrow_mut();
            settings_ui.render(self, ui)
        }
    }

    fn render_overlay(&self, ui: &imgui::Ui) {
        let settings = self.settings.borrow();

        if settings.valthrun_watermark {
            {
                let text_buf;
                let text = obfstr!(text_buf = "Valthrun Overlay");

                ui.set_cursor_pos([
                    ui.window_size()[0] - ui.calc_text_size(text)[0] - 10.0,
                    10.0,
                ]);
                ui.text(text);
            }
            {
                let text = format!("{:.2} FPS", ui.io().framerate);
                ui.set_cursor_pos([
                    ui.window_size()[0] - ui.calc_text_size(&text)[0] - 10.0,
                    24.0,
                ]);
                ui.text(text)
            }
            {
                let text = format!("{} Reads", self.frame_read_calls);
                ui.set_cursor_pos([
                    ui.window_size()[0] - ui.calc_text_size(&text)[0] - 10.0,
                    38.0,
                ]);
                ui.text(text)
            }
        }

        for hack in self.enhancements.iter() {
            let hack = hack.borrow();
            hack.render(&*settings, ui, &self.view_controller);
        }
    }
}

fn show_critical_error(message: &str) {
    for line in message.lines() {
        log::error!("{}", line);
    }

    if !is_console_invoked() {
        overlay::show_error_message(obfstr!("Valthrun Controller"), message);
    }
}

#[actix_web::main]
async fn main() {
    let args = match AppArgs::try_parse() {
        Ok(args) => args,
        Err(error) => {
            println!("{:#}", error);
            std::process::exit(1);
        }
    };

    env_logger::builder()
        .filter_level(if args.verbose {
            log::LevelFilter::Trace
        } else {
            log::LevelFilter::Info
        })
        .parse_default_env()
        .init();

    let command = args.command.as_ref().unwrap_or(&AppCommand::Overlay);
    let result = match command {
        AppCommand::DumpSchema(args) => main_schema_dump(args),
        AppCommand::Overlay => main_overlay().await,
    };

    if let Err(error) = result {
        show_critical_error(&format!("{:#}", error));
    }
}

#[derive(Debug, Parser)]
#[clap(name = "Valthrun", version)]
struct AppArgs {
    /// Enable verbose logging ($env:RUST_LOG="trace")
    #[clap(short, long)]
    verbose: bool,

    #[clap(subcommand)]
    command: Option<AppCommand>,
}

#[derive(Debug, Subcommand)]
enum AppCommand {
    /// Start the overlay
    Overlay,

    /// Create a schema dump
    DumpSchema(SchemaDumpArgs),
}

#[derive(Debug, Args)]
struct SchemaDumpArgs {
    pub target_file: PathBuf,
}

fn is_console_invoked() -> bool {
    let console_count = unsafe {
        let mut result = [0u32; 128];
        GetConsoleProcessList(&mut result)
    };

    console_count > 1
}

fn main_schema_dump(args: &SchemaDumpArgs) -> anyhow::Result<()> {
    log::info!("Dumping schema. Please wait...");

    let cs2 = CS2Handle::create(true)?;
    let schema = cs2::dump_schema(&cs2, false)?;

    let output = File::options()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&args.target_file)?;

    let mut output = BufWriter::new(output);
    serde_json::to_writer_pretty(&mut output, &schema)?;
    log::info!("Schema dumped to {}", args.target_file.to_string_lossy());
    Ok(())
}

async fn main_overlay() -> anyhow::Result<()> {
    let build_info = version_info()?;
    log::info!(
        "{} v{} ({}). Windows build {}.",
        obfstr!("Valthrun"),
        env!("CARGO_PKG_VERSION"),
        env!("GIT_HASH"),
        build_info.dwBuildNumber
    );
    log::info!(
        "{} {}",
        obfstr!("Current executable was built on"),
        env!("BUILD_TIME")
    );

    if unsafe { IsUserAnAdmin().as_bool() } {
        log::warn!("{}", obfstr!("Please do not run this as administrator!"));
        log::warn!("{}", obfstr!("Running the controller as administrator might cause failures with your graphic drivers."));
    }

    let settings = load_app_settings()?;
    let cs2 = match CS2Handle::create(settings.metrics) {
        Ok(handle) => handle,
        Err(err) => {
            if let Some(err) = err.downcast_ref::<KInterfaceError>() {
                if let KInterfaceError::DeviceUnavailable(error) = &err {
                    if error.code().0 as u32 == 0x80070002 {
                        /* The system cannot find the file specified. */
                        show_critical_error(obfstr!("** PLEASE READ CAREFULLY **\nCould not find the kernel driver interface.\nEnsure you have successfully loaded/mapped the kernel driver (valthrun-driver.sys) before starting the CS2 controller.\nPlease explicitly check the driver entry status code which should be 0x0.\n\nFor more help, checkout:\nhttps://wiki.valth.run/#/030_troubleshooting/overlay/020_driver_has_not_been_loaded."));
                        return Ok(());
                    }
                } else if let KInterfaceError::DriverTooOld {
                    driver_version_string,
                    requested_version_string,
                    ..
                } = &err
                {
                    let message = obfstr!(
                        "\nThe installed/loaded Valthrun driver version is too old.\nPlease ensure you installed/mapped the latest Valthrun driver.\nATTENTION: If you have manually mapped the driver, you have to restart your PC in order to load the new version."
                    ).to_string();

                    show_critical_error(&format!(
                        "{}\n\nLoaded driver version: {}\nRequired driver version: {}",
                        message, driver_version_string, requested_version_string
                    ));
                    return Ok(());
                } else if let KInterfaceError::DriverTooNew {
                    driver_version_string,
                    requested_version_string,
                    ..
                } = &err
                {
                    let message = obfstr!(
                        "\nThe installed/loaded Valthrun driver version is too new.\nPlease ensure you're using the lattest controller."
                    ).to_string();

                    show_critical_error(&format!(
                        "{}\n\nLoaded driver version: {}\nRequired driver version: {}",
                        message, driver_version_string, requested_version_string
                    ));
                    return Ok(());
                } else if let KInterfaceError::ProcessDoesNotExists = &err {
                    show_critical_error(obfstr!("Could not find CS2 process.\nPlease start CS2 prior to executing this application!"));
                    return Ok(());
                }
            }

            return Err(err);
        }
    };

    cs2.add_metrics_record(obfstr!("controller-status"), "initializing");

    let cs2_build_info = BuildInfo::read_build_info(&cs2).with_context(|| {
        obfstr!("Failed to load CS2 build info. CS2 version might be newer / older then expected")
            .to_string()
    })?;
    log::info!(
        "Found {}. Revision {} from {}.",
        obfstr!("Counter-Strike 2"),
        cs2_build_info.revision,
        cs2_build_info.build_datetime
    );
    cs2.add_metrics_record(
        obfstr!("cs2-version"),
        &format!("revision: {}", cs2_build_info.revision),
    );

    let cs2_offsets = Arc::new(
        CS2Offsets::resolve_offsets(&cs2)
            .with_context(|| obfstr!("failed to load CS2 offsets").to_string())?,
    );

    setup_runtime_offset_provider(&cs2)?;

    let imgui_settings = settings.imgui.clone();
    let settings = Rc::new(RefCell::new(settings));

    log::debug!("Initialize overlay");
    let app_fonts: Rc<RefCell<Option<AppFonts>>> = Default::default();
    let overlay_options = OverlayOptions {
        title: obfstr!("CS2 Overlay").to_string(),
        target: OverlayTarget::WindowOfProcess(cs2.process_id() as u32),
        font_init: Some(Box::new({
            let app_fonts = app_fonts.clone();

            move |imgui| {
                let mut app_fonts = app_fonts.borrow_mut();

                let font_size = 18.0;
                let valthrun_font = imgui.fonts().add_font(&[FontSource::TtfData {
                    data: include_bytes!("../resources/Valthrun-Regular.ttf"),
                    size_pixels: font_size,
                    config: Some(FontConfig {
                        rasterizer_multiply: 1.5,
                        oversample_h: 4,
                        oversample_v: 4,
                        ..FontConfig::default()
                    }),
                }]);

                *app_fonts = Some(AppFonts {
                    valthrun: valthrun_font,
                });
            }
        })),
    };

    let mut overlay = match overlay::init(&overlay_options) {
        Err(OverlayError::VulkanDllNotFound(LoadingError::LibraryLoadFailure(source))) => {
            match &source {
                libloading::Error::LoadLibraryExW { .. } => {
                    let error = source.source().context("LoadLibraryExW to have a source")?;
                    let message = format!("Failed to load vulkan-1.dll.\nError: {:#}", error);
                    show_critical_error(&message);
                }
                error => {
                    let message = format!(
                        "An error occurred while loading vulkan-1.dll.\nError: {:#}",
                        error
                    );
                    show_critical_error(&message);
                }
            }
            return Ok(());
        }
        value => value?,
    };
    if let Some(imgui_settings) = imgui_settings {
        overlay.imgui.load_ini_settings(&imgui_settings);
    }

    let app = Application {
        fonts: app_fonts
            .borrow_mut()
            .take()
            .context("failed to initialize app fonts")?,

        cs2: cs2.clone(),
        cs2_entities: EntitySystem::new(cs2.clone(), cs2_offsets.clone()),
        cs2_offsets: cs2_offsets.clone(),
        cs2_globals: None,
        cs2_build_info,

        current_map: None,
        current_map_changed: false,

        model_cache: EntryCache::new({
            let cs2 = cs2.clone();
            move |model| {
                let model_name = cs2.read_string(&[*model as u64 + 0x08, 0], Some(32))?;
                log::debug!(
                    "{} {} at {:X}. Caching.",
                    obfstr!("Discovered new player model"),
                    model_name,
                    model
                );

                Ok(CS2Model::read(&cs2, *model as u64)?)
            }
        }),
        class_name_cache: ClassNameCache::new(cs2.clone()),
        view_controller: ViewController::new(cs2_offsets.clone()),

        enhancements: vec![
            Rc::new(RefCell::new(PlayerESP::new())),
            Rc::new(RefCell::new(SpectatorsList::new())),
            Rc::new(RefCell::new(BombInfo::new())),
            Rc::new(RefCell::new(TriggerBot::new(LocalCrosshair::new(
                cs2_offsets.offset_crosshair_id,
            )))),
            Rc::new(RefCell::new(WebRadar::new())),
            Rc::new(RefCell::new(AntiAimPunsh::new())),
        ],

        last_total_read_calls: 0,
        frame_read_calls: 0,

        settings: settings.clone(),
        settings_visible: false,
        settings_dirty: false,
        settings_ui: RefCell::new(SettingsUI::new(settings)),
        /* set the screen capture visibility at the beginning of the first update */
        settings_screen_capture_changed: AtomicBool::new(true),
        settings_render_debug_window_changed: AtomicBool::new(true),
    };
    let app = Rc::new(RefCell::new(app));

    cs2.add_metrics_record(
        obfstr!("controller-status"),
        &format!(
            "initialized, version: {}, git-hash: {}, win-build: {}",
            env!("CARGO_PKG_VERSION"),
            env!("GIT_HASH"),
            build_info.dwBuildNumber
        ),
    );

    std::thread::spawn(|| {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(web_radar_server::run_server())
    });

    log::info!("{}", obfstr!("App initialized. Spawning overlay."));
    let mut update_fail_count = 0;
    let mut update_timeout: Option<(Instant, Duration)> = None;
    overlay.main_loop(
        {
            let app = app.clone();
            move |controller| {
                let mut app = app.borrow_mut();
                if let Err(err) = app.pre_update(controller) {
                    show_critical_error(&format!("{:#}", err));
                    false
                } else {
                    true
                }
            }
        },
        move |ui| {
            let mut app = app.borrow_mut();

            if let Some((timeout, target)) = &update_timeout {
                if timeout.elapsed() > *target {
                    update_timeout = None;
                } else {
                    /* Not updating. On timeout... */
                    return true;
                }
            }

            if let Err(err) = app.update(ui) {
                if update_fail_count >= 10 {
                    log::error!("Over 10 errors occurred. Waiting 1s and try again.");
                    log::error!("Last error: {:#}", err);

                    update_timeout = Some((Instant::now(), Duration::from_millis(1000)));
                    update_fail_count = 0;
                    return true;
                } else {
                    update_fail_count += 1;
                }
            }

            app.render(ui);
            true
        },
    )
}
