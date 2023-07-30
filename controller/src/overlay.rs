use copypasta::{ClipboardContext, ClipboardProvider};
use glium::glutin;
use glium::glutin::event::{Event, WindowEvent};
use glium::glutin::event_loop::{ControlFlow, EventLoop};
use glium::glutin::platform::windows::WindowExtWindows;
use glium::glutin::window::{Window, WindowBuilder};
use glium::{Display, Surface};
use imgui::{ClipboardBackend, Context, FontConfig, FontSource, Key, MouseButton, Ui};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use std::time::Instant;
use windows::Win32::Foundation::{BOOL, HWND, POINT};
use windows::Win32::Graphics::Dwm::{
    DwmEnableBlurBehindWindow, DWM_BB_BLURREGION, DWM_BB_ENABLE, DWM_BLURBEHIND,
};
use windows::Win32::Graphics::Gdi::{CreateRectRgn, ScreenToClient};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, SetActiveWindow, VIRTUAL_KEY, VK_CONTROL, VK_LBUTTON, VK_LCONTROL, VK_LMENU,
    VK_LSHIFT, VK_LWIN, VK_MBUTTON, VK_MENU, VK_RBUTTON, VK_RMENU, VK_RSHIFT, VK_RWIN, VK_XBUTTON1,
    VK_XBUTTON2,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetCursorPos, GetWindowLongPtrA, SetWindowLongA, SetWindowLongPtrA, SetWindowPos, ShowWindow,
    GWL_EXSTYLE, GWL_STYLE, HWND_TOPMOST, SWP_NOMOVE, SWP_NOSIZE, SW_SHOW, WS_CLIPSIBLINGS,
    WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT, WS_POPUP, WS_VISIBLE,
};

pub struct ClipboardSupport(pub ClipboardContext);
impl ClipboardBackend for ClipboardSupport {
    fn get(&mut self) -> Option<String> {
        self.0.get_contents().ok()
    }
    fn set(&mut self, text: &str) {
        if let Err(error) = self.0.set_contents(text.to_owned()) {
            log::warn!("Failed to set clipboard data: {}", error);
        }
    }
}

pub struct System {
    pub event_loop: EventLoop<()>,
    pub display: glium::Display,
    pub imgui: Context,
    pub platform: WinitPlatform,
    pub renderer: Renderer,
    pub font_size: f32,
}

pub fn init(title: &str) -> System {
    let event_loop = EventLoop::new();
    let context = glutin::ContextBuilder::new().with_vsync(false);

    let target_monitor = event_loop
        .primary_monitor()
        .or_else(|| event_loop.available_monitors().next())
        .unwrap();

    let builder = WindowBuilder::new()
        .with_resizable(false)
        .with_title(title.to_owned())
        .with_inner_size(target_monitor.size())
        .with_position(target_monitor.position());

    let display =
        Display::new(builder, context, &event_loop).expect("Failed to initialize display");

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
    let font_size = 13.0;

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

        window.set_decorations(false);
        window.set_undecorated_shadow(false);

        let hwnd = HWND(window.hwnd());
        unsafe {
            SetWindowLongA(
                hwnd,
                GWL_STYLE,
                (WS_POPUP | WS_VISIBLE | WS_CLIPSIBLINGS).0 as i32,
            );
            SetWindowLongPtrA(
                hwnd,
                GWL_EXSTYLE,
                (WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE).0
                    as isize,
            );
            ShowWindow(hwnd, SW_SHOW);

            SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);

            let mut bb: DWM_BLURBEHIND = Default::default();
            bb.dwFlags = DWM_BB_ENABLE | DWM_BB_BLURREGION;
            bb.fEnable = BOOL::from(true);
            bb.hRgnBlur = CreateRectRgn(0, 0, 1, 1);
            DwmEnableBlurBehindWindow(hwnd, &bb).unwrap();
        }
    }

    let renderer = Renderer::init(&mut imgui, &display).expect("Failed to initialize renderer");

    System {
        event_loop,
        display,
        imgui,
        platform,
        renderer,
        font_size,
    }
}

