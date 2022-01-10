use crate::{loading::FontAssets, AppState};
use bevy::prelude::*;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(AppState::Menu).with_system(setup_menu))
            .add_system_set(SystemSet::on_update(AppState::Menu).with_system(go_to_game));
    }
}

fn setup_menu(mut commands: Commands, font_assets: Res<FontAssets>) {
    commands.spawn_bundle(UiCameraBundle::default());
    commands.spawn_bundle(TextBundle {
        text: Text {
            sections: vec![TextSection {
                value: "Press Space KEY!".to_string(),
                style: TextStyle {
                    font: font_assets.font.clone(),
                    font_size: 40.0,
                    color: Color::rgb(1.0, 1.0, 1.0),
                },
            }],
            alignment: Default::default(),
        },
        ..Default::default()
    });
}

fn go_to_game(
    mut commands: Commands,
    input: Res<Input<KeyCode>>,
    text: Query<Entity, With<Text>>,
    mut state: ResMut<State<AppState>>,
) {
    if input.just_pressed(KeyCode::Space) {
        for entity in text.iter() {
            commands.entity(entity).despawn();
        }
        state.set(AppState::InGame).unwrap();
    }
}
