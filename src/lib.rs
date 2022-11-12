#![doc = include_str!("../README.md")]
mod glfw_windows;

use bevy::{
    app::AppExit,
    ecs::event::ManualEventReader,
    prelude::*,
    window::{
        CreateWindow, ModifiesWindows, WindowClosed, WindowCommand, WindowCreated, WindowMode,
    },
};
use glfw_bindgen::*;
use glfw_windows::GlfwWindows;
use std::{
    ffi::{c_char, c_int, CStr, CString},
    ptr,
};

unsafe extern "C" fn glfw_error_callback(error_code: c_int, description: *const c_char) {
    let description = CStr::from_ptr(description).to_string_lossy();
    debug!("GLFW error {error_code}: {description}");
}

#[derive(Default)]
pub struct GlfwPlugin;

impl Plugin for GlfwPlugin {
    fn build(&self, app: &mut App) {
        assert!(
            !app.world.contains_resource::<GlfwWindows>(),
            "GlfwPlugin added multiple times"
        );

        unsafe {
            glfwSetErrorCallback(Some(glfw_error_callback));
            assert_eq!(glfwInit(), GLFW_TRUE as c_int);
        }

        app.init_non_send_resource::<GlfwWindows>()
            .set_runner(glfw_runner)
            .add_system_to_stage(CoreStage::PostUpdate, change_window.label(ModifiesWindows));

        handle_create_window_events(&mut app.world);
    }
}

fn change_window(
    mut glfw_windows: NonSendMut<GlfwWindows>,
    mut windows: ResMut<Windows>,
    mut window_close_events: EventWriter<WindowClosed>,
) {
    let mut removed_windows = vec![];
    for bevy_window in windows.iter_mut() {
        let id = bevy_window.id();
        let cursors = glfw_windows.cursors.clone();
        let window = glfw_windows.windows.get_mut(&id).unwrap();
        for command in bevy_window.drain_commands() {
            match command {
                WindowCommand::SetWindowMode { mode, resolution } => unsafe {
                    window.set_window_mode(mode, resolution);
                },
                WindowCommand::SetTitle { title } => unsafe {
                    let title = CString::new(title.as_str()).expect("Invalid window title");
                    glfwSetWindowTitle(window.window, title.as_ptr());
                },
                WindowCommand::SetScaleFactor { .. } => {
                    panic!("Manually changing window scale factor not supported");
                }
                WindowCommand::SetResolution {
                    logical_resolution, ..
                } => unsafe {
                    window.set_size(UVec2::new(
                        logical_resolution.x as _,
                        logical_resolution.y as _,
                    ));
                },
                WindowCommand::SetPresentMode { .. } => (),
                WindowCommand::SetResizable { resizable } => unsafe {
                    glfwSetWindowAttrib(window.window, GLFW_RESIZABLE as _, resizable as _);
                },
                WindowCommand::SetDecorations { decorations } => unsafe {
                    glfwSetWindowAttrib(window.window, GLFW_DECORATED as _, decorations as _);
                },
                WindowCommand::SetCursorLockMode { locked } => {
                    window.cursor_locked = locked;
                    unsafe { window.update_cursor_mode() };
                }
                WindowCommand::SetCursorIcon { icon } => {
                    let cursor = match icon {
                        CursorIcon::Crosshair => cursors.crosshair,
                        CursorIcon::Hand => cursors.pointing_hand,
                        CursorIcon::Arrow => cursors.arrow,
                        CursorIcon::Text => cursors.ibeam,
                        CursorIcon::NotAllowed => cursors.resize_all,
                        CursorIcon::EResize => cursors.resize_ew,
                        CursorIcon::NResize => cursors.resize_ns,
                        CursorIcon::NeResize => cursors.resize_nesw,
                        CursorIcon::NwResize => cursors.resize_nwse,
                        CursorIcon::SResize => cursors.resize_ns,
                        CursorIcon::SeResize => cursors.resize_nwse,
                        CursorIcon::SwResize => cursors.resize_nesw,
                        CursorIcon::WResize => cursors.resize_ew,
                        CursorIcon::EwResize => cursors.resize_ew,
                        CursorIcon::NsResize => cursors.resize_ns,
                        CursorIcon::NeswResize => cursors.resize_nesw,
                        CursorIcon::NwseResize => cursors.resize_nwse,
                        _ => ptr::null_mut(), // use default cursor
                    };

                    unsafe { glfwSetCursor(window.window, cursor) };
                }
                WindowCommand::SetCursorVisibility { visible } => {
                    window.cursor_visible = visible;
                    unsafe { window.update_cursor_mode() };
                }
                WindowCommand::SetCursorPosition { position } => unsafe {
                    glfwSetCursorPos(
                        window.window,
                        position.x as f64,
                        window.size.y as f64 - position.y as f64,
                    );
                },
                WindowCommand::SetMaximized { maximized } => unsafe {
                    if maximized {
                        glfwMaximizeWindow(window.window);
                    } else {
                        glfwRestoreWindow(window.window);
                    }
                },
                WindowCommand::SetMinimized { minimized } => unsafe {
                    if minimized {
                        glfwIconifyWindow(window.window);
                    } else {
                        glfwRestoreWindow(window.window);
                    }
                },
                WindowCommand::SetPosition { position } => unsafe {
                    window.set_pos(position);
                },
                WindowCommand::Center(monitor) => unsafe {
                    window.center_to(monitor);
                },
                WindowCommand::SetResizeConstraints { resize_constraints } => unsafe {
                    window.set_resize_constraints(resize_constraints);
                },
                WindowCommand::Close => {
                    removed_windows.push(id);
                    break;
                }
            }
        }
    }

    for id in removed_windows {
        if windows.remove(id).is_some() {
            window_close_events.send(WindowClosed { id });
            let mut window = glfw_windows.windows.remove(&id).unwrap();
            if !glfw_windows.windows.is_empty() {
                unsafe {
                    // glfwHideWindow only works on windowed windows
                    window.set_window_mode(WindowMode::Windowed, UVec2::default());
                    // HACK: glfwDestroyWindow may cause vkDestroySwapchain to deadlock
                    glfwHideWindow(window.window);
                }
            }
        }
    }
}