fn to_imgui_key(keycode: VIRTUAL_KEY) -> Option<Key> {
    use windows::Win32::UI::Input::KeyboardAndMouse::*;

    match keycode {
        VK_TAB => Some(Key::Tab),
        VK_LEFT => Some(Key::LeftArrow),
        VK_RIGHT => Some(Key::RightArrow),
        VK_SHIFT => Some(Key::LeftShift),
        VK_MENU => Some(Key::LeftAlt),
        VK_UP => Some(Key::UpArrow),
        VK_DOWN => Some(Key::DownArrow),
        VK_PRIOR => Some(Key::PageUp),
        VK_NEXT => Some(Key::PageDown),
        VK_HOME => Some(Key::Home),
        VK_END => Some(Key::End),
        VK_INSERT => Some(Key::Insert),
        VK_DELETE => Some(Key::Delete),
        VK_BACK => Some(Key::Backspace),
        VK_SPACE => Some(Key::Space),
        VK_RETURN => Some(Key::Enter),
        VK_ESCAPE => Some(Key::Escape),
        VK_OEM_7 => Some(Key::Apostrophe),
        VK_OEM_COMMA => Some(Key::Comma),
        VK_OEM_MINUS => Some(Key::Minus),
        VK_OEM_PERIOD => Some(Key::Period),
        VK_OEM_2 => Some(Key::Slash),
        VK_OEM_1 => Some(Key::Semicolon),
        VK_OEM_PLUS => Some(Key::Equal),
        VK_OEM_4 => Some(Key::LeftBracket),
        VK_OEM_5 => Some(Key::Backslash),
        VK_OEM_6 => Some(Key::RightBracket),
        VK_OEM_3 => Some(Key::GraveAccent),
        VK_CAPITAL => Some(Key::CapsLock),
        VK_SCROLL => Some(Key::ScrollLock),
        VK_NUMLOCK => Some(Key::NumLock),
        VK_SNAPSHOT => Some(Key::PrintScreen),
        VK_PAUSE => Some(Key::Pause),
        VK_NUMPAD0 => Some(Key::Keypad0),
        VK_NUMPAD1 => Some(Key::Keypad1),
        VK_NUMPAD2 => Some(Key::Keypad2),
        VK_NUMPAD3 => Some(Key::Keypad3),
        VK_NUMPAD4 => Some(Key::Keypad4),
        VK_NUMPAD5 => Some(Key::Keypad5),
        VK_NUMPAD6 => Some(Key::Keypad6),
        VK_NUMPAD7 => Some(Key::Keypad7),
        VK_NUMPAD8 => Some(Key::Keypad8),
        VK_NUMPAD9 => Some(Key::Keypad9),
        VK_DECIMAL => Some(Key::KeypadDecimal),
        VK_DIVIDE => Some(Key::KeypadDivide),
        VK_MULTIPLY => Some(Key::KeypadMultiply),
        VK_SUBTRACT => Some(Key::KeypadSubtract),
        VK_ADD => Some(Key::KeypadAdd),
        VK_LSHIFT => Some(Key::LeftShift),
        VK_LCONTROL | VK_CONTROL => Some(Key::LeftCtrl),
        VK_RCONTROL => Some(Key::RightCtrl),
        VK_LMENU => Some(Key::LeftAlt),
        VK_LWIN => Some(Key::LeftSuper),
        VK_RSHIFT => Some(Key::RightShift),
        VK_RMENU => Some(Key::RightAlt),
        VK_RWIN => Some(Key::RightSuper),
        VK_APPS => Some(Key::Menu),
        VK_0 => Some(Key::Alpha0),
        VK_1 => Some(Key::Alpha1),
        VK_2 => Some(Key::Alpha2),
        VK_3 => Some(Key::Alpha3),
        VK_4 => Some(Key::Alpha4),
        VK_5 => Some(Key::Alpha5),
        VK_6 => Some(Key::Alpha6),
        VK_7 => Some(Key::Alpha7),
        VK_8 => Some(Key::Alpha8),
        VK_9 => Some(Key::Alpha9),
        VK_A => Some(Key::A),
        VK_B => Some(Key::B),
        VK_C => Some(Key::C),
        VK_D => Some(Key::D),
        VK_E => Some(Key::E),
        VK_F => Some(Key::F),
        VK_G => Some(Key::G),
        VK_H => Some(Key::H),
        VK_I => Some(Key::I),
        VK_J => Some(Key::J),
        VK_K => Some(Key::K),
        VK_L => Some(Key::L),
        VK_M => Some(Key::M),
        VK_N => Some(Key::N),
        VK_O => Some(Key::O),
        VK_P => Some(Key::P),
        VK_Q => Some(Key::Q),
        VK_R => Some(Key::R),
        VK_S => Some(Key::S),
        VK_T => Some(Key::T),
        VK_U => Some(Key::U),
        VK_V => Some(Key::V),
        VK_W => Some(Key::W),
        VK_X => Some(Key::X),
        VK_Y => Some(Key::Y),
        VK_Z => Some(Key::Z),
        VK_F1 => Some(Key::F1),
        VK_F2 => Some(Key::F2),
        VK_F3 => Some(Key::F3),
        VK_F4 => Some(Key::F4),
        VK_F5 => Some(Key::F5),
        VK_F6 => Some(Key::F6),
        VK_F7 => Some(Key::F7),
        VK_F8 => Some(Key::F8),
        VK_F9 => Some(Key::F9),
        VK_F10 => Some(Key::F10),
        VK_F11 => Some(Key::F11),
        VK_F12 => Some(Key::F12),
        _ => None,
    }
}

