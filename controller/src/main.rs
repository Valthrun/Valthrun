#![feature(iterator_try_collect)]
#![feature(result_option_inspect)]
#![allow(dead_code)]

use anyhow::Context;
use cache::EntryCache;
use clap::{Args, Parser, Subcommand};
use cs2::{
    CS2Handle, CS2Model, CS2Offsets, EngineBuildInfo, EntitySystem, Globals, Module, PCStrEx,
    Signature,
};
use cs2_schema_declaration::Ptr;
use enhancements::Enhancement;
use imgui::{Condition, Ui};
use obfstr::obfstr;
use overlay::{LoadingError, OverlayError, SystemRuntimeController};
use settings::{load_app_settings, AppSettings};
use settings_ui::SettingsUI;
use std::{
    cell::{RefCell, RefMut},
    error::Error,
    fmt::Debug,
    fs::File,
    io::BufWriter,
    path::PathBuf,
    rc::Rc,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use valthrun_kernel_interface::KInterfaceError;
use view::ViewController;
use windows::Win32::{System::Console::GetConsoleProcessList, UI::Shell::IsUserAnAdmin};

use crate::{
    enhancements::{AntiAimPunsh, BombInfo, PlayerESP, TriggerBot},
    settings::save_app_settings,
    view::LocalCrosshair,
};

mod cache;
mod enhancements;
mod settings;
mod settings_ui;
mod view;

pub trait UpdateInputState {
    fn is_key_down(&self, key: imgui::Key) -> bool;
    fn is_key_pressed(&self, key: imgui::Key, repeating: bool) -> bool;
}

impl UpdateInputState for imgui::Ui {
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
    pub input: &'a dyn UpdateInputState,

    pub cs2: &'a Arc<CS2Handle>,
    pub cs2_entities: &'a EntitySystem,

    pub model_cache: &'a EntryCache<u64, CS2Model>,
    pub class_name_cache: &'a EntryCache<Ptr<()>, Option<String>>,
    pub view_controller: &'a ViewController,

    pub globals: Globals,
}

pub struct Application {
    pub cs2: Arc<CS2Handle>,
    pub cs2_offsets: Arc<CS2Offsets>,
    pub cs2_entities: EntitySystem,
    pub cs2_globals: Option<Globals>,
    pub cs2_build_info: BuildInfo,

    pub model_cache: EntryCache<u64, CS2Model>,
    pub class_name_cache: EntryCache<Ptr<()>, Option<String>>,
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

        let update_context = UpdateContext {
            cs2: &self.cs2,
            cs2_entities: &self.cs2_entities,

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

        if self.settings_visible {
            let mut settings_ui = self.settings_ui.borrow_mut();
            settings_ui.render(self, ui)
        }
    }

    fn render_overlay(&self, ui: &imgui::Ui) {
        let settings = self.settings.borrow();

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

fn main() {
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
        AppCommand::Overlay => main_overlay(),
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

    let cs2 = CS2Handle::create()?;
    let schema = cs2::dump_schema(&cs2)?;

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

#[derive(Debug)]
pub struct BuildInfo {
    revision: String,
    build_datetime: String,
}

impl BuildInfo {
    fn find_build_info(cs2: &CS2Handle) -> anyhow::Result<u64> {
        cs2.resolve_signature(
            Module::Engine,
            &Signature::relative_address(
                obfstr!("client build info"),
                obfstr!("48 8B 1D ? ? ? ? 48 85 DB 74 6B"),
                0x03,
                0x07,
            ),
        )
    }

    pub fn read_build_info(cs2: &CS2Handle) -> anyhow::Result<Self> {
        let address = Self::find_build_info(cs2)?;
        let engine_build_info = cs2.read_schema::<EngineBuildInfo>(&[address])?;
        Ok(Self {
            revision: engine_build_info.revision()?.read_string(&cs2)?,
            build_datetime: format!(
                "{} {}",
                engine_build_info.build_date()?.read_string(&cs2)?,
                engine_build_info.build_time()?.read_string(&cs2)?
            ),
        })
    }
}

fn main_overlay() -> anyhow::Result<()> {
    let settings = load_app_settings()?;

    let cs2 = match CS2Handle::create() {
        Ok(handle) => handle,
        Err(err) => {
            if let Some(err) = err.downcast_ref::<KInterfaceError>() {
                if let KInterfaceError::DeviceUnavailable(_) = &err {
                    if !unsafe { IsUserAnAdmin().as_bool() } {
                        if !is_console_invoked() {
                            /* If we don't have a console, show the message box and abort execution. */
                            show_critical_error("Please re-run this application as administrator!");
                            return Ok(());
                        }

                        /* Just print this warning message and return the actual error.  */
                        log::warn!("Application run without administrator privileges.");
                        log::warn!("Please re-run with administrator privileges!");
                    }
                } else if let KInterfaceError::ProcessDoesNotExists = &err {
                    show_critical_error("Could not find CS2 process.\nPlease start CS2 prior to executing this application!");
                    return Ok(());
                }
            }

            return Err(err);
        }
    };
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

    let cs2_offsets = Arc::new(
        CS2Offsets::resolve_offsets(&cs2)
            .with_context(|| obfstr!("failed to load CS2 offsets").to_string())?,
    );

    let imgui_settings = settings.imgui.clone();
    let settings = Rc::new(RefCell::new(settings));
    let app = Application {
        cs2: cs2.clone(),
        cs2_entities: EntitySystem::new(cs2.clone(), cs2_offsets.clone()),
        cs2_offsets: cs2_offsets.clone(),
        cs2_globals: None,
        cs2_build_info,

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
        class_name_cache: EntryCache::new({
            let cs2 = cs2.clone();
            move |class_info: &Ptr<()>| {
                let address = class_info.address()?;
                let class_name = cs2.read_string(&[address + 0x28, 0x08, 0x00], Some(32))?;
                Ok(Some(class_name))
            }
        }),
        view_controller: ViewController::new(cs2_offsets.clone()),

        enhancements: vec![
            Rc::new(RefCell::new(PlayerESP::new())),
            Rc::new(RefCell::new(BombInfo::new())),
            Rc::new(RefCell::new(TriggerBot::new(LocalCrosshair::new(
                cs2_offsets.offset_crosshair_id,
            )))),
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

    log::debug!("Initialize overlay");
    // OverlayError
    let mut overlay = match overlay::init(obfstr!("CS2 Overlay"), obfstr!("Counter-Strike 2")) {
        Err(OverlayError::VulkanDllNotFound(LoadingError::LibraryLoadFailure(source))) => {
            match &source {
                libloading::Error::LoadLibraryExW { .. } => {
                    let message = format!("Failed to load vulkan-1.dll.\nError: {:#}", source);
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
