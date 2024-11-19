use std::{
    cell::{
        Ref,
        RefCell,
        RefMut,
    },
    error::Error,
    fmt::Debug,
    rc::Rc,
    sync::{
        atomic::{
            AtomicBool,
            Ordering,
        },
        Arc,
        Mutex,
    },
    time::{
        Duration,
        Instant,
    },
};

use anyhow::Context;
use clap::Parser;
use cs2::{
    offsets_runtime,
    CS2Handle,
    ConVars,
    InterfaceError,
    StateBuildInfo,
    StateCS2Handle,
    StateCS2Memory,
};
use enhancements::{
    Enhancement,
    GrenadeHelper,
};
use imgui::{
    Condition,
    FontConfig,
    FontId,
    FontSource,
    Ui,
};
use obfstr::obfstr;
use overlay::{
    LoadingError,
    OverlayError,
    OverlayOptions,
    OverlayTarget,
    SystemRuntimeController,
    UnicodeTextRenderer,
};
use radar::WebRadar;
use settings::{
    load_app_settings,
    AppSettings,
    SettingsUI,
};
use tokio::runtime;
use utils::show_critical_error;
use utils_state::StateRegistry;
use view::ViewController;
use windows::Win32::UI::Shell::IsUserAnAdmin;

use crate::{
    enhancements::{
        AntiAimPunsh,
        BombInfoIndicator,
        PlayerESP,
        SpectatorsListIndicator,
        TriggerBot,
        Aimbot,
    },
    settings::save_app_settings,
    winver::version_info,
};

mod enhancements;
mod radar;
mod settings;
mod utils;
mod view;
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
    pub input: &'a dyn KeyboardInput,
    pub states: &'a StateRegistry,

    pub cs2: &'a Arc<CS2Handle>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FontReference {
    inner: Arc<RefCell<Option<FontId>>>,
}

impl FontReference {
    pub fn font_id(&self) -> Option<FontId> {
        self.inner.borrow().clone()
    }

    pub fn set_id(&self, font_id: FontId) {
        *self.inner.borrow_mut() = Some(font_id);
    }
}

#[derive(Clone, Default)]
pub struct AppFonts {
    valthrun: FontReference,
}

pub struct Application {
    pub fonts: AppFonts,
    pub app_state: StateRegistry,

    pub cs2: Arc<CS2Handle>,
    pub enhancements: Vec<Rc<RefCell<dyn Enhancement>>>,

    pub frame_read_calls: usize,
    pub last_total_read_calls: usize,

    pub settings_visible: bool,
    pub settings_dirty: bool,
    pub settings_ui: RefCell<SettingsUI>,
    pub settings_screen_capture_changed: AtomicBool,
    pub settings_render_debug_window_changed: AtomicBool,

    pub web_radar: RefCell<Option<Arc<Mutex<WebRadar>>>>,
}

