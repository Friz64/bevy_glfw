mod callbacks;

use self::callbacks::{CallbackMetadata, GlfwEvent};
use bevy::{
    ecs::world::WorldCell,
    input::{
        keyboard::KeyboardInput,
        mouse::{MouseButtonInput, MouseWheel},
    },
    prelude::*,
    utils::HashMap,
    window::{
        Window, WindowBackendScaleFactorChanged, WindowCloseRequested, WindowDescriptor,
        WindowFocused, WindowId, WindowMode, WindowResizeConstraints, WindowResized,
        WindowScaleFactorChanged,
    },
};
use core::slice;
use glfw_bindgen::*;
use raw_window_handle::RawWindowHandle;
use std::{
    ffi::{c_int, CString},
    ptr,
};

#[derive(Default)]
pub struct GlfwWindows {
    pub windows: HashMap<WindowId, GlfwWindow>,
    pub cursors: GlfwStandardCursors,
}

impl GlfwWindows {
    pub fn create_window(
        &mut self,
        window_id: WindowId,
        window_descriptor: &WindowDescriptor,
    ) -> Window {
        let window = unsafe { GlfwWindow::new(window_id, window_descriptor) };
        let bevy_window = Window::new(
            window_id,
            window_descriptor,
            window.size.x,
            window.size.y,
            window.scale_factor,
            Some(window.pos),
            unsafe { window.raw_window_handle() },
        );

        self.windows.insert(window_id, window);
        bevy_window
    }
}

pub struct GlfwWindow {
    pub window: *mut GLFWwindow,
    pub pos: IVec2,
    pub size: UVec2,
    pub pre_fullscreen_pos: IVec2,
    pub pre_fullscreen_size: UVec2,
    pub scale_factor: f64,
    pub cursor_visible: bool,
    pub cursor_locked: bool,
}

