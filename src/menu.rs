use crate::{loading::FontAssets, AppState};
use bevy::{app::AppExit, prelude::*};
use bevy_ui_navigation::{
    components::FocusableButtonBundle,
    systems::{default_keyboard_input, InputMapping},
    Focusable, NavEvent, NavMenu, NavRequest,
};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ButtonColors>()
            .init_resource::<InputMapping>()
            .add_system_set(SystemSet::on_enter(AppState::Menu).with_system(setup_menu))
            .add_system_set(
                SystemSet::on_update(AppState::Menu)
                    .with_system(go_to_game)
                    .with_system(default_keyboard_input)
                    .with_system(button_system),
            )
            .add_system_set(SystemSet::on_exit(AppState::Menu).with_system(cleanup_menu));
    }
}

struct ButtonColors {
    normal: Color,
    focused: Color,
    active: Color,
}

impl Default for ButtonColors {
    fn default() -> Self {
        ButtonColors {
            normal: Color::DARK_GRAY,
            focused: Color::ORANGE_RED,
            active: Color::GOLD,
        }
    }
}

#[derive(Component)]
struct DifficultyButton;

fn setup_menu(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    button_colors: Res<ButtonColors>,
    mut requests: EventWriter<NavRequest>,
) {
    commands.spawn_bundle(UiCameraBundle::default());

    let root_button = commands
        .spawn_bundle(FocusableButtonBundle::from(ButtonBundle {
            color: Color::NONE.into(),
            ..Default::default()
        }))
        .id();
    let positions = [
        (Vec2::new(10.0, 0.0), "EASY"),
        (Vec2::new(30.0, 0.0), "NORMAL"),
        (Vec2::new(50.0, 0.0), "HARD"),
    ];

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                ..Default::default()
            },
            color: Color::NONE.into(),
            ..Default::default()
        })
        .insert(NavMenu::root())
        .push_children(&[root_button])
        .with_children(|cmd| {
            cmd.spawn_bundle(NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    ..Default::default()
                },
                color: Color::NONE.into(),
                ..Default::default()
            })
            .insert(NavMenu::reachable_from(root_button));
            for (idx, (pos, mode)) in positions.into_iter().enumerate() {
                let position = Rect {
                    left: Val::Percent(pos.x),
                    top: Val::Percent(pos.y),
                    ..Default::default()
                };
                let focusable_button = cmd
                    .spawn_bundle(ButtonBundle {
                        style: Style {
                            size: Size::new(Val::Px(95.0), Val::Px(95.0)),
                            position,
                            position_type: PositionType::Absolute,
                            ..Default::default()
                        },
                        color: button_colors.normal.into(),
                        ..Default::default()
                    })
                    .with_children(|p| {
                        p.spawn_bundle(TextBundle {
                            text: Text::with_section(
                                mode,
                                TextStyle {
                                    font: font_assets.font.clone(),
                                    font_size: 30.0,
                                    ..Default::default()
                                },
                                Default::default(),
                            ),
                            ..Default::default()
                        });
                    })
                    .insert(Focusable::default())
                    .insert(DifficultyButton)
                    .id();
                if idx == 0 {
                    requests.send(NavRequest::FocusOn(focusable_button));
                }
            }
        });
}

fn button_system(
    button_colors: Res<ButtonColors>,
    mut interaction: Query<(&Focusable, &mut UiColor), (Changed<Focusable>, With<Button>)>,
) {
    for (focus_state, mut color) in interaction.iter_mut() {
        if focus_state.is_focused() {
            *color = button_colors.focused.into();
        } else if focus_state.is_active() {
            *color = button_colors.active.into();
        } else {
            *color = button_colors.normal.into();
        }
    }
}

fn go_to_game(
    mut state: ResMut<State<AppState>>,
    mut events: EventReader<NavEvent>,
    mut exit: EventWriter<AppExit>,
) {
    for event in events.iter() {
        match event {
            NavEvent::NoChanges { from: _, request } => match request {
                NavRequest::Action => {
                    state.set(AppState::InGame).unwrap();
                }
                NavRequest::Cancel => {
                    exit.send(AppExit);
                }
                _ => {}
            },
            _ => {}
        }
    }
}

fn cleanup_menu() {}
