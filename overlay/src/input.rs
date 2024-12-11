use imgui::{
    Key,
    MouseButton,
};
use imgui_winit_support::winit::window::Window;
use windows::Win32::{
    Foundation::{
        HWND,
        POINT,
    },
    Graphics::Gdi::ScreenToClient,
    UI::{
        Input::KeyboardAndMouse::{
            GetAsyncKeyState,
            VIRTUAL_KEY,
            VK_CONTROL,
            VK_LBUTTON,
            VK_LCONTROL,
            VK_LMENU,
            VK_LSHIFT,
            VK_LWIN,
            VK_MBUTTON,
            VK_MENU,
            VK_RBUTTON,
            VK_RMENU,
            VK_RSHIFT,
            VK_RWIN,
            VK_XBUTTON1,
            VK_XBUTTON2,
        },
        WindowsAndMessaging::GetCursorPos,
    },
};

const VK_KEY_MAX: usize = 256;

#[derive(Debug, Default)]
pub struct MouseInputSystem {
    hwnd: HWND,
}
impl MouseInputSystem {
    pub fn new(hwnd: HWND) -> Self {
        Self { hwnd }
    }

    pub fn update(&mut self, window: &Window, io: &mut imgui::Io) {
        let mut point: POINT = Default::default();
        unsafe {
            GetCursorPos(&mut point);
            ScreenToClient(self.hwnd, &mut point);
        };

        io.add_mouse_pos_event([
            (point.x as f64 / window.scale_factor()) as f32,
            (point.y as f64 / window.scale_factor()) as f32,
        ]);
    }
}

/// Simple input system using the global mouse / keyboard state.
/// This does not require the need to process window messages or the imgui overlay to be active.
#[derive(Debug, Default)]
#[allow(unused)]
pub struct KeyboardInputSystem {
    key_states: Vec<bool>,
}

#[allow(unused)]
impl KeyboardInputSystem {
    pub fn new() -> Self {
        Self {
            key_states: vec![false; VK_KEY_MAX],
        }
    }

    pub fn update(&mut self, _window: &Window, io: &mut imgui::Io) {
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
