use crate::{loading::FontAssets, AppState};
use bevy::{app::AppExit, prelude::*};
use bevy_ui_navigation::{
    components::FocusableButtonBundle,
    systems::{default_keyboard_input, InputMapping},
    Focusable, NavEvent, NavMenu, NavRequest, NavigationPlugin,
};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ButtonColors>()
            .init_resource::<InputMapping>()
            .add_plugin(NavigationPlugin)
            .add_system_set(
                SystemSet::on_enter(AppState::Menu)
                    .with_system(setup_camera)
                    .with_system(setup_menu),
            )
            .add_system_set(
                SystemSet::on_update(AppState::Menu)
                    .with_system(go_to_game)
                    .with_system(default_keyboard_input)
                    .with_system(button_system)
                    .with_system(visible_battle_mode_node)
                    .with_system(invisible_battle_mode_node)
                    .with_system(quit_game),
            )
            .add_system_set(SystemSet::on_exit(AppState::Menu).with_system(cleanup_menu));
    }
}

struct MenuData {
    ui_root: Entity,
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
struct RootButton;

#[derive(Component)]
struct PlayerModeNode;

#[derive(Component)]
enum PlayerModeButton {
    OnePlayer,
    TwoPlayer,
}

#[derive(Component)]
struct BattleModeNode;

#[derive(Component)]
enum OnePlayerBattleModeButton {
    Endless,
    ScoreAttack,
    Puzzle,
    VsCom,
}

#[derive(Component)]
struct DifficultyButton;

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(UiCameraBundle::default());
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn setup_menu(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    mut requests: EventWriter<NavRequest>,
) {
    // setup root
    let root_button = commands
        .spawn_bundle(FocusableButtonBundle::from(ButtonBundle {
            color: Color::NONE.into(),
            ..Default::default()
        }))
        .insert(RootButton)
        .id();
    let ui_root = commands
        .spawn_bundle(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                position_type: PositionType::Absolute,
                align_items: AlignItems::FlexEnd,
                size: Size::new(Val::Percent(50.0), Val::Percent(100.0)),
                ..Default::default()
            },
            color: Color::NONE.into(),
            ..Default::default()
        })
        .insert(NavMenu::root())
        .insert(PlayerModeNode)
        .push_children(&[root_button])
        .id();
    // setup player mode node
    let one_player_mode = commands
        .spawn_bundle(focusable_button(true))
        .insert(PlayerModeButton::OnePlayer)
        .with_children(|cmd| {
            cmd.spawn_bundle(text(&font_assets, "1 Player", true));
        })
        .id();
    let two_player_mode = commands
        .spawn_bundle(focusable_button(true))
        .insert(PlayerModeButton::TwoPlayer)
        .with_children(|cmd| {
            cmd.spawn_bundle(text(&font_assets, "2 Players", true));
        })
        .id();

    let player_mode_node = commands
        .spawn_bundle(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::ColumnReverse,
                position_type: PositionType::Relative,
                size: Size::new(Val::Percent(20.0), Val::Percent(20.0)),
                ..Default::default()
            },
            color: Color::NONE.into(),
            ..Default::default()
        })
        .insert(NavMenu::reachable_from(root_button))
        .insert(BattleModeNode)
        .push_children(&[one_player_mode, two_player_mode])
        .id();
    requests.send(NavRequest::FocusOn(one_player_mode));

    // setup one plyaer mode node
    let one_player_endless = commands
        .spawn_bundle(focusable_button(false))
        .insert(OnePlayerBattleModeButton::Endless)
        .with_children(|cmd| {
            cmd.spawn_bundle(text(&font_assets, "endless", false));
        })
        .id();
    let one_player_score_attack = commands
        .spawn_bundle(focusable_button(false))
        .insert(OnePlayerBattleModeButton::ScoreAttack)
        .with_children(|cmd| {
            cmd.spawn_bundle(text(&font_assets, "score attack", false));
        })
        .id();
    let one_player_puzzle = commands
        .spawn_bundle(focusable_button(false))
        .insert(OnePlayerBattleModeButton::Puzzle)
        .with_children(|cmd| {
            cmd.spawn_bundle(text(&font_assets, "puzzle", false));
        })
        .id();
    let one_player_vs_com = commands
        .spawn_bundle(focusable_button(false))
        .insert(OnePlayerBattleModeButton::VsCom)
        .with_children(|cmd| {
            cmd.spawn_bundle(text(&font_assets, "vs com", false));
        })
        .id();
    let one_player_mode_node = commands
        .spawn_bundle(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::ColumnReverse,
                position_type: PositionType::Relative,
                size: Size::new(Val::Percent(20.0), Val::Percent(20.0)),
                ..Default::default()
            },
            visibility: Visibility { is_visible: false },
            color: Color::NONE.into(),
            ..Default::default()
        })
        .insert(NavMenu::reachable_from(one_player_mode))
        .push_children(&[
            one_player_endless,
            one_player_score_attack,
            one_player_puzzle,
            one_player_vs_com,
        ])
        .id();

    // TODO: setup 2player mode
    let two_player_mode_node = commands
        .spawn_bundle(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                size: Size::new(Val::Auto, Val::Auto),
                ..Default::default()
            },
            color: Color::NONE.into(),
            ..Default::default()
        })
        .id();

    commands.entity(ui_root).push_children(&[
        player_mode_node,
        one_player_mode_node,
        two_player_mode_node,
    ]);

    commands.insert_resource(MenuData { ui_root });
}

