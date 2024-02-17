use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::Context;
use cs2::{
    BuildInfo,
    CS2Handle,
    CS2Offsets,
    EntitySystem,
};
use obfstr::obfstr;
use valthrun_kernel_interface::{KeyboardState, MouseState};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState,
    VK_XBUTTON2, VK_A, MapVirtualKeyA, MAP_VIRTUAL_KEY_TYPE, VK_D, VK_NUMPAD0, VIRTUAL_KEY,
};
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum LogState {
    MissingLocalController,
    MissingLocalPawn,
    LocalPawnDead,
    Armed,
    Active,
}

const SC_NUMPAD_MINUS: u16 = 0x4A;
const SC_A: u16 = 0x1E;
const SC_D: u16 = 0x20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum StrafeState {
    None,
    StrafeRight,
    StrafeLeft
}

impl StrafeState {
    pub fn opposite(&self) -> Self {
        match self {
            Self::None => Self::None,
            Self::StrafeRight => Self::StrafeLeft,
            Self::StrafeLeft => Self::StrafeRight
        }
    }

    pub fn sign(&self) -> i32 {
        match self {
            Self::None => 0,
            Self::StrafeRight => 1,
            Self::StrafeLeft => -1
        }
    }
}

struct Autostrafe {
    cs2: Arc<CS2Handle>,

    strafe_time_ms: u32,
    strafe_total: u32,

    strafe_start: Instant,
    strafe_state: StrafeState,

    strafe_applied_offset: u32,
    strafe_time_offset: u32,
}

impl Autostrafe {
    pub fn new(cs2: Arc<CS2Handle>) -> Self {
        Self {
            cs2,

            strafe_time_ms: 675,
            strafe_total: 6000,

            strafe_start: Instant::now(),
            strafe_state: StrafeState::None,

            strafe_applied_offset: 0,
            strafe_time_offset: 0,
        }
    }

    fn update_keyboard(&self) -> anyhow::Result<()> {
        log::debug!("Send KB {:?}", self.strafe_state);
        match self.strafe_state {
            StrafeState::StrafeLeft => {
                self.cs2.send_keyboard_state(&[
                    KeyboardState { scane_code: SC_D, down: false },
                ])?;
                std::thread::sleep(Duration::from_micros(100));
                self.cs2.send_keyboard_state(&[
                    KeyboardState { scane_code: SC_A, down: true },
                ])?;
            },
            StrafeState::StrafeRight => {
                self.cs2.send_keyboard_state(&[
                    KeyboardState { scane_code: SC_A, down: false },
                ])?;
                std::thread::sleep(Duration::from_micros(100));
                self.cs2.send_keyboard_state(&[
                    KeyboardState { scane_code: SC_D, down: true },
                ])?;
            },
            StrafeState::None => {
                self.cs2.send_keyboard_state(&[
                    KeyboardState { scane_code: SC_A, down: false },
                ])?;
                std::thread::sleep(Duration::from_micros(100));
                self.cs2.send_keyboard_state(&[
                    KeyboardState { scane_code: SC_D, down: false },
                ])?;
            }
        }
        Ok(())
    }

    pub fn update(&mut self, should_strafe: bool) -> anyhow::Result<()> {
        if !should_strafe {
            if self.strafe_state == StrafeState::None {
                /* nothing changed */
                return Ok(());
            }

            self.strafe_state = StrafeState::None;
            self.strafe_applied_offset = 0;
            self.update_keyboard()?;
            return Ok(());
        }

        if self.strafe_state == StrafeState::None {
            /* strafe start */
            self.strafe_start = Instant::now();
            self.strafe_state = StrafeState::StrafeLeft;
            self.strafe_applied_offset = self.strafe_total / 2;
            self.strafe_time_offset = self.strafe_time_ms / 2;
            self.update_keyboard()?;
        }

        let time_delta = self.strafe_start.elapsed().as_millis() as u32 + self.strafe_time_offset;

        if time_delta > self.strafe_time_ms {
            let offset_difference = (self.strafe_total - self.strafe_applied_offset) as i32;
            log::debug!("Applying {} pending difference from {:?}", offset_difference, self.strafe_state);
            if offset_difference > 0 {
                let pending_difference = offset_difference * self.strafe_state.sign();
                self.cs2.send_mouse_state(&[ MouseState { last_x: pending_difference, ..Default::default() } ])?;
            }

            self.strafe_state = self.strafe_state.opposite();
            self.strafe_start = Instant::now();
            self.strafe_applied_offset = 0;
            self.strafe_time_offset = 0;
            self.update_keyboard()?;
            return Ok(());
        }

        let x = time_delta as f32 / self.strafe_time_ms as f32;
        let offset = (x * x) / (2.0 * (x*x - x) + 1.0);
        let expected_offset = (self.strafe_total as f32 * offset) as u32;

        if expected_offset > self.strafe_applied_offset {
            let pending_difference = (expected_offset - self.strafe_applied_offset) as i32 * self.strafe_state.sign();
            if pending_difference.abs() > 60 {
                self.cs2.send_mouse_state(&[ MouseState { last_x: pending_difference, ..Default::default() } ])?;
                self.strafe_applied_offset = expected_offset;

                match self.strafe_state {
                    StrafeState::StrafeLeft => {
                        // self.cs2.send_keyboard_state(&[
                        //     KeyboardState { scane_code: SC_A, down: false },
                        // ])?;
                        // std::thread::sleep(Duration::from_micros(100));
                        self.cs2.send_keyboard_state(&[
                            KeyboardState { scane_code: SC_A, down: true },
                        ])?;
                    },
                    StrafeState::StrafeRight => {
                        // self.cs2.send_keyboard_state(&[
                        //     KeyboardState { scane_code: SC_D, down: false },
                        // ])?;
                        // std::thread::sleep(Duration::from_micros(100));
                        self.cs2.send_keyboard_state(&[
                            KeyboardState { scane_code: SC_D, down: true },
                        ])?;
                    },
                    _ => {}
                }
            }
        }

        Ok(())
    }
}