fn handle_key_modifier(io: &mut imgui::Io, key: VIRTUAL_KEY, down: bool) {
    if key == VK_LSHIFT || key == VK_RSHIFT {
        io.add_key_event(imgui::Key::ModShift, down);
    } else if key == VK_LCONTROL || key == VK_CONTROL {
        io.add_key_event(imgui::Key::ModCtrl, down);
    } else if key == VK_MENU || key == VK_LMENU || key == VK_RMENU {
        io.add_key_event(imgui::Key::ModAlt, down);
    } else if key == VK_LWIN || key == VK_RWIN {
        io.add_key_event(imgui::Key::ModSuper, down);
    }
}

const VK_KEY_MAX: usize = 256;
#[derive(Debug, Default)]
struct InputSystem {
    key_states: Vec<bool>,
}

impl InputSystem {
    pub fn new() -> Self {
        Self {
            key_states: vec![false; VK_KEY_MAX],
        }
    }

    pub fn update(&mut self, window: &glutin::window::Window, io: &mut imgui::Io) {
        for vkey in 0..VK_KEY_MAX {
            let key_state = unsafe { GetAsyncKeyState(vkey as i32) as u16 };
            let pressed = (key_state & 0x8000) > 0;
            if self.key_states[vkey] == pressed {
                continue;
            }

            self.key_states[vkey] = pressed;
            let vkey = VIRTUAL_KEY(vkey as u16);

            handle_key_modifier(io, vkey, pressed);
            let mouse_button = match vkey {
                VK_LBUTTON => Some(MouseButton::Left),
                VK_RBUTTON => Some(MouseButton::Right),
                VK_MBUTTON => Some(MouseButton::Middle),
                VK_XBUTTON1 => Some(MouseButton::Extra1),
                VK_XBUTTON2 => Some(MouseButton::Extra2),
                _ => None,
            };

            if let Some(button) = mouse_button {
                io.add_mouse_button_event(button, pressed);
            } else if let Some(key) = to_imgui_key(vkey) {
                // log::trace!("Key toogle {:?}: {}", key, pressed);
                io.add_key_event(key, pressed);
            } else {
                log::trace!("Missing ImGui key for {:?}", vkey);
            }
        }

        let mut point: POINT = Default::default();
        unsafe {
            GetCursorPos(&mut point);
            ScreenToClient(HWND(window.hwnd()), &mut point);
        };
        io.add_mouse_pos_event([
            (point.x as f64 / window.scale_factor()) as f32,
            (point.y as f64 / window.scale_factor()) as f32,
        ]);
    }
}

impl System {
    pub fn main_loop<F: FnMut(&mut bool, &Window, &mut Ui) + 'static>(self, mut run_ui: F) {
        let System {
            event_loop,
            display,
            mut imgui,
            mut platform,
            mut renderer,
            ..
        } = self;
        let mut last_frame = Instant::now();

        let mut ui_active = false;
        let mut input_system = InputSystem::new();

        event_loop.run(move |event, _, control_flow| match event {
            Event::NewEvents(_) => {
                let now = Instant::now();
                imgui.io_mut().update_delta_time(now - last_frame);
                last_frame = now;
            }
            Event::MainEventsCleared => {
                let gl_window = display.gl_window();
                platform
                    .prepare_frame(imgui.io_mut(), gl_window.window())
                    .expect("Failed to prepare frame");

                input_system.update(gl_window.window(), imgui.io_mut());
                unsafe {
                    let io = imgui.io();

                    let window_active = io.want_capture_mouse | io.want_capture_keyboard;
                    if window_active != ui_active {
                        ui_active = window_active;

                        let hwnd = HWND(gl_window.window().hwnd());
                        let mut style = GetWindowLongPtrA(hwnd, GWL_EXSTYLE);
                        if window_active {
                            style &= !(WS_EX_NOACTIVATE.0 as isize | WS_EX_TRANSPARENT.0 as isize);
                        } else {
                            style |= WS_EX_NOACTIVATE.0 as isize | WS_EX_TRANSPARENT.0 as isize;
                        }

                        log::debug!("Set UI active: {ui_active}");
                        SetWindowLongPtrA(hwnd, GWL_EXSTYLE, style);
                        if ui_active {
                            SetActiveWindow(hwnd);
                        }
                    }
                }

                gl_window.window().request_redraw();
            }
            Event::RedrawRequested(_) => {
                let gl_window = display.gl_window();
                let ui = imgui.frame();

                let mut run = true;
                run_ui(&mut run, gl_window.window(), ui);
                if !run {
                    *control_flow = ControlFlow::Exit;
                }

                let mut target = display.draw();
                target.clear_all((0.0, 0.0, 0.0, 0.0), 0.0, 0);
                platform.prepare_render(ui, gl_window.window());

                let draw_data = imgui.render();
                renderer
                    .render(&mut target, draw_data)
                    .expect("Rendering failed");
                target.finish().expect("Failed to swap buffers");
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            event => {
                let gl_window = display.gl_window();
                platform.handle_event(imgui.io_mut(), gl_window.window(), &event);
            }
        })
    }
}