impl GlfwWindow {
    unsafe fn new(
        window_id: WindowId,
        WindowDescriptor {
            width,
            height,
            position,
            resize_constraints,
            scale_factor_override,
            title,
            present_mode: _,
            resizable,
            decorations,
            cursor_visible,
            cursor_locked,
            mode,
            transparent,
            canvas: _,
            fit_canvas_to_parent: _,
        }: &WindowDescriptor,
    ) -> GlfwWindow {
        glfwWindowHint(GLFW_CLIENT_API as _, GLFW_NO_API as _);
        glfwWindowHint(GLFW_RESIZABLE as _, *resizable as _);
        glfwWindowHint(GLFW_DECORATED as _, *decorations as _);
        glfwWindowHint(GLFW_TRANSPARENT_FRAMEBUFFER as _, *transparent as _);

        assert_eq!(
            *scale_factor_override, None,
            "Overriding scale factor not supported"
        );

        let title = CString::new(title.as_str()).expect("Invalid window title");
        let window = glfwCreateWindow(
            *width as c_int,
            *height as c_int,
            title.as_ptr(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert!(!window.is_null(), "Failed to create window");

        unsafe { CallbackMetadata::register(window, window_id) }

        let mut xpos = 0;
        let mut ypos = 0;
        unsafe { glfwGetWindowPos(window, &mut xpos, &mut ypos) }

        let mut width = 0;
        let mut height = 0;
        unsafe { glfwGetWindowSize(window, &mut width, &mut height) }

        let mut xscale = 0.;
        let mut yscale = 0.;
        unsafe { glfwGetWindowContentScale(window, &mut xscale, &mut yscale) }
        assert_eq!(xscale, yscale);

        let mut window = GlfwWindow {
            window,
            pos: IVec2::new(xpos, ypos),
            size: UVec2::new(width as u32, height as u32),
            pre_fullscreen_pos: IVec2::default(),
            pre_fullscreen_size: UVec2::default(),
            scale_factor: xscale as _,
            cursor_visible: *cursor_visible,
            cursor_locked: *cursor_locked,
        };

        match position {
            WindowPosition::Automatic => (),
            WindowPosition::Centered(monitor) => window.center_to(*monitor),
            WindowPosition::At(pos) => window.set_pos(pos.as_ivec2()),
        }

        window.set_resize_constraints(*resize_constraints);
        window.update_cursor_mode();
        window.set_window_mode(*mode, window.size);
        window
    }

    unsafe fn current_monitor(&self) -> *mut GLFWmonitor {
        // function doesn't work on wayland, as window position is always 0
        if glfwGetPlatform() == GLFW_PLATFORM_WAYLAND as c_int {
            return monitors()[0];
        }

        let window_left = self.pos.x;
        let window_top = self.pos.y;
        let window_right = window_left + self.size.x as i32;
        let window_bottom = window_top + self.size.y as i32;
        *monitors()
            .iter()
            // 1. the first monitor is guaranteed to be the primary monitor
            // 2. max_by_key returns the last element if all are equal
            // -> if the window is not on any monitor, the primary monitor is returned
            .rev()
            .max_by_key(|&&monitor| {
                let mut monitor_left = 0;
                let mut monitor_top = 0;
                glfwGetMonitorPos(monitor, &mut monitor_left, &mut monitor_top);
                let video_mode = glfwGetVideoMode(monitor);
                let monitor_right = monitor_left + (*video_mode).width;
                let monitor_bottom = monitor_top + (*video_mode).height;

                let visible_left = window_left.max(monitor_left).min(monitor_right);
                let visible_top = window_top.max(monitor_top).min(monitor_bottom);
                let visible_right = window_right.min(monitor_right).max(monitor_left);
                let visible_bottom = window_bottom.min(monitor_bottom).max(monitor_top);
                (visible_right - visible_left) * (visible_bottom - visible_top)
            })
            .unwrap()
    }

    pub unsafe fn set_window_mode(&mut self, mode: WindowMode, resolution: UVec2) {
        let set_windowed = matches!(mode, WindowMode::Windowed);
        let currently_windowed = glfwGetWindowMonitor(self.window).is_null();
        match (currently_windowed, set_windowed) {
            (true, true) => return,
            (true, false) => {
                self.pre_fullscreen_pos = self.pos;
                self.pre_fullscreen_size = self.size;
            }
            _ => (),
        }

        let target_monitor = self.current_monitor();
        let video_mode = glfwGetVideoMode(target_monitor);
        match mode {
            WindowMode::Windowed => {
                glfwSetWindowMonitor(
                    self.window,
                    ptr::null_mut(),
                    self.pre_fullscreen_pos.x,
                    self.pre_fullscreen_pos.y,
                    self.pre_fullscreen_size.x as _,
                    self.pre_fullscreen_size.y as _,
                    GLFW_DONT_CARE,
                );
            }
            WindowMode::SizedFullscreen => glfwSetWindowMonitor(
                self.window,
                target_monitor,
                0,
                0,
                resolution.x as _,
                resolution.y as _,
                GLFW_DONT_CARE,
            ),
            WindowMode::BorderlessFullscreen | WindowMode::Fullscreen => glfwSetWindowMonitor(
                self.window,
                target_monitor,
                0,
                0,
                (*video_mode).width,
                (*video_mode).height,
                GLFW_DONT_CARE,
            ),
        }
    }

    pub unsafe fn set_pos(&mut self, pos: IVec2) {
        self.pos = pos;
        glfwSetWindowPos(self.window, pos.x, pos.y);
    }

    pub unsafe fn set_size(&mut self, size: UVec2) {
        self.size = size;
        glfwSetWindowSize(self.window, size.x as _, size.y as _);
    }

    pub unsafe fn center_to(&mut self, monitor_selection: MonitorSelection) {
        let monitor = match monitor_selection {
            MonitorSelection::Current => self.current_monitor(),
            MonitorSelection::Primary => monitors()[0],
            MonitorSelection::Number(idx) => {
                *monitors().get(idx).expect("No monitor at specified index")
            }
        };

        let mut monitor_x = 0;
        let mut monitor_y = 0;
        glfwGetMonitorPos(monitor, &mut monitor_x, &mut monitor_y);
        let video_mode = glfwGetVideoMode(monitor);
        let monitor_middle_x = monitor_x + (*video_mode).width / 2;
        let monitor_middle_y = monitor_y + (*video_mode).height / 2;

        self.set_pos(IVec2::new(
            monitor_middle_x - self.size.x as i32 / 2,
            monitor_middle_y - self.size.y as i32 / 2,
        ));
    }

    pub unsafe fn set_resize_constraints(&mut self, constraints: WindowResizeConstraints) {
        let conv = |dim: f32| {
            if dim.is_normal() {
                dim as c_int
            } else {
                GLFW_DONT_CARE
            }
        };

        glfwSetWindowSizeLimits(
            self.window,
            conv(constraints.min_width),
            conv(constraints.min_height),
            conv(constraints.max_width),
            conv(constraints.max_height),
        );
    }

    pub unsafe fn update_cursor_mode(&self) {
        glfwSetInputMode(
            self.window,
            GLFW_CURSOR as _,
            if self.cursor_locked {
                GLFW_CURSOR_DISABLED
            } else if self.cursor_visible {
                GLFW_CURSOR_NORMAL
            } else {
                GLFW_CURSOR_HIDDEN
            } as _,
        );

        if glfwRawMouseMotionSupported() == GLFW_TRUE as c_int {
            glfwSetInputMode(
                self.window,
                GLFW_RAW_MOUSE_MOTION as _,
                self.cursor_locked as _,
            );
        }
    }

    unsafe fn raw_window_handle(&self) -> RawWindowHandle {
        #[cfg(target_family = "windows")]
        'windows: {
            let hwnd = glfwGetWin32Window(self.window);
            if hwnd.is_null() {
                break 'windows;
            }

            let mut handle = raw_window_handle::Win32Handle::empty();
            handle.hwnd = hwnd as _;
            return RawWindowHandle::Win32(handle);
        }

        #[cfg(target_os = "macos")]
        'macos: {
            let ns_window: *mut objc::runtime::Object = glfwGetCocoaWindow(self.window) as *mut _;
            let ns_view: *mut objc::runtime::Object = objc::msg_send![ns_window, contentView];
            if ns_view.is_null() {
                break 'macos;
            }

            let mut handle = AppKitHandle::empty();
            handle.ns_window = ns_window as _;
            handle.ns_view = ns_view as _;
            return RawWindowHandle::AppKit(handle);
        }

        #[cfg(all(target_family = "unix", not(target_os = "macos")))]
        'unix: {
            'wayland: {
                let display = glfwGetWaylandDisplay();
                if display.is_null() {
                    break 'wayland;
                }

                let mut handle = raw_window_handle::WaylandHandle::empty();
                handle.display = glfwGetWaylandDisplay() as _;
                handle.surface = glfwGetWaylandWindow(self.window) as _;
                return RawWindowHandle::Wayland(handle);
            }

