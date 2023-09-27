use clipboard::ClipboardSupport;
use copypasta::ClipboardContext;
use error::{OverlayError, Result};
use glium::glutin;
use glium::glutin::event::{Event, WindowEvent};
use glium::glutin::event_loop::{ControlFlow, EventLoop};
use glium::glutin::platform::windows::WindowExtWindows;
use glium::glutin::window::{Window, WindowBuilder};
use glium::{Display, Surface};
use imgui::{Context, FontConfig, FontSource, Io};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use input::{KeyboardInputSystem, MouseInputSystem};
use obfstr::obfstr;
use std::ffi::CString;
use std::time::Instant;
use window_tracker::WindowTracker;
use windows::core::PCSTR;
use windows::Win32::Foundation::{BOOL, HWND};
use windows::Win32::Graphics::Dwm::{
    DwmEnableBlurBehindWindow, DWM_BB_BLURREGION, DWM_BB_ENABLE, DWM_BLURBEHIND,
};
use windows::Win32::Graphics::Gdi::CreateRectRgn;
use windows::Win32::UI::Input::KeyboardAndMouse::SetActiveWindow;
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowLongPtrA, MessageBoxA, SetWindowDisplayAffinity, SetWindowLongA, SetWindowLongPtrA,
    SetWindowPos, ShowWindow, GWL_EXSTYLE, GWL_STYLE, HWND_TOPMOST, MB_ICONERROR, MB_OK,
    SWP_NOMOVE, SWP_NOSIZE, SW_SHOW, WDA_EXCLUDEFROMCAPTURE, WS_CLIPSIBLINGS, WS_EX_LAYERED,
    WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT, WS_POPUP, WS_VISIBLE, WDA_NONE,
};

mod clipboard;
mod error;
mod input;
mod window_tracker;

pub fn show_error_message(title: &str, message: &str) {
    let title = CString::new(title).unwrap_or_else(|_| CString::new("[[ NulError ]]").unwrap());
    let message = CString::new(message).unwrap_or_else(|_| CString::new("[[ NulError ]]").unwrap());
    unsafe {
        MessageBoxA(
            HWND::default(),
            PCSTR::from_raw(message.as_ptr() as *const u8),
            PCSTR::from_raw(title.as_ptr() as *const u8),
            MB_ICONERROR | MB_OK,
        );
    }
}

pub struct System {
    pub event_loop: EventLoop<()>,
    pub display: glium::Display,
    pub imgui: Context,
    pub platform: WinitPlatform,
    pub renderer: Renderer,
    pub font_size: f32,
    pub window_tracker: WindowTracker,
}

pub fn init(title: &str, target_window: &str) -> Result<System> {
    let window_tracker = WindowTracker::new(target_window)?;

    let event_loop = EventLoop::new();
    let context = glutin::ContextBuilder::new().with_vsync(false);

    let builder = WindowBuilder::new()
        .with_title(title.to_owned())
        .with_visible(false);

    let display =
        Display::new(builder, context, &event_loop).map_err(OverlayError::DisplayError)?;

    let mut imgui = Context::create();
    imgui.set_ini_filename(None);

    match ClipboardContext::new() {
        Ok(backend) => imgui.set_clipboard_backend(ClipboardSupport(backend)),
        Err(error) => log::warn!("Failed to initialize clipboard: {}", error),
    };

    let mut platform = WinitPlatform::init(&mut imgui);
    {
        let gl_window = display.gl_window();
        let window = gl_window.window();
        platform.attach_window(imgui.io_mut(), window, HiDpiMode::Default);
    }

    // Fixed font size. Note imgui_winit_support uses "logical
    // pixels", which are physical pixels scaled by the devices
    // scaling factor. Meaning, 13.0 pixels should look the same size
    // on two different screens, and thus we do not need to scale this
    // value (as the scaling is handled by winit)
    let font_size = 18.0;

    imgui.fonts().add_font(&[FontSource::TtfData {
        data: include_bytes!("../resources/Roboto-Regular.ttf"),
        size_pixels: font_size,
        config: Some(FontConfig {
            // As imgui-glium-renderer isn't gamma-correct with
            // it's font rendering, we apply an arbitrary
            // multiplier to make the font a bit "heavier". With
            // default imgui-glow-renderer this is unnecessary.
            rasterizer_multiply: 1.5,
            // Oversampling font helps improve text rendering at
            // expense of larger font atlas texture.
            oversample_h: 4,
            oversample_v: 4,
            ..FontConfig::default()
        }),
    }]);

    {
        let window = display.gl_window();
        let window = window.window();

        // window.set_decorations(false);
        // window.set_undecorated_shadow(false);

        let hwnd = HWND(window.hwnd());
        unsafe {
            // Make it transparent
            SetWindowLongA(
                hwnd,
                GWL_STYLE,
                (WS_POPUP | WS_VISIBLE | WS_CLIPSIBLINGS).0 as i32,
            );
            SetWindowLongPtrA(
                hwnd,
                GWL_EXSTYLE,
                (WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TRANSPARENT).0
                    as isize,
            );

            let mut bb: DWM_BLURBEHIND = Default::default();
            bb.dwFlags = DWM_BB_ENABLE | DWM_BB_BLURREGION;
            bb.fEnable = BOOL::from(true);
            bb.hRgnBlur = CreateRectRgn(0, 0, 1, 1);
            DwmEnableBlurBehindWindow(hwnd, &bb)?;

            // Move the window to the top
            SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
        }
    }

    let renderer = Renderer::init(&mut imgui, &display).map_err(OverlayError::RenderError)?;

    Ok(System {
        event_loop,
        display,
        imgui,
        platform,
        renderer,
        font_size,
        window_tracker,
    })
}

