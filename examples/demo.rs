use bevy::{
    input::keyboard::KeyboardInput, prelude::*, sprite::MaterialMesh2dBundle, time::FixedTimestep,
    window::WindowMode,
};

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
struct FixedUpdateStage;

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
        .add_system(bevy::window::close_on_esc)
        .add_system(update_example_cube)
        .add_stage_after(
            CoreStage::Update,
            FixedUpdateStage,
            SystemStage::parallel()
                .with_run_criteria(FixedTimestep::step(0.1))
                .with_system(change_title),
        )
        .add_system(cycle_cursor_icon)
        .add_system(cycle_window_mode)
        .add_system(cycle_resolution)
        .add_system(toggle_resizable)
        .add_system(toggle_decorations)
        .add_system(toggle_cursor_lock)
        .add_system(toggle_cursor_visibility)
        .add_system(set_cursor_pos)
        .add_system(toggle_minimized)
        .add_system(toggle_maximized)
        .add_system(set_window_pos)
        .add_system(cycle_center_window)
        .add_system(log_keys)
        .run();
}

struct ExampleCube(Entity);

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

    info!("Take a look at the source code!");
}

fn update_example_cube(
    time: Res<Time>,
    example_cube: Res<ExampleCube>,
    mut transform: Query<&mut Transform>,
) {
    *transform.get_mut(example_cube.0).unwrap().rotation =
        *Quat::from_rotation_z((5.0 * time.seconds_since_startup()) as _);
}

fn change_title(time: Res<Time>, mut windows: ResMut<Windows>) {
    let window = windows.primary_mut();
    window.set_title(format!("Seconds: {:.1}", time.seconds_since_startup()));
}

fn cycle_cursor_icon(
    input: Res<Input<KeyCode>>,
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
        let window = windows.primary_mut();
        *index = (*index + 1) % ICONS.len();
        window.set_cursor_icon(dbg!(ICONS[*index]));
    }
}

fn cycle_window_mode(
    input: Res<Input<KeyCode>>,
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
        let window = windows.primary_mut();
        *index = (*index + 1) % WINDOW_MODES.len();
        window.set_mode(dbg!(WINDOW_MODES[*index]));
    }
}

fn cycle_resolution(
    input: Res<Input<KeyCode>>,
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
        let window = windows.primary_mut();
        *index = (*index + 1) % RESOLUTIONS.len();
        let resolution = dbg!(RESOLUTIONS[*index]);
        window.set_resolution(resolution.x, resolution.y);
    }
}

fn toggle_resizable(input: Res<Input<KeyCode>>, mut windows: ResMut<Windows>) {
    if input.just_pressed(KeyCode::D) {
        let window = windows.primary_mut();
        window.set_resizable(dbg!(!window.resizable()));
    }
}

fn toggle_decorations(input: Res<Input<KeyCode>>, mut windows: ResMut<Windows>) {
    if input.just_pressed(KeyCode::E) {
        let window = windows.primary_mut();
        window.set_decorations(dbg!(!window.decorations()));
    }
}

fn toggle_cursor_lock(input: Res<Input<KeyCode>>, mut windows: ResMut<Windows>) {
    if input.just_pressed(KeyCode::F) {
        let window = windows.primary_mut();
        window.set_cursor_lock_mode(dbg!(!window.cursor_locked()));
    }
}

fn toggle_cursor_visibility(input: Res<Input<KeyCode>>, mut windows: ResMut<Windows>) {
    if input.just_pressed(KeyCode::G) {
        let window = windows.primary_mut();
        window.set_cursor_visibility(dbg!(!window.cursor_visible()));
    }
}

fn set_cursor_pos(input: Res<Input<KeyCode>>, mut windows: ResMut<Windows>) {
    if input.just_pressed(KeyCode::H) {
        let window = windows.primary_mut();
        dbg!(window.set_cursor_position(Vec2::new(100.0, 100.0)));
    }
}

fn toggle_maximized(
    input: Res<Input<KeyCode>>,
    mut windows: ResMut<Windows>,
    mut maximized: Local<bool>,
) {
    if input.just_pressed(KeyCode::I) {
        let window = windows.primary_mut();
        *maximized = !*maximized;
        window.set_maximized(dbg!(*maximized));
    }
}

fn toggle_minimized(
    input: Res<Input<KeyCode>>,
    mut windows: ResMut<Windows>,
    mut minimized: Local<bool>,
) {
    if input.just_pressed(KeyCode::J) {
        let window = windows.primary_mut();
        *minimized = !*minimized;
        window.set_minimized(dbg!(*minimized));
    }
}

fn set_window_pos(input: Res<Input<KeyCode>>, mut windows: ResMut<Windows>) {
    if input.just_pressed(KeyCode::K) {
        let window = windows.primary_mut();
        window.set_position(dbg!(IVec2::new(100, 100)));
    }
}

fn cycle_center_window(
    input: Res<Input<KeyCode>>,
    mut windows: ResMut<Windows>,
    mut index: Local<usize>,
) {
    const SELECTIONS: &[MonitorSelection] = &[
        MonitorSelection::Current,
        MonitorSelection::Primary,
        MonitorSelection::Number(0),
    ];

    if input.just_pressed(KeyCode::L) {
        let window = windows.primary_mut();
        *index = (*index + 1) % SELECTIONS.len();
        window.center_window(dbg!(SELECTIONS[*index]));
    }
}

fn log_keys(mut keyboard_events: EventReader<KeyboardInput>) {
    for keyboard_event in keyboard_events.iter() {
        if keyboard_event.state.is_pressed() {
            println!("Pressed: {:?}", keyboard_event.key_code);
        }
    }
}