            let display = glfwGetX11Display();
            if display.is_null() {
                break 'unix;
            }

            let mut handle = raw_window_handle::XlibHandle::empty();
            handle.window = glfwGetX11Window(self.window);
            handle.display = display as _;
            return RawWindowHandle::Xlib(handle);
        }

        panic!("No window backend found")
    }

    pub unsafe fn handle_events(&mut self, world: &WorldCell, bevy_window: &mut Window) {
        let callback_metadata = glfwGetWindowUserPointer(self.window).cast::<CallbackMetadata>();
        let mut send_size = false;
        for event in (*callback_metadata).events.drain(..) {
            match event {
                GlfwEvent::WindowClose => world
                    .resource_mut::<Events<WindowCloseRequested>>()
                    .send(WindowCloseRequested {
                        id: (*callback_metadata).window_id,
                    }),
                GlfwEvent::WindowFocused(focused) => {
                    bevy_window.update_focused_status_from_backend(focused);
                    world
                        .resource_mut::<Events<WindowFocused>>()
                        .send(WindowFocused {
                            id: (*callback_metadata).window_id,
                            focused,
                        });
                }
                GlfwEvent::WindowContentScale(scale_factor) => {
                    self.scale_factor = scale_factor;
                    world
                        .resource_mut::<Events<WindowScaleFactorChanged>>()
                        .send(WindowScaleFactorChanged {
                            id: (*callback_metadata).window_id,
                            scale_factor,
                        });

                    bevy_window.update_scale_factor_from_backend(scale_factor);
                    world
                        .resource_mut::<Events<WindowBackendScaleFactorChanged>>()
                        .send(WindowBackendScaleFactorChanged {
                            id: (*callback_metadata).window_id,
                            scale_factor,
                        });

                    send_size = true;
                }
                GlfwEvent::WindowPos(position) => {
                    bevy_window.update_actual_position_from_backend(position);
                    self.pos = position;
                    world
                        .resource_mut::<Events<WindowMoved>>()
                        .send(WindowMoved {
                            id: (*callback_metadata).window_id,
                            position,
                        });
                }
                GlfwEvent::WindowSize(size) => {
                    self.size = size;
                    send_size = true;
                }
                GlfwEvent::FramebufferSize(size) => {
                    bevy_window.update_actual_size_from_backend(size.x, size.y);
                    send_size = true;
                }
                GlfwEvent::Drop(event) => {
                    world.resource_mut::<Events<FileDragAndDrop>>().send(event);
                }
                GlfwEvent::Scroll(scroll) => {
                    world.resource_mut::<Events<MouseWheel>>().send(MouseWheel {
                        unit: bevy::input::mouse::MouseScrollUnit::Line,
                        x: scroll.x,
                        y: scroll.y,
                    });
                }
                GlfwEvent::CursorEnter(entered) => {
                    if entered {
                        world
                            .resource_mut::<Events<CursorEntered>>()
                            .send(CursorEntered {
                                id: (*callback_metadata).window_id,
                            });
                    } else {
                        bevy_window.update_cursor_physical_position_from_backend(None);
                        world.resource_mut::<Events<CursorLeft>>().send(CursorLeft {
                            id: (*callback_metadata).window_id,
                        });
                    }
                }
                GlfwEvent::CursorPos(mut pos) => {
                    pos.y = self.size.y as f64 - pos.y; // convert top-left -> bottom-left origin
                    let physical = pos * self.scale_factor;
                    bevy_window.update_cursor_physical_position_from_backend(Some(physical));
                    world
                        .resource_mut::<Events<CursorMoved>>()
                        .send(CursorMoved {
                            id: (*callback_metadata).window_id,
                            position: pos.as_vec2(),
                        });
                }
                GlfwEvent::MouseButton(event) => {
                    world.resource_mut::<Events<MouseButtonInput>>().send(event);
                }
                GlfwEvent::Char(char) => {
                    world
                        .resource_mut::<Events<ReceivedCharacter>>()
                        .send(ReceivedCharacter {
                            id: (*callback_metadata).window_id,
                            char,
                        });
                }
                GlfwEvent::Key(event) => {
                    world.resource_mut::<Events<KeyboardInput>>().send(event);
                }
            }
        }

        if send_size {
            world
                .resource_mut::<Events<WindowResized>>()
                .send(WindowResized {
                    id: (*callback_metadata).window_id,
                    width: self.size.x as _,
                    height: self.size.y as _,
                });
        }
    }
}

