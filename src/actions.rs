use crate::AppState;
use bevy::prelude::*;
use std::time::Duration;
pub struct ActionPlugin;

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MoveActions>()
            .init_resource::<SwapAction>()
            .init_resource::<LiftAction>()
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
                    .with_system(set_movement_actions.system())
                    .with_system(set_swap_action.system())
                    .with_system(set_lift_action.system()),
            );
    }
}

#[derive(Default)]
pub struct MoveActions {
    pub cursor_movement: Option<Vec2>,
    pub reinput_timer: Timer,
}

#[derive(Default)]
pub struct SwapAction(pub bool);

#[derive(Default)]
pub struct LiftAction {
    pub lift: bool,
    pub reinput_timer: Timer,
}

const FIRST_REINPUT_DURATION: f32 = 0.4;
const REINPUT_DURATION: f32 = 0.04;

fn set_movement_actions(
    mut actions: ResMut<MoveActions>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    actions
        .reinput_timer
        .tick(Duration::from_secs_f32(time.delta_seconds()));
    if GameControl::Up.just_released(&keyboard_input)
        || GameControl::Up.pressed(&keyboard_input)
        || GameControl::Left.just_released(&keyboard_input)
        || GameControl::Left.pressed(&keyboard_input)
        || GameControl::Down.just_released(&keyboard_input)
        || GameControl::Down.pressed(&keyboard_input)
        || GameControl::Right.just_released(&keyboard_input)
        || GameControl::Right.pressed(&keyboard_input)
    {
        let mut cursor_movement = Vec2::ZERO;

        if GameControl::Up.just_pressed(&keyboard_input) {
            cursor_movement.y = 1.;
            actions
                .reinput_timer
                .set_duration(Duration::from_secs_f32(FIRST_REINPUT_DURATION));
            actions.reinput_timer.reset();
        } else if GameControl::Down.just_pressed(&keyboard_input) {
            cursor_movement.y = -1.;
            actions
                .reinput_timer
                .set_duration(Duration::from_secs_f32(FIRST_REINPUT_DURATION));
            actions.reinput_timer.reset();
        } else if GameControl::Down.pressed(&keyboard_input)
            && actions.reinput_timer.just_finished()
        {
            cursor_movement.y = -1.;
            actions
                .reinput_timer
                .set_duration(Duration::from_secs_f32(REINPUT_DURATION));
            actions.reinput_timer.reset();
        } else if GameControl::Up.pressed(&keyboard_input) && actions.reinput_timer.just_finished()
        {
            cursor_movement.y = 1.;
            actions
                .reinput_timer
                .set_duration(Duration::from_secs_f32(REINPUT_DURATION));
            actions.reinput_timer.reset();
        } else {
            cursor_movement.y = 0.;
        }

        if GameControl::Right.just_pressed(&keyboard_input) {
            cursor_movement.x = 1.;
            actions
                .reinput_timer
                .set_duration(Duration::from_secs_f32(FIRST_REINPUT_DURATION));
            actions.reinput_timer.reset();
        } else if GameControl::Left.just_pressed(&keyboard_input) {
            cursor_movement.x = -1.;
            actions
                .reinput_timer
                .set_duration(Duration::from_secs_f32(FIRST_REINPUT_DURATION));
            actions.reinput_timer.reset();
        } else if GameControl::Right.pressed(&keyboard_input)
            && actions.reinput_timer.just_finished()
        {
            cursor_movement.x = 1.;
            actions
                .reinput_timer
                .set_duration(Duration::from_secs_f32(REINPUT_DURATION));
            actions.reinput_timer.reset();
        } else if GameControl::Left.pressed(&keyboard_input)
            && actions.reinput_timer.just_finished()
        {
            cursor_movement.x = -1.;
            actions
                .reinput_timer
                .set_duration(Duration::from_secs_f32(REINPUT_DURATION));
            actions.reinput_timer.reset();
        } else {
            cursor_movement.x = 0.;
        }
        actions.cursor_movement = Some(cursor_movement);
    } else {
        actions.cursor_movement = None;
    }
}

fn set_swap_action(mut actions: ResMut<SwapAction>, keyboard_input: Res<Input<KeyCode>>) {
    if GameControl::Swap.just_pressed(&keyboard_input) {
        actions.0 = true;
    } else {
        actions.0 = false;
    }
}

fn set_lift_action(
    mut actions: ResMut<LiftAction>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    actions
        .reinput_timer
        .tick(Duration::from_secs_f32(time.delta_seconds()));
    if GameControl::ManualLift.pressed(&keyboard_input) {
        if GameControl::ManualLift.just_pressed(&keyboard_input)
            || actions.reinput_timer.just_finished()
        {
            actions.lift = true;
            actions
                .reinput_timer
                .set_duration(Duration::from_secs_f32(REINPUT_DURATION));
            actions.reinput_timer.reset();
        }
    } else {
        actions.lift = false;
    }
}

enum GameControl {
    Up,
    Down,
    Left,
    Right,
    Swap,
    ManualLift,
}