impl Application {
    pub fn settings(&self) -> Ref<'_, AppSettings> {
        self.app_state
            .get::<AppSettings>(())
            .expect("app settings to be present")
    }

    pub fn settings_mut(&self) -> RefMut<'_, AppSettings> {
        self.app_state
            .get_mut::<AppSettings>(())
            .expect("app settings to be present")
    }

    pub fn pre_update(&mut self, controller: &mut SystemRuntimeController) -> anyhow::Result<()> {
        if self.settings_dirty {
            self.settings_dirty = false;
            let mut settings = self.settings_mut();

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
            let settings = self.settings();
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
            let settings = self.settings();
            controller.toggle_debug_overlay(settings.render_debug_window);
        }

        Ok(())
    }

    pub fn update(&mut self, ui: &imgui::Ui) -> anyhow::Result<()> {
        {
            for enhancement in self.enhancements.iter() {
                let mut hack = enhancement.borrow_mut();
                if hack.update_settings(ui, &mut *self.settings_mut())? {
                    self.settings_dirty = true;
                }
            }
        }

        if ui.is_key_pressed_no_repeat(self.settings().key_settings.0) {
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

        self.app_state.invalidate_states();
        if let Ok(mut view_controller) = self.app_state.resolve_mut::<ViewController>(()) {
            view_controller.update_screen_bounds(mint::Vector2::from_slice(&ui.io().display_size));
        }

        let update_context = UpdateContext {
            cs2: &self.cs2,

            states: &self.app_state,
            input: ui,
        };

        for enhancement in self.enhancements.iter() {
            let mut enhancement = enhancement.borrow_mut();
            enhancement.update(&update_context)?;
        }

        let read_calls = self.cs2.ke_interface.total_read_calls();
        self.frame_read_calls = read_calls - self.last_total_read_calls;
        self.last_total_read_calls = read_calls;

        Ok(())
    }

    pub fn render(&self, ui: &imgui::Ui, unicode_text: &UnicodeTextRenderer) {
        ui.window("overlay")
            .draw_background(false)
            .no_decoration()
            .no_inputs()
            .size(ui.io().display_size, Condition::Always)
            .position([0.0, 0.0], Condition::Always)
            .build(|| self.render_overlay(ui, unicode_text));

        {
            for enhancement in self.enhancements.iter() {
                let mut enhancement = enhancement.borrow_mut();
                if let Err(err) = enhancement.render_debug_window(&self.app_state, ui, unicode_text)
                {
                    log::error!("{:?}", err);
                }
            }
        }

        if self.settings_visible {
            let mut settings_ui = self.settings_ui.borrow_mut();
            settings_ui.render(self, ui, unicode_text)
        }
    }

    fn render_overlay(&self, ui: &imgui::Ui, unicode_text: &UnicodeTextRenderer) {
        let settings = self.settings();

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

        for enhancement in self.enhancements.iter() {
            let hack = enhancement.borrow();
            if let Err(err) = hack.render(&self.app_state, ui, unicode_text) {
                log::error!("{:?}", err);
            }
        }
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

    let runtime = runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(1)
        .build()
        .expect("to be able to build a runtime");

    let _runtime_guard = runtime.enter();
    if let Err(error) = real_main() {
        show_critical_error(&format!("{:#}", error));
    }
}

#[derive(Debug, Parser)]
#[clap(name = "Valthrun", version)]
struct AppArgs {
    /// Enable verbose logging ($env:RUST_LOG="trace")
    #[clap(short, long)]
    verbose: bool,
}

fn real_main() -> anyhow::Result<()> {
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
            if let Some(err) = err.downcast_ref::<InterfaceError>() {
                if let Some(detailed_message) = err.detailed_message() {
                    show_critical_error(&detailed_message);
                    return Ok(());
                }
            }

            return Err(err);
        }
    };

    cs2.add_metrics_record(obfstr!("controller-status"), "initializing");

    let mut app_state = StateRegistry::new(1024 * 8);
    app_state.set(StateCS2Handle::new(cs2.clone()), ())?;
    app_state.set(StateCS2Memory::new(cs2.create_memory_view()), ())?;
    app_state.set(settings, ())?;

    {
        let cs2_build_info = app_state.resolve::<StateBuildInfo>(()).with_context(|| {
            obfstr!(
                "Failed to load CS2 build info. CS2 version might be newer / older then expected"
            )
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
    }

    offsets_runtime::setup_provider(&app_state)?;

    let cvars = ConVars::new(&app_state).context("cvars")?;
    let cvar_sensitivity = cvars
        .find_cvar("sensitivity")
        .context("cvar ensitivity")?
        .context("missing cvar ensitivity")?;

    log::debug!("Initialize overlay");
    let app_fonts: AppFonts = Default::default();
    let overlay_options = OverlayOptions {
        title: obfstr!("CS2 Overlay").to_string(),
        target: OverlayTarget::WindowOfProcess(cs2.process_id() as u32),
        register_fonts_callback: Some(Box::new({
            let app_fonts = app_fonts.clone();

            move |atlas| {
                let font_size = 18.0;
                let valthrun_font = atlas.add_font(&[FontSource::TtfData {
                    data: include_bytes!("../resources/Valthrun-Regular.ttf"),
                    size_pixels: font_size,
                    config: Some(FontConfig {
                        rasterizer_multiply: 1.5,
                        oversample_h: 4,
                        oversample_v: 4,
                        ..FontConfig::default()
                    }),
                }]);

                app_fonts.valthrun.set_id(valthrun_font);
            }
        })),
    };

    let mut overlay = match overlay::init(overlay_options) {
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

    {
        let settings = app_state.resolve::<AppSettings>(())?;
        if let Some(imgui_settings) = &settings.imgui {
            overlay.imgui.load_ini_settings(imgui_settings);
        }
    }

    let app = Application {
        fonts: app_fonts,
        app_state,

        cs2: cs2.clone(),
        web_radar: Default::default(),

        enhancements: vec![
            Rc::new(RefCell::new(AntiAimPunsh::new(cvar_sensitivity))),
            Rc::new(RefCell::new(PlayerESP::new())),
            Rc::new(RefCell::new(SpectatorsListIndicator::new())),
            Rc::new(RefCell::new(BombInfoIndicator::new())),
            Rc::new(RefCell::new(TriggerBot::new())),
            Rc::new(RefCell::new(GrenadeHelper::new())),
            Rc::new(RefCell::new(Aimbot::new())),
        ],

        last_total_read_calls: 0,
        frame_read_calls: 0,

        settings_visible: false,
        settings_dirty: false,
        settings_ui: RefCell::new(SettingsUI::new()),
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
        move |ui, unicode_text| {
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

            app.render(ui, unicode_text);
            true
        },
    );

    Ok(())
}