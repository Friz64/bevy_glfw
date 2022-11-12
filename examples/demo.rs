use bevy::{
    input::keyboard::KeyboardInput,
    prelude::*,
    render::camera::RenderTarget,
    sprite::MaterialMesh2dBundle,
    time::FixedTimestep,
    window::{CreateWindow, WindowFocused, WindowId, WindowMode},
};

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
struct FixedUpdateStage;

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
struct FocusedWindowDependant;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: 200.,
            height: 200.,
            title: "bevy_glfw".to_string(),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_glfw::GlfwPlugin)
        .add_startup_system(setup)
        .add_system(update_example_cube)
        .add_system(update_focused_window)
        .add_stage_after(
            CoreStage::Update,
            FixedUpdateStage,
            SystemStage::parallel()
                .with_run_criteria(FixedTimestep::step(0.1))
                .with_system(change_title),
        )
        .add_system(log_keys)
        .add_system(spawn_window)
        .add_system(close_on_esc.after(update_focused_window))
        .add_system(cycle_cursor_icon.after(update_focused_window))
        .add_system(cycle_window_mode.after(update_focused_window))
        .add_system(cycle_resolution.after(update_focused_window))
        .add_system(toggle_resizable.after(update_focused_window))
        .add_system(toggle_decorations.after(update_focused_window))
        .add_system(toggle_cursor_lock.after(update_focused_window))
        .add_system(toggle_cursor_visibility.after(update_focused_window))
        .add_system(set_cursor_pos.after(update_focused_window))
        .add_system(toggle_minimized.after(update_focused_window))
        .add_system(toggle_maximized.after(update_focused_window))
        .add_system(set_window_pos.after(update_focused_window))
        .add_system(cycle_center_window.after(update_focused_window))
        .run();
}

struct ExampleCube(Entity);
struct LastFocusedWindow(WindowId);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_bundle(Camera2dBundle::default());
    let example_cube = commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
            transform: Transform::default().with_scale(Vec3::splat(128.)),
            material: materials.add(ColorMaterial::from(Color::MIDNIGHT_BLUE)),
            ..default()
        })
        .id();

    commands.insert_resource(ExampleCube(example_cube));
    commands.insert_resource(LastFocusedWindow(WindowId::primary()));

    info!("Take a look at the source code!");
}

fn update_focused_window(
    mut focused: ResMut<LastFocusedWindow>,
    mut focused_events: EventReader<WindowFocused>,
    windows: Res<Windows>,
) {
    for event in focused_events.iter() {
        if event.focused {
            focused.0 = event.id;
        }
    }

    if windows.get(focused.0).is_none() {
        focused.0 = windows.iter().next().unwrap().id();
    }
}

fn update_example_cube(
    time: Res<Time>,
    example_cube: Res<ExampleCube>,
    mut transform: Query<&mut Transform>,
) {
    *transform.get_mut(example_cube.0).unwrap().rotation =
        *Quat::from_rotation_z((5.0 * time.seconds_since_startup()) as _);
}

fn log_keys(mut keyboard_events: EventReader<KeyboardInput>) {
    for keyboard_event in keyboard_events.iter() {
        if keyboard_event.state.is_pressed() {
            println!("Pressed: {:?}", keyboard_event.key_code);
        }
    }
}

fn spawn_window(
    mut commands: Commands,
    input: Res<Input<KeyCode>>,
    mut create_window_events: EventWriter<CreateWindow>,
) {
    if input.just_pressed(KeyCode::M) {
        let window_id = WindowId::new();

        create_window_events.send(CreateWindow {
            id: window_id,
            descriptor: WindowDescriptor {
                width: 300.,
                height: 300.,
                title: "Additional window".to_string(),
                ..default()
            },
        });

        commands.spawn_bundle(Camera2dBundle {
            camera: Camera {
                target: RenderTarget::Window(window_id),
                ..default()
            },
            ..default()
        });
    }
}

fn close_on_esc(
    focused: ResMut<LastFocusedWindow>,
    mut windows: ResMut<Windows>,
    input: Res<Input<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Escape) {
        windows.get_mut(focused.0).unwrap().close();
    }
}

fn change_title(time: Res<Time>, focused: ResMut<LastFocusedWindow>, mut windows: ResMut<Windows>) {
    let window = windows.get_mut(focused.0).unwrap();
    window.set_title(format!("Seconds: {:.1}", time.seconds_since_startup()));
}

fn cycle_cursor_icon(
    input: Res<Input<KeyCode>>,
    focused: ResMut<LastFocusedWindow>,
    mut windows: ResMut<Windows>,
    mut index: Local<usize>,
) {
    const ICONS: &[CursorIcon] = &[
        CursorIcon::Crosshair,
        CursorIcon::Hand,
        CursorIcon::Arrow,
        CursorIcon::Text,
        CursorIcon::NotAllowed,
        CursorIcon::EResize,
        CursorIcon::NResize,
        CursorIcon::NeResize,
        CursorIcon::NwResize,
        CursorIcon::SResize,
        CursorIcon::SeResize,
        CursorIcon::SwResize,
        CursorIcon::WResize,
        CursorIcon::EwResize,
        CursorIcon::NsResize,
        CursorIcon::NeswResize,
        CursorIcon::NwseResize,
    ];

    if input.just_pressed(KeyCode::A) {
        let window = windows.get_mut(focused.0).unwrap();
        *index = (*index + 1) % ICONS.len();
        window.set_cursor_icon(dbg!(ICONS[*index]));
    }
}