fn focusable_button(is_visible: bool) -> FocusableButtonBundle {
    FocusableButtonBundle::from(ButtonBundle {
        style: Style {
            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
            ..Default::default()
        },
        color: Color::GREEN.into(),
        visibility: Visibility { is_visible },
        ..Default::default()
    })
}

fn text(font_assets: &Res<FontAssets>, label: &str, is_visible: bool) -> TextBundle {
    TextBundle {
        style: Style {
            margin: Rect::all(Val::Px(10.0)),
            align_items: AlignItems::FlexStart,
            ..Default::default()
        },
        text: Text::with_section(
            label,
            TextStyle {
                font: font_assets.font.clone(),
                font_size: 30.0,
                color: Color::BLACK,
            },
            Default::default(),
        ),
        visibility: Visibility { is_visible },
        ..Default::default()
    }
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

fn visible_battle_mode_node(
    mut events: EventReader<NavEvent>,
    to_battle_mode_button: Query<Entity, With<OnePlayerBattleModeButton>>,
    from_player_mode_buttons: Query<Entity, With<PlayerModeButton>>,
    mut battle_mode_buttons: Query<
        (&mut Visibility, &Children),
        (With<OnePlayerBattleModeButton>, Without<Text>),
    >,
    mut battle_mode_button_text: Query<&mut Visibility, With<Text>>,
) {
    for event in events.iter() {
        if let NavEvent::FocusChanged { from, to } = event {
            if from_player_mode_buttons.get(*from.first()).is_ok()
                && to_battle_mode_button.get(*to.first()).is_ok()
            {
                for (mut button_visibility, children) in battle_mode_buttons.iter_mut() {
                    button_visibility.is_visible = true;
                    for &child in children.iter() {
                        if let Ok(mut text_visibility) = battle_mode_button_text.get_mut(child) {
                            text_visibility.is_visible = true;
                        }
                    }
                }
            }
        }
    }
}

fn invisible_battle_mode_node(
    mut events: EventReader<NavEvent>,
    from_battle_mode_button: Query<Entity, With<OnePlayerBattleModeButton>>,
    to_player_mode_button: Query<Entity, With<PlayerModeButton>>,
    mut battle_mode_buttons: Query<
        (&mut Visibility, &Children),
        (With<OnePlayerBattleModeButton>, Without<Text>),
    >,
    mut battle_mode_button_text: Query<&mut Visibility, With<Text>>,
) {
    for event in events.iter() {
        if let NavEvent::FocusChanged { from, to } = event {
            if from_battle_mode_button.get(*from.first()).is_ok()
                && to_player_mode_button.get(*to.first()).is_ok()
            {
                for (mut button_visibility, children) in battle_mode_buttons.iter_mut() {
                    button_visibility.is_visible = false;
                    for &child in children.iter() {
                        if let Ok(mut text_visibility) = battle_mode_button_text.get_mut(child) {
                            text_visibility.is_visible = false;
                        }
                    }
                }
            }
        }
    }
}

fn quit_game(
    mut events: EventReader<NavEvent>,
    from_player_mode_button: Query<Entity, With<PlayerModeButton>>,
    to_root_button: Query<Entity, With<RootButton>>,
    mut exit: EventWriter<AppExit>,
) {
    for event in events.iter() {
        if let NavEvent::FocusChanged { from, to } = event {
            if from_player_mode_button.get(*from.first()).is_ok()
                && to_root_button.get(*to.first()).is_ok()
            {
                exit.send(AppExit);
            }
        }
    }
}

fn go_to_game(
    mut state: ResMut<State<AppState>>,
    mut events: EventReader<NavEvent>,
    mut exit: EventWriter<AppExit>,
) {
    for event in events.iter() {
        if let NavEvent::NoChanges { from: _, request } = event {
            match request {
                NavRequest::Action => {
                    state.set(AppState::InGame).unwrap();
                }
                NavRequest::Cancel => {
                    exit.send(AppExit);
                }
                _ => {}
            }
        }
    }
}

fn cleanup_menu(mut commands: Commands, menu_data: Res<MenuData>) {
    commands.entity(menu_data.ui_root).despawn_recursive();
}
