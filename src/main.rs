use bevy::prelude::*;
use bevy::window::PresentMode;

// import plugins
mod hexagon;

fn main() {
  App::new()
    .add_plugins(DefaultPlugins.set(WindowPlugin {
      primary_window: Some(Window {
        title: "Bevy Test Program".into(),
        resolution: (800.0,600.0).into(),
        present_mode: PresentMode::AutoVsync,
        resizable: false,
        ..default()
      }),
      ..default()
    }))
    .add_plugin(hexagon::HexagonPlugin) // full system for hello
    .run();
}
