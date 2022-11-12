# `bevy_glfw`

GLFW window backend for Bevy

[![crates.io](https://img.shields.io/crates/v/bevy_glfw)](https://crates.io/crates/bevy_glfw)
[![docs.rs](https://docs.rs/bevy_glfw/badge.svg)](https://docs.rs/bevy_glfw)

## Usage

`Cargo.toml`
```
bevy = {
    version = "...",
    default-features = false, // <- Important 
    features = [
        // Only required features!
        // Notably *not*:
        // - "bevy_winit"
        // - "x11" (also enables winit)
        // - "wayland" (also enables winit)
    ]
}
```

---

`main.rs`
```
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_glfw::GlfwPlugin) // <- Add the plugin as usual
        .run();
}
```

## Motivation

Introducing a proper stop-gap solution until
[winit#1806](https://github.com/rust-windowing/winit/issues/1806) is completed
and released.

## Bevy Version Support

| bevy | bevy_glfw |
| ---- | --------- |
| 0.8  | 0.1       |

## Credit

- [Red Artist](https://github.com/coderedart) for the base code.

## License

`bevy_glfw` is free, open source and permissively licensed!
Except where noted (below and/or in individual files), all code in this repository is dual-licensed under either:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.
This means you can select the license you prefer!
This dual-licensing approach is the de-facto standard in the Rust ecosystem and there are [very good reasons](https://github.com/bevyengine/bevy/issues/2373) to include both.

### Your contributions

Unless you explicitly state otherwise,
any contribution intentionally submitted for inclusion in the work by you,
as defined in the Apache-2.0 license,
shall be dual licensed as above,
without any additional terms or conditions.
