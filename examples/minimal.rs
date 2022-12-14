use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_glfw::GlfwPlugin)
        .run();
}