pub fn glfw_runner(mut app: App) {
    let mut app_exit_event_reader = ManualEventReader::<AppExit>::default();
    loop {
        {
            unsafe { glfwPollEvents() };
            let world = app.world.cell();
            let mut glfw_windows = world.non_send_resource_mut::<GlfwWindows>();
            for (window_id, window) in glfw_windows.windows.iter_mut() {
                let mut windows = world.resource_mut::<Windows>();
                let bevy_window = windows.get_mut(*window_id).unwrap();
                unsafe { window.handle_events(&world, bevy_window) };
            }
        }

        app.update();

        if let Some(app_exit_events) = app.world.get_resource::<Events<AppExit>>() {
            if app_exit_event_reader.iter(app_exit_events).last().is_some() {
                break;
            }
        }

        handle_create_window_events(&mut app.world);
    }

    drop(app);
    unsafe { glfwTerminate() };
}

fn handle_create_window_events(world: &mut World) {
    let world = world.cell();
    for create_window_event in world
        .get_resource_mut::<Events<CreateWindow>>()
        .unwrap()
        .drain()
    {
        let mut windows = world.get_resource_mut::<Windows>().unwrap();
        let mut window_created_events = world.get_resource_mut::<Events<WindowCreated>>().unwrap();
        let window = world
            .non_send_resource_mut::<GlfwWindows>()
            .create_window(create_window_event.id, &create_window_event.descriptor);
        window_created_events.send(WindowCreated { id: window.id() });
        windows.add(window);
    }
}
