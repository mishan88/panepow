// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(target_arch = "wasm32")]
use bevy_webgl2;

use bevy::prelude::{App, DefaultPlugins, WindowDescriptor};
use game_plugin::GamePlugin;

fn main() {
    let mut app = App::build();
    app
        .insert_resource(WindowDescriptor {
            title: String::from("PanelPow"),
            width: 1920.0,
            height: 1080.0,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(GamePlugin);
    #[cfg(target_arch = "wasm32")]
    app.add_plugin(bevy_webgl2::WebGL2Plugin);
    app.run();
}