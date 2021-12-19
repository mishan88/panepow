use bevy::prelude::*;
use game_plugin::GamePlugin;

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: String::from("PanelPow"),
            width: 1920.0,
            height: 1080.0,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(GamePlugin)
        .run();
}
