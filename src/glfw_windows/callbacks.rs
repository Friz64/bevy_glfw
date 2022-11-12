use bevy::{
    input::{keyboard::KeyboardInput, mouse::MouseButtonInput, ButtonState},
    math::DVec2,
    prelude::*,
    window::WindowId,
};
use core::slice;
use glfw_bindgen::*;
use std::{
    ffi::{c_char, c_int, c_uint, CStr},
    path::PathBuf,
};

#[derive(Default)]
pub struct CallbackMetadata {
    pub window_id: WindowId,
    pub events: Vec<GlfwEvent>,
}

impl CallbackMetadata {
    pub unsafe fn register(window: *mut GLFWwindow, window_id: WindowId) {
        let callback_metadata = CallbackMetadata {
            window_id,
            ..Default::default()
        };

        glfwSetWindowUserPointer(window, Box::into_raw(Box::new(callback_metadata)).cast());
        glfwSetWindowCloseCallback(window, Some(windowclose));
        glfwSetWindowFocusCallback(window, Some(windowfocus));
        glfwSetWindowContentScaleCallback(window, Some(windowcontentscale));
        glfwSetWindowPosCallback(window, Some(windowpos));
        glfwSetWindowSizeCallback(window, Some(windowsize));
        glfwSetFramebufferSizeCallback(window, Some(framebuffersize));
        glfwSetDropCallback(window, Some(drop_));
        glfwSetScrollCallback(window, Some(scroll));
        glfwSetCursorEnterCallback(window, Some(cursorenter));
        glfwSetCursorPosCallback(window, Some(cursorpos));
        glfwSetMouseButtonCallback(window, Some(mousebutton));
        glfwSetCharCallback(window, Some(char_));
        glfwSetKeyCallback(window, Some(key));
    }
}

#[derive(Debug)]
pub enum GlfwEvent {
    WindowClose,
    WindowFocused(bool),
    WindowContentScale(f64),
    WindowPos(IVec2),
    WindowSize(UVec2),
    FramebufferSize(UVec2),
    Drop(FileDragAndDrop),
    Scroll(Vec2),
    CursorEnter(bool),
    CursorPos(DVec2),
    MouseButton(MouseButtonInput),
    Char(char),
    Key(KeyboardInput),
}

pub unsafe extern "C" fn windowclose(window: *mut GLFWwindow) {
    (*glfwGetWindowUserPointer(window).cast::<CallbackMetadata>())
        .events
        .push(GlfwEvent::WindowClose);
}

pub unsafe extern "C" fn windowfocus(window: *mut GLFWwindow, focused: c_int) {
    (*glfwGetWindowUserPointer(window).cast::<CallbackMetadata>())
        .events
        .push(GlfwEvent::WindowFocused(focused == GLFW_TRUE as c_int));
}

pub unsafe extern "C" fn windowcontentscale(window: *mut GLFWwindow, xscale: f32, yscale: f32) {
    assert_eq!(xscale, yscale);

    (*glfwGetWindowUserPointer(window).cast::<CallbackMetadata>())
        .events
        .push(GlfwEvent::WindowContentScale(xscale as f64));
}

pub unsafe extern "C" fn windowpos(window: *mut GLFWwindow, xpos: c_int, ypos: c_int) {
    (*glfwGetWindowUserPointer(window).cast::<CallbackMetadata>())
        .events
        .push(GlfwEvent::WindowPos(IVec2::new(xpos, ypos)));
}

pub unsafe extern "C" fn windowsize(window: *mut GLFWwindow, width: c_int, height: c_int) {
    (*glfwGetWindowUserPointer(window).cast::<CallbackMetadata>())
        .events
        .push(GlfwEvent::WindowSize(UVec2::new(width as _, height as _)));
}

pub unsafe extern "C" fn framebuffersize(window: *mut GLFWwindow, width: c_int, height: c_int) {
    (*glfwGetWindowUserPointer(window).cast::<CallbackMetadata>())
        .events
        .push(GlfwEvent::FramebufferSize(UVec2::new(
            width as _,
            height as _,
        )));
}