impl GameControl {
    fn just_released(&self, keyboard_input: &Res<Input<KeyCode>>) -> bool {
        match self {
            GameControl::Up => {
                keyboard_input.just_released(KeyCode::W)
                    || keyboard_input.just_released(KeyCode::Up)
            }
            GameControl::Down => {
                keyboard_input.just_released(KeyCode::S)
                    || keyboard_input.just_released(KeyCode::Down)
            }
            GameControl::Left => {
                keyboard_input.just_released(KeyCode::A)
                    || keyboard_input.just_released(KeyCode::Left)
            }
            GameControl::Right => {
                keyboard_input.just_released(KeyCode::D)
                    || keyboard_input.just_released(KeyCode::Right)
            }
            GameControl::Swap => {
                keyboard_input.just_released(KeyCode::F)
                    || keyboard_input.just_released(KeyCode::Space)
            }
            GameControl::ManualLift => {
                keyboard_input.just_released(KeyCode::B)
                    || keyboard_input.just_released(KeyCode::Return)
            }
        }
    }

    fn pressed(&self, keyboard_input: &Res<Input<KeyCode>>) -> bool {
        match self {
            GameControl::Up => {
                keyboard_input.pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up)
            }
            GameControl::Down => {
                keyboard_input.pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down)
            }
            GameControl::Left => {
                keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left)
            }
            GameControl::Right => {
                keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right)
            }
            GameControl::Swap => {
                keyboard_input.pressed(KeyCode::F) || keyboard_input.pressed(KeyCode::Space)
            }
            GameControl::ManualLift => {
                keyboard_input.pressed(KeyCode::B) || keyboard_input.pressed(KeyCode::Return)
            }
        }
    }

    fn just_pressed(&self, keyboard_input: &Res<Input<KeyCode>>) -> bool {
        match self {
            GameControl::Up => {
                keyboard_input.just_pressed(KeyCode::W) || keyboard_input.just_pressed(KeyCode::Up)
            }
            GameControl::Down => {
                keyboard_input.just_pressed(KeyCode::S)
                    || keyboard_input.just_pressed(KeyCode::Down)
            }
            GameControl::Left => {
                keyboard_input.just_pressed(KeyCode::A)
                    || keyboard_input.just_pressed(KeyCode::Left)
            }
            GameControl::Right => {
                keyboard_input.just_pressed(KeyCode::D)
                    || keyboard_input.just_pressed(KeyCode::Right)
            }
            GameControl::Swap => {
                keyboard_input.just_pressed(KeyCode::F)
                    || keyboard_input.just_pressed(KeyCode::Space)
            }
            GameControl::ManualLift => {
                keyboard_input.just_pressed(KeyCode::B)
                    || keyboard_input.just_pressed(KeyCode::Return)
            }
        }
    }
}

#[test]
fn test_movement_actions() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(set_movement_actions.system());
    let mut time = Time::default();
    time.update();
    world.insert_resource(time);
    world.insert_resource(MoveActions::default());

    let mut input = Input::<KeyCode>::default();
    input.press(KeyCode::Up);
    world.insert_resource(input);

    update_stage.run(&mut world);
    assert_eq!(
        world.get_resource::<MoveActions>().unwrap().cursor_movement,
        Some(Vec2::new(0.0, 1.0))
    );
    assert_eq!(
        world
            .get_resource::<MoveActions>()
            .unwrap()
            .reinput_timer
            .duration(),
        Duration::from_secs_f32(FIRST_REINPUT_DURATION)
    );

    let mut input = Input::<KeyCode>::default();
    input.press(KeyCode::Down);
    world.insert_resource(input);

    update_stage.run(&mut world);
    assert_eq!(
        world.get_resource::<MoveActions>().unwrap().cursor_movement,
        Some(Vec2::new(0.0, -1.0))
    );
    assert_eq!(
        world
            .get_resource::<MoveActions>()
            .unwrap()
            .reinput_timer
            .duration(),
        Duration::from_secs_f32(FIRST_REINPUT_DURATION)
    );

    let mut input = Input::<KeyCode>::default();
    input.press(KeyCode::Right);
    world.insert_resource(input);

    update_stage.run(&mut world);
    assert_eq!(
        world.get_resource::<MoveActions>().unwrap().cursor_movement,
        Some(Vec2::new(1.0, 0.0))
    );
    assert_eq!(
        world
            .get_resource::<MoveActions>()
            .unwrap()
            .reinput_timer
            .duration(),
        Duration::from_secs_f32(FIRST_REINPUT_DURATION)
    );

    let mut input = Input::<KeyCode>::default();
    input.press(KeyCode::Left);
    world.insert_resource(input);

    update_stage.run(&mut world);
    assert_eq!(
        world.get_resource::<MoveActions>().unwrap().cursor_movement,
        Some(Vec2::new(-1.0, 0.0))
    );
    assert_eq!(
        world
            .get_resource::<MoveActions>()
            .unwrap()
            .reinput_timer
            .duration(),
        Duration::from_secs_f32(FIRST_REINPUT_DURATION)
    );
}

#[test]
fn test_swap_action() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(set_swap_action.system());

    world.insert_resource(SwapAction::default());
    assert_eq!(world.get_resource::<SwapAction>().unwrap().0, false);
    let mut input = Input::<KeyCode>::default();
    input.press(KeyCode::Space);
    world.insert_resource(input);
    update_stage.run(&mut world);
    assert_eq!(world.get_resource::<SwapAction>().unwrap().0, true);
}

#[test]
fn test_lift_action() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(set_lift_action.system());

    let mut time = Time::default();
    time.update();
    world.insert_resource(time);

    world.insert_resource(LiftAction::default());
    assert_eq!(world.get_resource::<LiftAction>().unwrap().lift, false);
    let mut input = Input::<KeyCode>::default();
    input.press(KeyCode::Return);
    world.insert_resource(input);
    update_stage.run(&mut world);
    assert_eq!(world.get_resource::<LiftAction>().unwrap().lift, true);
}