#[derive(Clone)]
pub struct GlfwStandardCursors {
    pub arrow: *mut GLFWcursor,
    pub ibeam: *mut GLFWcursor,
    pub crosshair: *mut GLFWcursor,
    pub pointing_hand: *mut GLFWcursor,
    pub resize_ew: *mut GLFWcursor,
    pub resize_ns: *mut GLFWcursor,
    pub resize_nwse: *mut GLFWcursor,
    pub resize_nesw: *mut GLFWcursor,
    pub resize_all: *mut GLFWcursor,
    pub not_allowed: *mut GLFWcursor,
}

impl Default for GlfwStandardCursors {
    fn default() -> Self {
        unsafe {
            GlfwStandardCursors {
                arrow: glfwCreateStandardCursor(GLFW_ARROW_CURSOR as _),
                ibeam: glfwCreateStandardCursor(GLFW_IBEAM_CURSOR as _),
                crosshair: glfwCreateStandardCursor(GLFW_CROSSHAIR_CURSOR as _),
                pointing_hand: glfwCreateStandardCursor(GLFW_POINTING_HAND_CURSOR as _),
                resize_ew: glfwCreateStandardCursor(GLFW_RESIZE_EW_CURSOR as _),
                resize_ns: glfwCreateStandardCursor(GLFW_RESIZE_NS_CURSOR as _),
                resize_nwse: glfwCreateStandardCursor(GLFW_RESIZE_NWSE_CURSOR as _),
                resize_nesw: glfwCreateStandardCursor(GLFW_RESIZE_NESW_CURSOR as _),
                resize_all: glfwCreateStandardCursor(GLFW_RESIZE_ALL_CURSOR as _),
                not_allowed: glfwCreateStandardCursor(GLFW_NOT_ALLOWED_CURSOR as _),
            }
        }
    }
}

// first monitor is the primary monitor
// SAFETY: only valid until glfw is terminated or the monitor config changes
unsafe fn monitors() -> &'static [*mut GLFWmonitor] {
    let mut monitor_count = 0;
    let monitors_ptr = glfwGetMonitors(&mut monitor_count);
    let monitors = slice::from_raw_parts(monitors_ptr, monitor_count as _);
    assert!(!monitors.is_empty(), "No monitors found");
    monitors
}