/// Toggles the overlay noactive and transparent state
/// according to whenever ImGui wants mouse/cursor grab.
struct OverlayActiveTracker {
    currently_active: bool,
}

impl OverlayActiveTracker {
    pub fn new() -> Self {
        Self {
            currently_active: true,
        }
    }

    pub fn update(&mut self, window: &Window, io: &Io) {
        let window_active = io.want_capture_mouse | io.want_capture_keyboard;
        if window_active == self.currently_active {
            return;
        }

        self.currently_active = window_active;
        unsafe {
            let hwnd = HWND(window.hwnd());
            let mut style = GetWindowLongPtrA(hwnd, GWL_EXSTYLE);
            if window_active {
                style &= !((WS_EX_NOACTIVATE | WS_EX_TRANSPARENT).0 as isize);
            } else {
                style |= (WS_EX_NOACTIVATE | WS_EX_TRANSPARENT).0 as isize;
            }

            log::trace!("Set UI active: {window_active}");
            SetWindowLongPtrA(hwnd, GWL_EXSTYLE, style);
            if window_active {
                SetActiveWindow(hwnd);
            }
        }
    }
}

impl System {
    pub fn main_loop<U, R>(self, mut update: U, mut render: R) -> !
    where
        U: FnMut(&mut SystemRuntimeController) -> bool + 'static,
        R: FnMut(&mut imgui::Ui) -> bool + 'static,
    {
        let System {
            event_loop,
            display,
            imgui,
            mut platform,
            mut renderer,
            window_tracker,
            ..
        } = self;
        let mut last_frame = Instant::now();

        let mut runtime_controller = SystemRuntimeController {
            hwnd: HWND(display.gl_window().window().hwnd() as isize),
            imgui,

            active_tracker: OverlayActiveTracker::new(),
            key_input_system: KeyboardInputSystem::new(),
            mouse_input_system: MouseInputSystem::new(),
            window_tracker,

            frame_count: 0,
        };

        event_loop.run(move |event, _, control_flow| match event {
            Event::NewEvents(_) => {
                let now = Instant::now();
                runtime_controller.imgui.io_mut().update_delta_time(now - last_frame);
                last_frame = now;
            }
            Event::MainEventsCleared => {
                let gl_window = display.gl_window();
                if let Err(error) = platform.prepare_frame(runtime_controller.imgui.io_mut(), gl_window.window()) {
                    *control_flow = ControlFlow::ExitWithCode(1);
                    log::error!("Platform implementation prepare_frame failed: {}", error);
                    return;
                }

                let window = gl_window.window();
                if !runtime_controller.update_state(window) {
                    log::info!("Target window has been closed. Exiting overlay.");
                    *control_flow = ControlFlow::Exit;
                    return;
                }

                if !update(&mut runtime_controller) {
                    *control_flow = ControlFlow::Exit;
                    return;
                }

                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                let gl_window = display.gl_window();
                let ui = runtime_controller.imgui.frame();

                let mut run = render(ui);

                let mut target = display.draw();
                target.clear_all((0.0, 0.0, 0.0, 0.0), 0.0, 0);
                platform.prepare_render(ui, gl_window.window());

                let draw_data = runtime_controller.imgui.render();

                if let Err(error) = renderer.render(&mut target, draw_data) {
                    log::error!("Failed to render ImGui draw data: {}", error);
                    run = false;
                } else if let Err(error) = target.finish() {
                    log::error!("Failed to swap render buffers: {}", error);
                    run = false;
                }

                if !run {
                    *control_flow = ControlFlow::Exit;
                }

                runtime_controller.frame_rendered();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            event => {
                let gl_window = display.gl_window();
                platform.handle_event(runtime_controller.imgui.io_mut(), gl_window.window(), &event);
            }
        })
    }
}

pub struct SystemRuntimeController {
    pub hwnd: HWND,

    pub imgui: imgui::Context,

    active_tracker: OverlayActiveTracker,
    mouse_input_system: MouseInputSystem,
    key_input_system: KeyboardInputSystem,

    window_tracker: WindowTracker,

    frame_count: u64,
}

impl SystemRuntimeController {
    fn update_state(&mut self, window: &glutin::window::Window) -> bool {
        self.mouse_input_system.update(window, self.imgui.io_mut());
        self.key_input_system.update(window, self.imgui.io_mut());
        self.active_tracker.update(window, self.imgui.io());
        if !self.window_tracker.update(window) {
            log::info!("Target window has been closed. Exiting overlay.");
            return false;
        }

        true
    }

    fn frame_rendered(&mut self) {
        self.frame_count += 1;
        if self.frame_count == 1 {
            /* initial frame */
            unsafe { ShowWindow(self.hwnd, SW_SHOW) };

            self.window_tracker.mark_force_update();
        }
    }

    pub fn toggle_screen_capture_visibility(&self, should_be_visible: bool) {
        unsafe {
            let (target_state, state_name) = if should_be_visible {
                (WDA_NONE, "normal")
            } else {
                (WDA_EXCLUDEFROMCAPTURE, "exclude from capture")
            };

            if !SetWindowDisplayAffinity(self.hwnd, target_state).as_bool() {
                log::warn!(
                    "{} '{}'.",
                    obfstr!("Failed to change overlay display affinity to"),
                    state_name
                );
            }
        }
    }
}