mod ingame;
mod loading;
mod menu;
use bevy::prelude::AppBuilder;
use bevy::prelude::*;

use crate::ingame::IngamePlugin;
use crate::loading::LoadingPlugin;
use crate::menu::MenuPlugin;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    Menu,
    Loading,
    InGame,
    GameOver,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_state(AppState::Loading)
            .add_plugin(LoadingPlugin)
            .add_plugin(MenuPlugin)
            .add_plugin(IngamePlugin);
    }
}