pub unsafe extern "C" fn drop_(window: *mut GLFWwindow, count: c_int, paths: *mut *const c_char) {
    let metadata = glfwGetWindowUserPointer(window).cast::<CallbackMetadata>();
    let raw_paths = slice::from_raw_parts(paths, count as _);
    (*metadata)
        .events
        .extend(raw_paths.iter().filter_map(|&path| {
            let string_path = CStr::from_ptr(path).to_owned().into_string();
            if let Err(err) = &string_path {
                error!("File drop failed: {}", err);
            }

            Some(GlfwEvent::Drop(FileDragAndDrop::DroppedFile {
                id: (*metadata).window_id,
                path_buf: PathBuf::from(string_path.ok()?),
            }))
        }));
}

pub unsafe extern "C" fn scroll(window: *mut GLFWwindow, xoffset: f64, yoffset: f64) {
    (*glfwGetWindowUserPointer(window).cast::<CallbackMetadata>())
        .events
        .push(GlfwEvent::Scroll(Vec2::new(xoffset as _, yoffset as _)));
}

pub unsafe extern "C" fn cursorenter(window: *mut GLFWwindow, entered: c_int) {
    (*glfwGetWindowUserPointer(window).cast::<CallbackMetadata>())
        .events
        .push(GlfwEvent::CursorEnter(entered == GLFW_TRUE as c_int));
}

pub unsafe extern "C" fn cursorpos(window: *mut GLFWwindow, xpos: f64, ypos: f64) {
    (*glfwGetWindowUserPointer(window).cast::<CallbackMetadata>())
        .events
        .push(GlfwEvent::CursorPos(DVec2::new(xpos, ypos)));
}

pub unsafe extern "C" fn mousebutton(
    window: *mut GLFWwindow,
    button: c_int,
    action: c_int,
    _mods: c_int,
) {
    (*glfwGetWindowUserPointer(window).cast::<CallbackMetadata>())
        .events
        .push(GlfwEvent::MouseButton(MouseButtonInput {
            button: match button + 1 {
                1 => MouseButton::Left,
                2 => MouseButton::Right,
                3 => MouseButton::Middle,
                other => MouseButton::Other(other as u16),
            },
            state: match action as u32 {
                GLFW_PRESS => ButtonState::Pressed,
                GLFW_RELEASE => ButtonState::Released,
                _ => unreachable!(),
            },
        }));
}

pub unsafe extern "C" fn char_(window: *mut GLFWwindow, codepoint: c_uint) {
    if let Some(c) = char::from_u32(codepoint) {
        (*glfwGetWindowUserPointer(window).cast::<CallbackMetadata>())
            .events
            .push(GlfwEvent::Char(c));
    }
}

