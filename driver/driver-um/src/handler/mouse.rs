use core::slice;
use std::mem;

use valthrun_driver_protocol::command::{
    DriverCommandInputMouse,
    MouseState,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput,
    INPUT,
    INPUT_MOUSE,
    MOUSEEVENTF_HWHEEL,
    MOUSEEVENTF_LEFTDOWN,
    MOUSEEVENTF_LEFTUP,
    MOUSEEVENTF_MIDDLEDOWN,
    MOUSEEVENTF_MIDDLEUP,
    MOUSEEVENTF_MOVE,
    MOUSEEVENTF_RIGHTDOWN,
    MOUSEEVENTF_RIGHTUP,
    MOUSEEVENTF_WHEEL,
    MOUSEEVENTF_XDOWN,
    MOUSEEVENTF_XUP,
};

pub fn mouse_move(command: &mut DriverCommandInputMouse) -> anyhow::Result<()> {
    let states = unsafe { slice::from_raw_parts(command.buffer, command.state_count) };
    let inputs = states.iter().map(mouse_state_to_input).collect::<Vec<_>>();

    unsafe { SendInput(&inputs, mem::size_of::<INPUT>() as i32) };
    Ok(())
}

fn mouse_state_to_input(state: &MouseState) -> INPUT {
    let mut input_data: INPUT = Default::default();
    input_data.r#type = INPUT_MOUSE;

    let mi = unsafe { &mut input_data.Anonymous.mi };

    if let Some(state) = &state.buttons[0] {
        mi.dwFlags |= if *state {
            MOUSEEVENTF_LEFTDOWN
        } else {
            MOUSEEVENTF_LEFTUP
        };
    }
    if let Some(state) = &state.buttons[1] {
        mi.dwFlags |= if *state {
            MOUSEEVENTF_RIGHTDOWN
        } else {
            MOUSEEVENTF_RIGHTUP
        };
    }
    if let Some(state) = &state.buttons[2] {
        mi.dwFlags |= if *state {
            MOUSEEVENTF_MIDDLEDOWN
        } else {
            MOUSEEVENTF_MIDDLEUP
        };
    }
    if let Some(state) = &state.buttons[3] {
        mi.dwFlags |= if *state {
            MOUSEEVENTF_XDOWN
        } else {
            MOUSEEVENTF_XUP
        };
    }
    if let Some(_state) = &state.buttons[4] { /* not supported :() */ }
    if state.wheel {
        mi.dwFlags |= MOUSEEVENTF_WHEEL;
    }
    if state.hwheel {
        mi.dwFlags |= MOUSEEVENTF_HWHEEL;
    }

    mi.dwFlags |= MOUSEEVENTF_MOVE;
    mi.dx = state.last_x;
    mi.dy = state.last_y;

    input_data
}
