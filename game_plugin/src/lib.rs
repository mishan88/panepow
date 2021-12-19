mod ingame;
use bevy::prelude::AppBuilder;
use bevy::prelude::*;

use crate::ingame::IngamePlugin;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    Menu,
    InGame,
    Paused,
    GameOver,
    GameOverMenu,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .add_plugin(IngamePlugin);
    }
}