fn cycle_window_mode(
    input: Res<Input<KeyCode>>,
    focused: ResMut<LastFocusedWindow>,
    mut windows: ResMut<Windows>,
    mut index: Local<usize>,
) {
    const WINDOW_MODES: &[WindowMode] = &[
        WindowMode::Windowed,
        WindowMode::SizedFullscreen,
        WindowMode::BorderlessFullscreen,
        WindowMode::Fullscreen,
    ];

    if input.just_pressed(KeyCode::B) {
        let window = windows.get_mut(focused.0).unwrap();
        *index = (*index + 1) % WINDOW_MODES.len();
        window.set_mode(dbg!(WINDOW_MODES[*index]));
    }
}

fn cycle_resolution(
    input: Res<Input<KeyCode>>,
    focused: ResMut<LastFocusedWindow>,
    mut windows: ResMut<Windows>,
    mut index: Local<usize>,
) {
    const RESOLUTIONS: &[Vec2] = &[
        Vec2::new(200., 200.),
        Vec2::new(300., 200.),
        Vec2::new(200., 300.),
        Vec2::new(1920., 1080.),
    ];

    if input.just_pressed(KeyCode::C) {
        let window = windows.get_mut(focused.0).unwrap();
        *index = (*index + 1) % RESOLUTIONS.len();
        let resolution = dbg!(RESOLUTIONS[*index]);
        window.set_resolution(resolution.x, resolution.y);
    }
}

fn toggle_resizable(
    input: Res<Input<KeyCode>>,
    focused: ResMut<LastFocusedWindow>,
    mut windows: ResMut<Windows>,
) {
    if input.just_pressed(KeyCode::D) {
        let window = windows.get_mut(focused.0).unwrap();
        window.set_resizable(dbg!(!window.resizable()));
    }
}

fn toggle_decorations(
    input: Res<Input<KeyCode>>,
    focused: ResMut<LastFocusedWindow>,
    mut windows: ResMut<Windows>,
) {
    if input.just_pressed(KeyCode::E) {
        let window = windows.get_mut(focused.0).unwrap();
        window.set_decorations(dbg!(!window.decorations()));
    }
}

fn toggle_cursor_lock(
    input: Res<Input<KeyCode>>,
    focused: ResMut<LastFocusedWindow>,
    mut windows: ResMut<Windows>,
) {
    if input.just_pressed(KeyCode::F) {
        let window = windows.get_mut(focused.0).unwrap();
        window.set_cursor_lock_mode(dbg!(!window.cursor_locked()));
    }
}

fn toggle_cursor_visibility(
    input: Res<Input<KeyCode>>,
    focused: ResMut<LastFocusedWindow>,
    mut windows: ResMut<Windows>,
) {
    if input.just_pressed(KeyCode::G) {
        let window = windows.get_mut(focused.0).unwrap();
        window.set_cursor_visibility(dbg!(!window.cursor_visible()));
    }
}

fn set_cursor_pos(
    input: Res<Input<KeyCode>>,
    focused: ResMut<LastFocusedWindow>,
    mut windows: ResMut<Windows>,
) {
    if input.just_pressed(KeyCode::H) {
        let window = windows.get_mut(focused.0).unwrap();
        dbg!(window.set_cursor_position(Vec2::new(100.0, 100.0)));
    }
}

fn toggle_maximized(
    input: Res<Input<KeyCode>>,
    focused: ResMut<LastFocusedWindow>,
    mut windows: ResMut<Windows>,
    mut maximized: Local<bool>,
) {
    if input.just_pressed(KeyCode::I) {
        let window = windows.get_mut(focused.0).unwrap();
        *maximized = !*maximized;
        window.set_maximized(dbg!(*maximized));
    }
}

fn toggle_minimized(
    input: Res<Input<KeyCode>>,
    focused: ResMut<LastFocusedWindow>,
    mut windows: ResMut<Windows>,
    mut minimized: Local<bool>,
) {
    if input.just_pressed(KeyCode::J) {
        let window = windows.get_mut(focused.0).unwrap();
        *minimized = !*minimized;
        window.set_minimized(dbg!(*minimized));
    }
}

fn set_window_pos(
    input: Res<Input<KeyCode>>,
    focused: ResMut<LastFocusedWindow>,
    mut windows: ResMut<Windows>,
) {
    if input.just_pressed(KeyCode::K) {
        let window = windows.get_mut(focused.0).unwrap();
        window.set_position(dbg!(IVec2::new(100, 100)));
    }
}

fn cycle_center_window(
    input: Res<Input<KeyCode>>,
    focused: ResMut<LastFocusedWindow>,
    mut windows: ResMut<Windows>,
    mut index: Local<usize>,
) {
    const SELECTIONS: &[MonitorSelection] = &[
        MonitorSelection::Current,
        MonitorSelection::Primary,
        MonitorSelection::Number(0),
    ];

    if input.just_pressed(KeyCode::L) {
        let window = windows.get_mut(focused.0).unwrap();
        *index = (*index + 1) % SELECTIONS.len();
        window.center_window(dbg!(SELECTIONS[*index]));
    }
}
