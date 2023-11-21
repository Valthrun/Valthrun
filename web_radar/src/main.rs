use std::{
    cell::RefCell,
    rc::Rc,
    sync::Arc,
};

use anyhow::Context;
use cs2::{
    BuildInfo,
    CS2Handle,
    CS2Model,
    CS2Offsets,
    EntitySystem,
    Globals,
};
use obfstr::obfstr;
use valthrun_kernel_interface::KInterfaceError;
use valthrun_toolkit::{
    get_current_map,
    setup_runtime_offset_provider,
    version_info,
    ClassNameCache,
    EntryCache,
    MapInfo,
    ViewController,
};

use crate::{
    web_radar::WebRadar,
    web_radar_server::{
        MessageData,
        CLIENTS,
    },
};

mod web_radar;
mod web_radar_server;

pub struct UpdateContext<'a> {
    pub current_map: &'a Option<MapInfo>,
    pub current_map_changed: &'a bool,

    pub cs2: &'a Arc<CS2Handle>,
    pub cs2_entities: &'a EntitySystem,

    pub model_cache: &'a EntryCache<u64, CS2Model>,
    pub class_name_cache: &'a ClassNameCache,
    pub view_controller: &'a ViewController,

    pub globals: Globals,
}

pub struct Application {
    pub cs2: Arc<CS2Handle>,
    pub cs2_offsets: Arc<CS2Offsets>,
    pub cs2_entities: EntitySystem,
    pub cs2_globals: Option<Globals>,
    pub cs2_build_info: BuildInfo,

    pub current_map: Option<MapInfo>,
    pub current_map_changed: bool,

    pub web_radar: WebRadar,

    pub model_cache: EntryCache<u64, CS2Model>,
    pub class_name_cache: ClassNameCache,
    pub view_controller: ViewController,

    pub frame_read_calls: usize,
    pub last_total_read_calls: usize,
}

impl Application {
    pub fn update(&mut self) -> anyhow::Result<()> {
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

            globals,
            class_name_cache: &self.class_name_cache,
            view_controller: &self.view_controller,
            model_cache: &self.model_cache,
        };

        self.web_radar
            .update(&update_context)
            .with_context(|| obfstr!("Failed to update radar").to_string())?;

        let read_calls = self.cs2.ke_interface.total_read_calls();
        self.frame_read_calls = read_calls - self.last_total_read_calls;
        self.last_total_read_calls = read_calls;

        Ok(())
    }
}

fn show_critical_error(message: &str) {
    for line in message.lines() {
        log::error!("{}", line);
    }
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

    log::info!("Valthrun web radar v{}.", env!("CARGO_PKG_VERSION"),);

    let cs2 = match CS2Handle::create(false) {
        Ok(handle) => handle,
        Err(err) => {
            if let Some(err) = err.downcast_ref::<KInterfaceError>() {
                if let KInterfaceError::DeviceUnavailable(error) = &err {
                    if error.code().0 as u32 == 0x80070002 {
                        /* The system cannot find the file specified. */
                        show_critical_error(obfstr!("** PLEASE READ CAREFULLY **\nCould not find the kernel driver interface.\nEnsure you have successfully loaded/mapped the kernel driver (valthrun-driver.sys) before starting the web-radar.\nPlease explicitly check the driver entry status code which should be 0x0.\n\nFor more help, checkout:\nhttps://wiki.valth.run/#/030_troubleshooting/overlay/020_driver_has_not_been_loaded."));
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
                        "\nThe installed/loaded Valthrun driver version is too new.\nPlease ensure you're using the latest web-radar."
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

    setup_runtime_offset_provider(&cs2)?;

    let app = Application {
        cs2: cs2.clone(),
        cs2_entities: EntitySystem::new(cs2.clone(), cs2_offsets.clone()),
        cs2_offsets: cs2_offsets.clone(),
        cs2_globals: None,
        cs2_build_info,

        current_map: None,
        current_map_changed: false,

        web_radar: WebRadar::new(),

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

        last_total_read_calls: 0,
        frame_read_calls: 0,
    };
    let app = Rc::new(RefCell::new(app));

    log::info!("Starting web radar.");
    std::thread::spawn(|| {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(web_radar_server::run_server())
    });

    loop {
        let mut app = app.borrow_mut();
        if let Err(err) = app.update() {
            log::error!("Error: {:#}", err);
        }
    }
}