pub unsafe extern "C" fn key(
    window: *mut GLFWwindow,
    key: c_int,
    scancode: c_int,
    action: c_int,
    _mods: c_int,
) {
    (*glfwGetWindowUserPointer(window).cast::<CallbackMetadata>())
        .events
        .push(GlfwEvent::Key(KeyboardInput {
            scan_code: scancode as _,
            key_code: match key as u32 {
                GLFW_KEY_SPACE => Some(KeyCode::Space),
                GLFW_KEY_APOSTROPHE => Some(KeyCode::Apostrophe),
                GLFW_KEY_COMMA => Some(KeyCode::Comma),
                GLFW_KEY_MINUS => Some(KeyCode::Minus),
                GLFW_KEY_PERIOD => Some(KeyCode::Period),
                GLFW_KEY_SLASH => Some(KeyCode::Slash),
                GLFW_KEY_0 => Some(KeyCode::Key0),
                GLFW_KEY_1 => Some(KeyCode::Key1),
                GLFW_KEY_2 => Some(KeyCode::Key2),
                GLFW_KEY_3 => Some(KeyCode::Key3),
                GLFW_KEY_4 => Some(KeyCode::Key4),
                GLFW_KEY_5 => Some(KeyCode::Key5),
                GLFW_KEY_6 => Some(KeyCode::Key6),
                GLFW_KEY_7 => Some(KeyCode::Key7),
                GLFW_KEY_8 => Some(KeyCode::Key8),
                GLFW_KEY_9 => Some(KeyCode::Key9),
                GLFW_KEY_SEMICOLON => Some(KeyCode::Semicolon),
                GLFW_KEY_EQUAL => Some(KeyCode::Equals),
                GLFW_KEY_A => Some(KeyCode::A),
                GLFW_KEY_B => Some(KeyCode::B),
                GLFW_KEY_C => Some(KeyCode::C),
                GLFW_KEY_D => Some(KeyCode::D),
                GLFW_KEY_E => Some(KeyCode::E),
                GLFW_KEY_F => Some(KeyCode::F),
                GLFW_KEY_G => Some(KeyCode::G),
                GLFW_KEY_H => Some(KeyCode::H),
                GLFW_KEY_I => Some(KeyCode::I),
                GLFW_KEY_J => Some(KeyCode::J),
                GLFW_KEY_K => Some(KeyCode::K),
                GLFW_KEY_L => Some(KeyCode::L),
                GLFW_KEY_M => Some(KeyCode::M),
                GLFW_KEY_N => Some(KeyCode::N),
                GLFW_KEY_O => Some(KeyCode::O),
                GLFW_KEY_P => Some(KeyCode::P),
                GLFW_KEY_Q => Some(KeyCode::Q),
                GLFW_KEY_R => Some(KeyCode::R),
                GLFW_KEY_S => Some(KeyCode::S),
                GLFW_KEY_T => Some(KeyCode::T),
                GLFW_KEY_U => Some(KeyCode::U),
                GLFW_KEY_V => Some(KeyCode::V),
                GLFW_KEY_W => Some(KeyCode::W),
                GLFW_KEY_X => Some(KeyCode::X),
                GLFW_KEY_Y => Some(KeyCode::Y),
                GLFW_KEY_Z => Some(KeyCode::Z),
                GLFW_KEY_LEFT_BRACKET => Some(KeyCode::LBracket),
                GLFW_KEY_BACKSLASH => Some(KeyCode::Backslash),
                GLFW_KEY_RIGHT_BRACKET => Some(KeyCode::RBracket),
                GLFW_KEY_GRAVE_ACCENT => Some(KeyCode::Grave),
                GLFW_KEY_ESCAPE => Some(KeyCode::Escape),
                GLFW_KEY_ENTER => Some(KeyCode::Return),
                GLFW_KEY_TAB => Some(KeyCode::Tab),
                GLFW_KEY_BACKSPACE => Some(KeyCode::Back),
                GLFW_KEY_INSERT => Some(KeyCode::Insert),
                GLFW_KEY_DELETE => Some(KeyCode::Delete),
                GLFW_KEY_RIGHT => Some(KeyCode::Right),
                GLFW_KEY_LEFT => Some(KeyCode::Left),
                GLFW_KEY_DOWN => Some(KeyCode::Down),
                GLFW_KEY_UP => Some(KeyCode::Up),
                GLFW_KEY_PAGE_UP => Some(KeyCode::PageUp),
                GLFW_KEY_PAGE_DOWN => Some(KeyCode::PageDown),
                GLFW_KEY_HOME => Some(KeyCode::Home),
                GLFW_KEY_END => Some(KeyCode::End),
                GLFW_KEY_SCROLL_LOCK => Some(KeyCode::Scroll),
                GLFW_KEY_NUM_LOCK => Some(KeyCode::Numlock),
                GLFW_KEY_PRINT_SCREEN => Some(KeyCode::Snapshot),
                GLFW_KEY_PAUSE => Some(KeyCode::Pause),
                GLFW_KEY_F1 => Some(KeyCode::F1),
                GLFW_KEY_F2 => Some(KeyCode::F2),
                GLFW_KEY_F3 => Some(KeyCode::F3),
                GLFW_KEY_F4 => Some(KeyCode::F4),
                GLFW_KEY_F5 => Some(KeyCode::F5),
                GLFW_KEY_F6 => Some(KeyCode::F6),
                GLFW_KEY_F7 => Some(KeyCode::F7),
                GLFW_KEY_F8 => Some(KeyCode::F8),
                GLFW_KEY_F9 => Some(KeyCode::F9),
                GLFW_KEY_F10 => Some(KeyCode::F10),
                GLFW_KEY_F11 => Some(KeyCode::F11),
                GLFW_KEY_F12 => Some(KeyCode::F12),
                GLFW_KEY_F13 => Some(KeyCode::F13),
                GLFW_KEY_F14 => Some(KeyCode::F14),
                GLFW_KEY_F15 => Some(KeyCode::F15),
                GLFW_KEY_F16 => Some(KeyCode::F16),
                GLFW_KEY_F17 => Some(KeyCode::F17),
                GLFW_KEY_F18 => Some(KeyCode::F18),
                GLFW_KEY_F19 => Some(KeyCode::F19),
                GLFW_KEY_F20 => Some(KeyCode::F20),
                GLFW_KEY_F21 => Some(KeyCode::F21),
                GLFW_KEY_F22 => Some(KeyCode::F22),
                GLFW_KEY_F23 => Some(KeyCode::F23),
                GLFW_KEY_F24 => Some(KeyCode::F24),
                GLFW_KEY_KP_0 => Some(KeyCode::Numpad0),
                GLFW_KEY_KP_1 => Some(KeyCode::Numpad1),
                GLFW_KEY_KP_2 => Some(KeyCode::Numpad2),
                GLFW_KEY_KP_3 => Some(KeyCode::Numpad3),
                GLFW_KEY_KP_4 => Some(KeyCode::Numpad4),
                GLFW_KEY_KP_5 => Some(KeyCode::Numpad5),
                GLFW_KEY_KP_6 => Some(KeyCode::Numpad6),
                GLFW_KEY_KP_7 => Some(KeyCode::Numpad7),
                GLFW_KEY_KP_8 => Some(KeyCode::Numpad8),
                GLFW_KEY_KP_9 => Some(KeyCode::Numpad9),
                GLFW_KEY_KP_DECIMAL => Some(KeyCode::NumpadDecimal),
                GLFW_KEY_KP_DIVIDE => Some(KeyCode::NumpadDivide),
                GLFW_KEY_KP_MULTIPLY => Some(KeyCode::NumpadMultiply),
                GLFW_KEY_KP_SUBTRACT => Some(KeyCode::NumpadSubtract),
                GLFW_KEY_KP_ADD => Some(KeyCode::NumpadAdd),
                GLFW_KEY_KP_ENTER => Some(KeyCode::NumpadEnter),
                GLFW_KEY_KP_EQUAL => Some(KeyCode::NumpadEquals),
                GLFW_KEY_LEFT_SHIFT => Some(KeyCode::LShift),
                GLFW_KEY_LEFT_CONTROL => Some(KeyCode::LControl),
                GLFW_KEY_LEFT_ALT => Some(KeyCode::LAlt),
                GLFW_KEY_LEFT_SUPER => Some(KeyCode::LWin),
                GLFW_KEY_RIGHT_SHIFT => Some(KeyCode::RShift),
                GLFW_KEY_RIGHT_CONTROL => Some(KeyCode::RControl),
                GLFW_KEY_RIGHT_ALT => Some(KeyCode::RAlt),
                GLFW_KEY_RIGHT_SUPER => Some(KeyCode::RWin),
                _ => None,
            },
            state: match action as u32 {
                GLFW_PRESS | GLFW_REPEAT => ButtonState::Pressed,
                GLFW_RELEASE => ButtonState::Released,
                _ => unreachable!(),
            },
        }));
}