struct KeyState {
    name: String,
    vk: i32,
    last_state: u16
}

impl KeyState {
    pub fn new(vk: VIRTUAL_KEY, name: String) -> Self {
        KeyState { name, vk: vk.0 as i32, last_state: 0 }
    }

    pub fn update(&mut self) {
        let new_state = unsafe { GetAsyncKeyState(self.vk) as u16 };
        if new_state == self.last_state {
            return;
        }

        log::debug!("{}: {:X} -> {:X}", self.name, self.last_state, new_state);
        self.last_state = new_state;
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

    log::info!("Valthrun BHop script v{}.", env!("CARGO_PKG_VERSION"),);

    let cs2 = CS2Handle::create()?;
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
    let mut cs2_entities = EntitySystem::new(cs2.clone(), cs2_offsets.clone());

    // How I came up with the kebinds:
    // https://www.youtube.com/watch?v=xRQR97_fPfc

    log::info!("Starting BHop.");
    log::info!("");
    log::warn!("Attention: In order for the B-Hop script to work, please enter the following command into your game console:");
    log::warn!(r#"alias XHOP_REL "-jump;"; alias XHOP_JMP "+jump; XHOP_REL;"; bind "KP_MINUS" "+XHOP_JMP""#);
    log::info!("");

    //log::debug!("{:X}", unsafe { MapVirtualKeyA(VK_D.0 as u32, MAP_VIRTUAL_KEY_TYPE(0)) });

    // let mut last_state = false;
    // let mut last_state_a = 0;
    // loop {
    //     let new_state_a = unsafe { GetAsyncKeyState(VK_A.0 as i32) };
    //     if new_state_a != last_state_a {
    //         log::debug!("A: {:X} -> {:X}", last_state_a, new_state_a);
    //         last_state_a = new_state_a;
    //     }

    //     let should_jump = unsafe { GetAsyncKeyState(VK_NUMPAD0.0 as i32) != 0 };
    //     if should_jump == last_state {
    //         continue;
    //     }

    //     log::debug!("Start changed {} -> {}", last_state, should_jump);
    //     last_state = should_jump;
    //     cs2.send_keyboard_state(&[
    //         KeyboardState { scane_code: SC_A, down: last_state },
    //     ])?;
    // }

    let mut ks_a = KeyState::new(VK_A, "  A".to_string());
    let mut ks_d = KeyState::new(VK_D, "  D".to_string());
    let mut ks_mb2 = KeyState::new(VK_XBUTTON2, "MB2".to_string());

    let mut auto_strafe = Autostrafe::new(cs2.clone());
    let mut log_state = LogState::Active;
    loop {
        cs2_entities
            .read_entities()
            .with_context(|| obfstr!("failed to read entities").to_string())?;

        let controller = match {
            cs2_entities
                .get_local_player_controller()?
                .try_reference_schema()?
        } {
            Some(controller) => controller,
            None => {
                if log_state != LogState::MissingLocalController {
                    log_state = LogState::MissingLocalController;
                    log::info!("Missing local player controller. Waiting...");
                }

                std::thread::sleep(Duration::from_millis(250));
                continue;
            }
        };

        if !controller.m_bPawnIsAlive()? {
            if log_state != LogState::LocalPawnDead {
                log_state = LogState::LocalPawnDead;
                log::info!("Local pawn is dead. Waiting...");
            }

            std::thread::sleep(Duration::from_millis(250));
            continue;
        }

        let pawn = match cs2_entities.get_by_handle(&controller.m_hPawn()?)? {
            Some(pawn) => pawn.entity()?.reference_schema()?,
            None => {
                if log_state != LogState::MissingLocalPawn {
                    log_state = LogState::MissingLocalPawn;
                    log::info!("Missing local player pawn. Waiting...");
                }

                std::thread::sleep(Duration::from_millis(250));
                continue;
            }
        };

        ks_mb2.update();
        ks_a.update();
        ks_d.update();

        let in_air = pawn.m_fFlags()? & 0x01 == 0;
        let should_jump = unsafe { GetAsyncKeyState(VK_XBUTTON2.0 as i32) != 0 };
        if !in_air && should_jump {
            if log_state != LogState::Active {
                log_state = LogState::Active;
                log::info!("Sending jumps...");
            }

            cs2.send_keyboard_state(&[KeyboardState {
                down: true,
                scane_code: SC_NUMPAD_MINUS,
            }])?;
            std::thread::sleep(Duration::from_millis(1));
            cs2.send_keyboard_state(&[KeyboardState {
                down: false,
                scane_code: SC_NUMPAD_MINUS,
            }])?;
        } else {
            match log_state {
                LogState::Armed => {},
                LogState::Active => log::info!("Jump initiated, await landing..."),
                _ => log::info!("Awaiting jump button to be pressed...")
            };

            log_state = LogState::Armed;
        }

        // auto_strafe.update(should_jump)
        //     .with_context(|| obfstr!("autostrafe").to_string())?;
    }
}
