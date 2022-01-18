use std::{collections::VecDeque, time::Duration};

use bevy::{
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
};
use bevy_easings::*;

use rand::prelude::*;

use crate::{
    actions::{LiftAction, MoveActions, SwapAction},
    loading::{
        BlockMaterials, BoardBottomCoverMaterials, BoardMaterials, BottomMaterials, CursorMaterials,
    },
    AppState,
};

pub struct IngamePlugin;

impl Plugin for IngamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameSpeed>()
            .add_plugin(bevy_easings::EasingsPlugin)
            .add_system_set(
                SystemSet::on_enter(AppState::InGame)
                    .with_system(setup_board)
                    .with_system(setup_board_bottom_cover)
                    .with_system(setup_chaincounter)
                    .with_system(setup_gamespeed),
            )
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
                    .label("move_set")
                    .before("fall_set")
                    .with_system(move_tag_block)
                    .with_system(custom_ease_system::<Moving>)
                    .with_system(move_block.label("move_block"))
                    .with_system(moving_to_fixed.after("move_block")),
            )
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
                    .label("fall_set")
                    .after("move_set")
                    .with_system(check_fall_block.label("check_fall"))
                    .with_system(fall_upward.label("fall_upward").after("check_fall"))
                    .with_system(
                        floating_to_fall
                            .label("floating_to_fall")
                            .after("fall_upward"),
                    )
                    .with_system(fall_block.label("fall_block").after("floating_to_fall"))
                    .with_system(stop_fall_block.label("stop_fall_block").after("fall_block"))
                    .with_system(
                        fixedprepare_to_fixed
                            .label("fixedprepare_to_fixed")
                            .after("stop_fall_block"),
                    ),
            )
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
                    .label("spawning_set")
                    .after("fall_set")
                    .with_system(generate_spawning_block.label("generate_spawning_block"))
                    .with_system(
                        spawning_to_fixed
                            .label("spawning_to_fixed")
                            .after("generate_spawning_block"),
                    )
                    .with_system(bottom_down.label("bottom_down").after("spawning_to_fixed")),
            )
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
                    .after("spawning_set")
                    .with_system(move_cursor)
                    .with_system(match_block.label("match_block"))
                    .with_system(
                        prepare_despawn_block
                            .label("prepare_despawn_block")
                            .after("match_block"),
                    )
                    .with_system(
                        despawn_block
                            .label("despawn_block")
                            .after("prepare_despawn_block"),
                    )
                    .with_system(remove_chain.label("remove_chain").after("despawn_block"))
                    .with_system(
                        reset_chain_counter
                            .label("reset_chain_counter")
                            .after("despawn_block"),
                    )
                    .with_system(
                        check_game_over
                            .label("check_game_over")
                            .after("reset_chain_counter"),
                    )
                    .with_system(
                        manual_liftup
                            .label("manual_liftup")
                            .after("check_game_over"),
                    )
                    .with_system(auto_liftup.label("auto_liftup").after("manual_liftup")),
            );
    }
}

const BOARD_WIDTH: usize = 6;
const BOARD_HEIGHT: usize = 13;
const BLOCK_SIZE: f32 = 50.0;

#[derive(Debug, PartialEq, Clone, Copy, Component)]
enum BlockColor {
    Red,
    Green,
    Blue,
    Yellow,
    Purple,
    Indigo,
}

#[derive(Debug, Component)]
struct Block;

#[derive(Debug, Component)]
struct Spawning;

#[derive(Debug, Component)]
struct Move(f32);

#[derive(Debug, Component, Default)]
struct Moving(f32);

impl Lerp for Moving {
    type Scalar = f32;
    fn lerp(&self, other: &Self, scalar: &Self::Scalar) -> Self {
        // std::f32::lerp is unstable
        Self(self.0 + (other.0 - self.0) * scalar)
    }
}

#[derive(Debug, Component)]
struct Fixed;
#[derive(Debug, Component)]
struct Matched;
#[derive(Debug, Component)]
struct FallPrepare;
#[derive(Debug, Component)]
struct Floating(Timer);
#[derive(Debug, Component)]
struct Fall;
#[derive(Debug, Component)]
struct FixedPrepare;
#[derive(Debug, Component)]
struct Despawining(Timer);

#[derive(Debug, Component)]
struct Chain(Timer);

#[derive(Debug, Component)]
struct Bottom;

#[derive(Debug, Component)]
struct Cursor;

#[derive(Debug, Component)]
struct Board;

#[derive(Debug, Component)]
struct BoardBottomCover;

#[derive(Debug, Component)]
struct CountTimer(Timer);

#[derive(Debug, Component)]
struct ChainCounter(u32);

#[derive(Default)]
struct GameSpeed {
    current: f32,
    origin: f32,
}

// TODO: divide function
fn setup_board(
    mut commands: Commands,
    board_materials: Res<BoardMaterials>,
    block_materials: Res<BlockMaterials>,
    bottom_materials: Res<BottomMaterials>,
    cursor_materials: Res<CursorMaterials>,
) {
    let board_transform = Transform {
        translation: Vec3::ZERO,
        ..Default::default()
    };
    let board_sprite = Sprite {
        custom_size: Some(Vec2::new(
            BOARD_WIDTH as f32 * BLOCK_SIZE,
            BOARD_HEIGHT as f32 * BLOCK_SIZE,
        )),
        ..Default::default()
    };
    let relative_x = board_transform.translation.x - board_sprite.custom_size.unwrap().x / 2.0
        + BLOCK_SIZE / 2.0;
    let relative_y = board_transform.translation.y - board_sprite.custom_size.unwrap().y / 2.0
        + BLOCK_SIZE / 2.0;
    let bottom_y = board_transform.translation.y
        - board_sprite.custom_size.unwrap().y / 2.0
        - BLOCK_SIZE / 2.0;

    let board_entity = commands
        .spawn_bundle(SpriteBundle {
            texture: board_materials.board_material.clone(),
            sprite: board_sprite,
            transform: board_transform,
            ..Default::default()
        })
        .insert(Board)
        .id();
    let patterns = [[
        [None, Some(3), None, None, None, None],
        [None, Some(0), None, Some(1), Some(0), None],
        [Some(0), Some(2), None, Some(2), Some(1), None],
        [Some(1), Some(2), None, Some(3), Some(2), None],
        [Some(3), Some(1), Some(3), Some(0), Some(3), Some(4)],
        [Some(2), Some(0), Some(4), Some(1), Some(0), Some(1)],
        [Some(4), Some(3), Some(2), Some(0), Some(4), Some(2)],
    ]];
    let mut rng = rand::thread_rng();
    let mut block_colors = vec![
        (BlockColor::Red, block_materials.red_material.clone()),
        (BlockColor::Green, block_materials.green_material.clone()),
        (BlockColor::Blue, block_materials.blue_material.clone()),
        (BlockColor::Yellow, block_materials.yellow_material.clone()),
        (BlockColor::Purple, block_materials.purple_material.clone()),
        // (BlockColor::Indigo, block_materials.indigo_material.clone()),
    ];

    // TODO: board entity
    block_colors.shuffle(&mut rng);

    if let Some(pattern) = patterns.iter().choose(&mut rng) {
        for (row_idx, row) in pattern.iter().rev().enumerate() {
            for (column_idx, one_block) in row.iter().enumerate() {
                match one_block {
                    None => {}
                    Some(num) => {
                        let block = commands
                            .spawn_bundle(SpriteBundle {
                                texture: block_colors[*num].1.clone(),
                                transform: Transform {
                                    translation: Vec3::new(
                                        relative_x + BLOCK_SIZE * column_idx as f32,
                                        relative_y + BLOCK_SIZE * row_idx as f32,
                                        0.0,
                                    ),
                                    ..Default::default()
                                },
                                ..Default::default()
                            })
                            .insert(Block)
                            .insert(block_colors[*num].0)
                            .insert(Fixed)
                            .id();
                        commands.entity(board_entity).push_children(&[block]);
                    }
                };
            }
        }
    };

    block_colors.shuffle(&mut rng);
    for row_idx in 0..2 {
        let mut previous_block_queue = VecDeque::with_capacity(2);
        for column_idx in 0..6 {
            let number = rng.gen_range(0..block_colors.len());
            let block = commands
                .spawn_bundle(SpriteBundle {
                    texture: block_colors[number].1.clone(),
                    transform: Transform {
                        translation: Vec3::new(
                            relative_x + BLOCK_SIZE * column_idx as f32,
                            bottom_y - BLOCK_SIZE * row_idx as f32,
                            0.0,
                        ),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Block)
                .insert(block_colors[number].0)
                .insert(Spawning)
                .id();
            commands.entity(board_entity).push_children(&[block]);
            let tmp_remove_block = Some(block_colors.remove(number));
            previous_block_queue.push_back(tmp_remove_block);
            if previous_block_queue.len() > 1 {
                if let Some(Some(back_color_block)) = previous_block_queue.pop_front() {
                    block_colors.push(back_color_block);
                }
            }
        }
    }
    let bottom = commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(BLOCK_SIZE * BOARD_WIDTH as f32, BLOCK_SIZE)),
                ..Default::default()
            },
            texture: bottom_materials.bottom_material.clone(),
            transform: Transform {
                translation: Vec3::new(0.0, bottom_y, 1.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Bottom)
        .id();
    commands.entity(board_entity).push_children(&[bottom]);
    let cursor = commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(BLOCK_SIZE * 2.0, BLOCK_SIZE)),
                ..Default::default()
            },
            texture: cursor_materials.cursor_material.clone(),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 1.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Cursor)
        .id();
    commands.entity(board_entity).push_children(&[cursor]);
    commands
        .spawn()
        .insert(CountTimer(Timer::from_seconds(1.0, false)));
}

fn setup_board_bottom_cover(
    mut commands: Commands,
    board_bottom_cover_materials: Res<BoardBottomCoverMaterials>,
) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: board_bottom_cover_materials
                .board_bottom_cover_material
                .clone(),
            sprite: Sprite {
                custom_size: Some(Vec2::new(BOARD_WIDTH as f32 * BLOCK_SIZE, 2.0 * BLOCK_SIZE)),
                ..Default::default()
            },
            transform: Transform {
                translation: Vec3::new(0.0, -375.0, 1.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(BoardBottomCover);
}

fn setup_chaincounter(mut commands: Commands) {
    commands.spawn().insert(ChainCounter(1));
}

fn setup_gamespeed(mut game_speed: ResMut<GameSpeed>) {
    game_speed.current = 10.0;
    game_speed.origin = 10.0;
}

fn move_cursor(actions: Res<MoveActions>, mut cursor: Query<&mut Transform, With<Cursor>>) {
    if let Some(cm) = actions.cursor_movement {
        let mut transform = cursor.single_mut();
        let movement = Vec3::new(cm.x * BLOCK_SIZE, cm.y * BLOCK_SIZE, 0.0);
        if transform.translation.x + movement.x > -125.0
            && transform.translation.x + movement.x < 125.0
        {
            transform.translation.x += movement.x;
        }
        if transform.translation.y + movement.y < 300.0
            && transform.translation.y + movement.y > -300.0
        {
            transform.translation.y += movement.y;
        }
    }
}

fn move_tag_block(
    action: Res<SwapAction>,
    mut commands: Commands,
    cursor: Query<&Transform, With<Cursor>>,
    mut block: Query<(Entity, &Transform, Option<&Fixed>), With<Block>>,
) {
    if action.0 {
        let cursor_transform = cursor.single();
        let x = cursor_transform.translation.x;
        let left_x = x - BLOCK_SIZE / 2.0;
        let right_x = x + BLOCK_SIZE / 2.0;
        let mut right_block = (None, None);
        let mut left_block = (None, None);
        let mut left_collide = false;
        let mut right_collide = false;

        for (block_entity, block_transform, fixed) in block.iter_mut() {
            if (block_transform.translation.y - cursor_transform.translation.y).abs()
                < BLOCK_SIZE / 2.0
            {
                // left target
                if (block_transform.translation.x - left_x).abs() < BLOCK_SIZE / 2.0 {
                    left_block = (Some(block_entity), fixed);
                }
                // right target
                if (block_transform.translation.x - right_x).abs() < BLOCK_SIZE / 2.0 {
                    right_block = (Some(block_entity), fixed);
                }
            }
            // fall block collision
            else if block_transform.translation.y - cursor_transform.translation.y < BLOCK_SIZE
                && block_transform.translation.y - cursor_transform.translation.y > 0.0
            {
                // left collision exists
                if (block_transform.translation.x - left_x).abs() < BLOCK_SIZE / 2.0 {
                    left_collide = true;
                }
                // right collision exsists
                else if (block_transform.translation.x - right_x).abs() < BLOCK_SIZE / 2.0 {
                    right_collide = true;
                }
            }
        }
        match (right_block, right_collide, left_block, left_collide) {
            // both exist and fixed -> remove fixed and insert move
            ((Some(right_entity), Some(_)), _, (Some(left_entity), Some(_)), _) => {
                commands
                    .entity(right_entity)
                    .remove::<Fixed>()
                    .insert(Move(left_x));
                commands
                    .entity(left_entity)
                    .remove::<Fixed>()
                    .insert(Move(right_x));
            }
            // one exists and fixed && no collide -> remove fixed and insert move
            ((Some(right_entity), Some(_)), _, (None, None), false) => {
                commands
                    .entity(right_entity)
                    .remove::<Fixed>()
                    .insert(Move(left_x));
            }
            ((None, None), false, (Some(left_entity), Some(_)), _) => {
                commands
                    .entity(left_entity)
                    .remove::<Fixed>()
                    .insert(Move(right_x));
            }
            // no fixed
            _ => {}
        }
    }
}

// Transform easing isn't match, because y-axis must be defined.
fn move_block(
    mut commands: Commands,
    mut block: Query<(Entity, &Transform, &Move), (With<Block>, With<Move>)>,
) {
    for (entity, transform, move_target) in block.iter_mut() {
        commands
            .entity(entity)
            .insert(Moving(transform.translation.x))
            .insert(Moving(transform.translation.x).ease_to(
                Moving(move_target.0),
                EaseMethod::Linear,
                EasingType::Once {
                    duration: std::time::Duration::from_secs_f32(0.04),
                },
            ))
            .remove::<Move>();
    }
}

fn moving_to_fixed(
    mut commands: Commands,
    mut block: Query<
        (
            Entity,
            &mut Transform,
            &Moving,
            Option<&EasingComponent<Moving>>,
        ),
        (With<Block>, With<Moving>),
    >,
) {
    for (entity, mut transform, moving, easing_component) in block.iter_mut() {
        match easing_component {
            Some(_) => {
                transform.translation.x = moving.0;
            }
            None => {
                commands.entity(entity).remove::<Moving>().insert(Fixed);
            }
        }
    }
}

// TODO: which fast?
// can not use collide
// match and fall check should be double loop...
// can not upwarding `Fall` state
fn match_block(
    mut commands: Commands,
    mut block: Query<
        (Entity, &Transform, &BlockColor),
        (With<Block>, With<Fixed>, With<BlockColor>),
    >,
    mut other_block: Query<
        (Entity, &Transform, &BlockColor),
        (With<Block>, With<Fixed>, With<BlockColor>),
    >,
) {
    let mut matched_entities: Vec<Entity> = Vec::new();
    for (entity, transform, block_color) in block.iter_mut() {
        let mut row_matched_entities = Vec::with_capacity(4);
        let mut column_matched_entities = Vec::with_capacity(4);

        for (other_entity, other_transform, other_block_color) in other_block.iter_mut() {
            // left next to
            if (transform.translation.x - other_transform.translation.x - BLOCK_SIZE).abs()
                < BLOCK_SIZE / 2.0
                && (transform.translation.y - other_transform.translation.y).abs()
                    < BLOCK_SIZE / 2.0
                && block_color == other_block_color
            {
                row_matched_entities.push(entity);
                row_matched_entities.push(other_entity);
            }
            // right next to
            if (transform.translation.x - other_transform.translation.x + BLOCK_SIZE).abs()
                < BLOCK_SIZE / 2.0
                && (transform.translation.y - other_transform.translation.y).abs()
                    < BLOCK_SIZE / 2.0
                && block_color == other_block_color
            {
                row_matched_entities.push(entity);
                row_matched_entities.push(other_entity);
            }
            // top next to
            if (transform.translation.y - other_transform.translation.y + BLOCK_SIZE).abs()
                < BLOCK_SIZE / 2.0
                && (transform.translation.x - other_transform.translation.x).abs()
                    < BLOCK_SIZE / 2.0
                && block_color == other_block_color
            {
                column_matched_entities.push(entity);
                column_matched_entities.push(other_entity);
            }
            // down next to
            if (transform.translation.y - other_transform.translation.y - BLOCK_SIZE).abs()
                < BLOCK_SIZE / 2.0
                && (transform.translation.x - other_transform.translation.x).abs()
                    < BLOCK_SIZE / 2.0
                && block_color == other_block_color
            {
                column_matched_entities.push(entity);
                column_matched_entities.push(other_entity);
            }
        }
        if row_matched_entities.len() == 4 {
            matched_entities.append(&mut row_matched_entities);
        }
        if column_matched_entities.len() == 4 {
            matched_entities.append(&mut column_matched_entities);
        }
    }
    for en in matched_entities {
        commands.entity(en).insert(Matched).remove::<Fixed>();
    }
}

fn prepare_despawn_block(
    mut commands: Commands,
    match_block: Query<(Entity, Option<&Chain>), (With<Block>, With<Matched>)>,
    mut chain_counter: Query<&mut ChainCounter>,
) {
    // TODO: despawning animation
    if match_block
        .iter()
        .collect::<Vec<_>>()
        .iter()
        .any(|(_, chain)| chain.is_some())
    {
        let mut cc = chain_counter.single_mut();
        cc.0 += 1;
    }

    let combo = match_block.iter().count();
    for (entity, _chain) in match_block.iter() {
        commands
            .entity(entity)
            .remove::<Matched>()
            .insert(Despawining(Timer::from_seconds(combo as f32 * 0.3, false)));
    }
}

// TODO: event?
// match_block event -> prepare_despawn_block event -> remove_chain event
fn remove_chain(
    mut commands: Commands,
    time: Res<Time>,
    mut chain_block: Query<(Entity, Option<&mut Chain>), (With<Block>, With<Fixed>)>,
) {
    for (entity, ch) in chain_block.iter_mut().filter(|(_en, ch)| ch.is_some()) {
        if let Some(mut chain) = ch {
            chain.0.tick(Duration::from_secs_f32(time.delta_seconds()));
            if chain.0.finished() {
                commands.entity(entity).remove::<Chain>();
            }
        }
    }
}

fn reset_chain_counter(
    chain_block: Query<&Chain, (With<Block>, With<Chain>)>,
    mut chain_counter: Query<&mut ChainCounter>,
) {
    if chain_block.iter().next().is_none() {
        let mut cc = chain_counter.single_mut();
        cc.0 = 1;
    }
}

fn despawn_block(
    mut commands: Commands,
    time: Res<Time>,
    mut despawning_block: Query<
        (Entity, &mut Despawining, &Transform),
        (With<Block>, With<Despawining>),
    >,
    other_block: Query<(Entity, &Transform), (With<Block>, Without<Despawining>)>,
) {
    for (despawning_entity, mut despawning, despawning_transform) in despawning_block.iter_mut() {
        despawning
            .0
            .tick(Duration::from_secs_f32(time.delta_seconds()));
        if despawning.0.just_finished() {
            commands.entity(despawning_entity).despawn();
            let mut chain_candidates = Vec::new();
            for (other_entity, other_transform) in other_block.iter() {
                if despawning_transform.translation.y < other_transform.translation.y
                    && (despawning_transform.translation.x - other_transform.translation.x).abs()
                        < BLOCK_SIZE / 2.0
                {
                    chain_candidates.push((other_entity, other_transform));
                }
            }
            chain_candidates.sort_unstable_by(|(_, trans_a), (_, trans_b)| {
                trans_a
                    .translation
                    .y
                    .partial_cmp(&trans_b.translation.y)
                    .unwrap()
            });
            let mut current_y = despawning_transform.translation.y;
            for (en, tr) in chain_candidates.iter() {
                if (tr.translation.y - BLOCK_SIZE - current_y).abs() < BLOCK_SIZE / 2.0 {
                    commands
                        .entity(*en)
                        .insert(Chain(Timer::from_seconds(0.04, false)));
                    current_y += BLOCK_SIZE;
                } else {
                    break;
                }
            }
        }
    }
}

fn check_fall_block(
    mut commands: Commands,
    mut block: Query<(Entity, &Transform), (With<Block>, With<Fixed>)>,
    mut other_block: Query<&Transform, With<Block>>,
) {
    // check is there block down next to?
    for (entity, transform) in block.iter_mut() {
        if transform.translation.y > -300.0 {
            let mut is_exist = false;
            for other_transform in other_block.iter_mut() {
                if (transform.translation.y - other_transform.translation.y - BLOCK_SIZE).abs()
                    < BLOCK_SIZE / 2.0
                    && (transform.translation.x - other_transform.translation.x).abs() < BLOCK_SIZE
                {
                    is_exist = true;
                    break;
                }
            }
            if !is_exist {
                commands
                    .entity(entity)
                    .remove::<Fixed>()
                    .insert(FallPrepare);
            }
        }
    }
}

fn fall_upward(
    mut commands: Commands,
    mut fallprepare_block: Query<(Entity, &Transform), (With<Block>, With<FallPrepare>)>,
    mut fixed_block: Query<(Entity, &Transform), (With<Block>, With<Fixed>)>,
) {
    for (fallprepare_entity, fallprepare_transform) in fallprepare_block.iter_mut() {
        let mut fall_block_candidates = vec![(fallprepare_entity, fallprepare_transform)];

        for (fixed_entity, fixed_transform) in fixed_block.iter_mut() {
            if fallprepare_transform.translation.y < fixed_transform.translation.y
                && (fallprepare_transform.translation.x - fixed_transform.translation.x).abs()
                    < BLOCK_SIZE / 2.0
            {
                fall_block_candidates.push((fixed_entity, fixed_transform));
            }
        }
        fall_block_candidates.sort_unstable_by(|(_ena, trans_a), (_enb, trans_b)| {
            trans_a
                .translation
                .y
                .partial_cmp(&trans_b.translation.y)
                .unwrap()
        });
        let mut iter = fall_block_candidates.iter().peekable();
        while let Some((en, tr)) = iter.next() {
            commands
                .entity(*en)
                .remove::<FallPrepare>()
                .remove::<Fixed>()
                .insert(Floating(Timer::from_seconds(0.02, false)));
            if let Some((_en, next_tr)) = iter.peek() {
                if (next_tr.translation.y - tr.translation.y).abs() > BLOCK_SIZE * 1.5 {
                    break;
                }
            }
        }
    }
}

fn floating_to_fall(
    mut commands: Commands,
    time: Res<Time>,
    mut floating_block: Query<(Entity, &mut Floating), (With<Floating>, With<Block>)>,
) {
    for (entity, mut floating) in floating_block.iter_mut() {
        floating
            .0
            .tick(Duration::from_secs_f32(time.delta_seconds()));
        if floating.0.just_finished() {
            commands.entity(entity).insert(Fall).remove::<Floating>();
        }
    }
}

// TODO: fix falling time
fn fall_block(time: Res<Time>, mut block: Query<&mut Transform, (With<Block>, With<Fall>)>) {
    for mut transform in block.iter_mut() {
        transform.translation.y -= 600.0 * time.delta_seconds();
    }
}

fn stop_fall_block(
    mut commands: Commands,
    mut fall_block: Query<(Entity, &mut Transform), (With<Block>, With<Fall>)>,
    other_block: Query<&Transform, (With<Block>, Without<Fall>)>,
) {
    for (fall_block_entity, mut fall_block_transform) in fall_block.iter_mut() {
        for other_block_transform in other_block.iter() {
            if let Some(Collision::Top) = collide(
                fall_block_transform.translation,
                Vec2::new(BLOCK_SIZE, BLOCK_SIZE),
                other_block_transform.translation,
                Vec2::new(BLOCK_SIZE, BLOCK_SIZE),
            ) {
                commands
                    .entity(fall_block_entity)
                    .insert(FixedPrepare)
                    .remove::<Fall>();
                // TODO: some animation
                fall_block_transform.translation.y =
                    other_block_transform.translation.y + BLOCK_SIZE;
            }
        }
    }
}

fn fixedprepare_to_fixed(
    mut commands: Commands,
    mut fixedprepare_block: Query<(Entity, &mut Transform), (With<Block>, With<FixedPrepare>)>,
    mut fall_block: Query<
        (Entity, &mut Transform),
        (With<Block>, With<Fall>, Without<FixedPrepare>),
    >,
) {
    for (fixedprepare_entity, fixedprepare_transform) in fixedprepare_block.iter_mut() {
        let fixedprepare_transform_vec = fixedprepare_transform.translation;
        let mut fixed_block_candidates = vec![(fixedprepare_entity, fixedprepare_transform)];

        for (fall_block_entity, fall_transform) in fall_block.iter_mut() {
            if fixedprepare_transform_vec.y < fall_transform.translation.y
                && (fixedprepare_transform_vec.x - fall_transform.translation.x).abs()
                    < BLOCK_SIZE / 2.0
            {
                fixed_block_candidates.push((fall_block_entity, fall_transform));
            }
        }
        fixed_block_candidates.sort_unstable_by(|(_, trans_a), (_, trans_b)| {
            trans_a
                .translation
                .y
                .partial_cmp(&trans_b.translation.y)
                .unwrap()
        });
        for (idx, (en, mut tr)) in fixed_block_candidates.into_iter().enumerate() {
            if tr.translation.y - (fixedprepare_transform_vec.y + BLOCK_SIZE * idx as f32)
                > BLOCK_SIZE * 0.5
            {
                break;
            }
            commands
                .entity(en)
                .remove::<FixedPrepare>()
                .remove::<Fall>()
                .insert(Fixed);
            tr.translation.y = fixedprepare_transform_vec.y + BLOCK_SIZE * idx as f32;
        }
    }
}

fn check_game_over(
    mut state: ResMut<State<AppState>>,
    count_timer: Query<&CountTimer>,
    block: Query<&Transform, With<Block>>,
) {
    let count_timer = count_timer.single();
    let max_height_block = block
        .iter()
        .max_by(|a_tr, b_tr| a_tr.translation.y.partial_cmp(&b_tr.translation.y).unwrap());
    if let Some(max_tr) = max_height_block {
        if count_timer.0.finished() && max_tr.translation.y > BLOCK_SIZE * 5.0 {
            state.set(AppState::GameOver).unwrap();
        }
    }
}

fn auto_liftup(
    time: Res<Time>,
    game_speed: Res<GameSpeed>,
    mut count_timer: Query<&mut CountTimer>,
    block: Query<
        Entity,
        (
            Without<Fixed>,
            Without<Spawning>,
            Without<Moving>,
            Without<Move>,
            With<Block>,
        ),
    >,
    mut target: Query<&mut Transform, Or<(With<Cursor>, With<Block>, With<Bottom>)>>,
) {
    let mut count_timer = count_timer.single_mut();
    count_timer
        .0
        .tick(Duration::from_secs_f32(time.delta_seconds()));
    if count_timer.0.finished() && block.iter().next().is_none() {
        for mut transform in target.iter_mut() {
            transform.translation.y += time.delta_seconds() * game_speed.current;
        }
    }
}

fn manual_liftup(
    lift_action: Res<LiftAction>,
    mut game_speed: ResMut<GameSpeed>,
    mut count_timer: Query<&mut CountTimer>,
) {
    if lift_action.lift {
        let mut count_timer = count_timer.single_mut();
        count_timer.0.set_duration(Duration::from_secs_f32(0.0));
        game_speed.current = 100.0;
    }
}

fn spawning_to_fixed(
    mut commands: Commands,
    spawning_block: Query<(Entity, &Transform), (With<Spawning>, With<Block>)>,
) {
    for (entity, transform) in spawning_block.iter() {
        if transform.translation.y > -300.0 {
            commands.entity(entity).remove::<Spawning>().insert(Fixed);
        }
    }
}

fn bottom_down(
    mut bottom: Query<&mut Transform, With<Bottom>>,
    mut game_speed: ResMut<GameSpeed>,
    time: Res<Time>,
) {
    for mut transform in bottom.iter_mut() {
        if transform.translation.y >= BLOCK_SIZE * -6.0 {
            transform.translation.y = BLOCK_SIZE * -7.0 + time.delta_seconds() * game_speed.current;
            game_speed.current = game_speed.origin;
        }
    }
}

fn generate_spawning_block(
    mut commands: Commands,
    game_speed: Res<GameSpeed>,
    time: Res<Time>,
    block_materials: Res<BlockMaterials>,
    board: Query<(Entity, &Transform, &Sprite), With<Board>>,
    spawning_block: Query<&Transform, (With<Block>, With<Spawning>)>,
) {
    for (board_entity, board_transform, board_sprite) in board.iter() {
        if spawning_block.iter().count() == 6 {
            if let Some(bottom_y) = spawning_block
                .iter()
                .min_by(|tr_a, tr_b| tr_a.translation.y.partial_cmp(&tr_b.translation.y).unwrap())
            {
                let relative_x = board_transform.translation.x
                    - board_sprite.custom_size.unwrap().x / 2.0
                    + BLOCK_SIZE / 2.0;
                let mut rng = rand::thread_rng();
                let mut block_colors = vec![
                    (BlockColor::Red, block_materials.red_material.clone()),
                    (BlockColor::Green, block_materials.green_material.clone()),
                    (BlockColor::Blue, block_materials.blue_material.clone()),
                    (BlockColor::Yellow, block_materials.yellow_material.clone()),
                    (BlockColor::Purple, block_materials.purple_material.clone()),
                    // (BlockColor::Indigo, block_materials.indigo_material.clone()),
                ];
                block_colors.shuffle(&mut rng);
                let mut previous_block_queue = VecDeque::with_capacity(2);
                for column_idx in 0..6 {
                    let number = rng.gen_range(0..block_colors.len());
                    let block = commands
                        .spawn_bundle(SpriteBundle {
                            texture: block_colors[number].1.clone(),
                            transform: Transform {
                                translation: Vec3::new(
                                    relative_x + BLOCK_SIZE * column_idx as f32,
                                    bottom_y.translation.y - BLOCK_SIZE
                                        + time.delta_seconds() * game_speed.current,
                                    0.0,
                                ),
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .insert(Block)
                        .insert(block_colors[number].0)
                        .insert(Spawning)
                        .id();
                    commands.entity(board_entity).push_children(&[block]);
                    let tmp_remove_block = Some(block_colors.remove(number));
                    previous_block_queue.push_back(tmp_remove_block);
                    if previous_block_queue.len() > 1 {
                        if let Some(Some(back_color_block)) = previous_block_queue.pop_front() {
                            block_colors.push(back_color_block);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn test_setup_board() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(setup_board);

    world.insert_resource(BoardMaterials {
        board_material: Handle::<Image>::default(),
    });
    world.insert_resource(BlockMaterials {
        red_material: Handle::<Image>::default(),
        green_material: Handle::<Image>::default(),
        blue_material: Handle::<Image>::default(),
        yellow_material: Handle::<Image>::default(),
        purple_material: Handle::<Image>::default(),
        indigo_material: Handle::<Image>::default(),
    });
    world.insert_resource(BottomMaterials {
        bottom_material: Handle::<Image>::default(),
    });
    world.insert_resource(CursorMaterials {
        cursor_material: Handle::<Image>::default(),
    });

    update_stage.run(&mut world);
    assert_eq!(world.query::<&Board>().iter(&world).len(), 1);
    assert_eq!(world.query::<&Cursor>().iter(&world).len(), 1);
    assert!(world.query::<&Block>().iter(&world).len() > 5);
    assert_eq!(world.query::<(&Block, &Spawning)>().iter(&world).len(), 12);
    assert_eq!(world.query::<&Bottom>().iter(&world).len(), 1);
}

#[test]
fn test_left_move_cursor() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(move_cursor);
    world.spawn().insert(Board);
    world.spawn().insert(Cursor).insert_bundle(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(BLOCK_SIZE * 2.0, BLOCK_SIZE)),
            ..Default::default()
        },
        transform: Transform {
            translation: Vec3::ZERO,
            ..Default::default()
        },
        ..Default::default()
    });
    world.insert_resource(MoveActions::default());

    assert_eq!(world.query::<&Cursor>().iter(&world).len(), 1);
    assert_eq!(
        world
            .query::<(&Cursor, &Transform)>()
            .iter(&world)
            .next()
            .unwrap()
            .1
            .translation,
        Vec3::ZERO
    );
    world
        .get_resource_mut::<MoveActions>()
        .unwrap()
        .cursor_movement = Some(Vec2::new(-1.0, 0.0));
    update_stage.run(&mut world);
    assert_eq!(
        world
            .query::<(&Cursor, &Transform)>()
            .iter(&world)
            .next()
            .unwrap()
            .1
            .translation,
        Vec3::new(-1.0 * BLOCK_SIZE, 0.0, 0.0)
    );
    update_stage.run(&mut world);
    assert_eq!(
        world
            .query::<(&Cursor, &Transform)>()
            .iter(&world)
            .next()
            .unwrap()
            .1
            .translation,
        Vec3::new(-2.0 * BLOCK_SIZE, 0.0, 0.0)
    );
    // can't move left more
    update_stage.run(&mut world);
    assert_eq!(
        world
            .query::<(&Cursor, &Transform)>()
            .iter(&world)
            .next()
            .unwrap()
            .1
            .translation,
        Vec3::new(-2.0 * BLOCK_SIZE, 0.0, 0.0)
    );
}

#[test]
fn test_right_move_cursor() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(move_cursor);
    world.spawn().insert(Board);
    world.spawn().insert(Cursor).insert_bundle(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(BLOCK_SIZE * 2.0, BLOCK_SIZE)),
            ..Default::default()
        },
        transform: Transform {
            translation: Vec3::ZERO,
            ..Default::default()
        },
        ..Default::default()
    });
    world.insert_resource(MoveActions::default());

    assert_eq!(world.query::<&Cursor>().iter(&world).len(), 1);
    assert_eq!(
        world
            .query::<(&Cursor, &Transform)>()
            .iter(&world)
            .next()
            .unwrap()
            .1
            .translation,
        Vec3::ZERO
    );
    world
        .get_resource_mut::<MoveActions>()
        .unwrap()
        .cursor_movement = Some(Vec2::new(1.0, 0.0));
    update_stage.run(&mut world);
    assert_eq!(
        world
            .query::<(&Cursor, &Transform)>()
            .iter(&world)
            .next()
            .unwrap()
            .1
            .translation,
        Vec3::new(BLOCK_SIZE, 0.0, 0.0)
    );
    update_stage.run(&mut world);
    assert_eq!(
        world
            .query::<(&Cursor, &Transform)>()
            .iter(&world)
            .next()
            .unwrap()
            .1
            .translation,
        Vec3::new(2.0 * BLOCK_SIZE, 0.0, 0.0)
    );
    // can't move right more
    update_stage.run(&mut world);
    assert_eq!(
        world
            .query::<(&Cursor, &Transform)>()
            .iter(&world)
            .next()
            .unwrap()
            .1
            .translation,
        Vec3::new(2.0 * BLOCK_SIZE, 0.0, 0.0)
    );
}

#[test]
fn test_down_move_cursor() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(move_cursor);

    world.spawn().insert(Board);
    world.spawn().insert(Cursor).insert_bundle(SpriteBundle {
        transform: Transform {
            translation: Vec3::ZERO,
            ..Default::default()
        },
        ..Default::default()
    });
    world.insert_resource(MoveActions::default());

    assert_eq!(world.query::<&Cursor>().iter(&world).len(), 1);
    assert_eq!(
        world
            .query::<(&Cursor, &Transform)>()
            .iter(&world)
            .next()
            .unwrap()
            .1
            .translation,
        Vec3::ZERO
    );
    world
        .get_resource_mut::<MoveActions>()
        .unwrap()
        .cursor_movement = Some(Vec2::new(0.0, -1.0));

    update_stage.run(&mut world);
    assert_eq!(
        world
            .query::<(&Cursor, &Transform)>()
            .iter(&world)
            .next()
            .unwrap()
            .1
            .translation,
        Vec3::new(0.0, -1.0 * BLOCK_SIZE, 0.0)
    );

    for _ in 0..7 {
        update_stage.run(&mut world);
    }
    assert_eq!(
        world
            .query::<(&Cursor, &Transform)>()
            .iter(&world)
            .next()
            .unwrap()
            .1
            .translation,
        Vec3::new(0.0, -5.0 * BLOCK_SIZE, 0.0)
    );
}

#[test]
fn test_up_move_cursor() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(move_cursor);

    world.spawn().insert(Board);
    world.spawn().insert(Cursor).insert_bundle(SpriteBundle {
        transform: Transform {
            translation: Vec3::ZERO,
            ..Default::default()
        },
        ..Default::default()
    });
    world.insert_resource(MoveActions::default());

    assert_eq!(world.query::<&Cursor>().iter(&world).len(), 1);
    assert_eq!(
        world
            .query::<(&Cursor, &Transform)>()
            .iter(&world)
            .next()
            .unwrap()
            .1
            .translation,
        Vec3::ZERO
    );
    world
        .get_resource_mut::<MoveActions>()
        .unwrap()
        .cursor_movement = Some(Vec2::new(0.0, 1.0));

    update_stage.run(&mut world);
    assert_eq!(
        world
            .query::<(&Cursor, &Transform)>()
            .iter(&world)
            .next()
            .unwrap()
            .1
            .translation,
        Vec3::new(0.0, BLOCK_SIZE, 0.0)
    );

    for _ in 0..7 {
        update_stage.run(&mut world);
    }

    assert_eq!(
        world
            .query::<(&Cursor, &Transform)>()
            .iter(&world)
            .next()
            .unwrap()
            .1
            .translation,
        Vec3::new(0.0, 5.0 * BLOCK_SIZE, 0.0)
    );
}

#[test]
fn test_move_tag_block_both_fix() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(move_tag_block);

    world.spawn().insert(Board).insert_bundle(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(
                BOARD_WIDTH as f32 * BLOCK_SIZE,
                BOARD_HEIGHT as f32 * BLOCK_SIZE,
            )),
            ..Default::default()
        },
        transform: Transform {
            translation: Vec3::ZERO,
            ..Default::default()
        },
        ..Default::default()
    });
    world.spawn().insert(Cursor).insert_bundle(SpriteBundle {
        transform: Transform {
            translation: Vec3::ZERO,
            ..Default::default()
        },
        ..Default::default()
    });
    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0, 0.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(BlockColor::Red)
        .insert(Fixed);
    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(-1.0 * BLOCK_SIZE / 2.0, 0.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(BlockColor::Blue)
        .insert(Fixed);

    world.insert_resource(SwapAction(true));
    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 2);

    update_stage.run(&mut world);
    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 0);
    assert_eq!(world.query::<(&Block, &Move)>().iter(&world).len(), 2);
}

#[test]
fn test_move_tag_block_left_one_fix() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(move_tag_block);

    world.spawn().insert(Board).insert_bundle(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(
                BOARD_WIDTH as f32 * BLOCK_SIZE,
                BOARD_HEIGHT as f32 * BLOCK_SIZE,
            )),
            ..Default::default()
        },
        transform: Transform {
            translation: Vec3::ZERO,
            ..Default::default()
        },
        ..Default::default()
    });
    world.spawn().insert(Cursor).insert_bundle(SpriteBundle {
        transform: Transform {
            translation: Vec3::ZERO,
            ..Default::default()
        },
        ..Default::default()
    });
    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0, 0.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(BlockColor::Red)
        .insert(Fixed);

    world.insert_resource(SwapAction(true));
    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 1);

    update_stage.run(&mut world);
    assert_eq!(world.query::<(&Block, &Move)>().iter(&world).len(), 1);
}

#[test]
fn test_move_tag_block_right_one_fix() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(move_tag_block);

    world.spawn().insert(Board).insert_bundle(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(
                BOARD_WIDTH as f32 * BLOCK_SIZE,
                BOARD_HEIGHT as f32 * BLOCK_SIZE,
            )),
            ..Default::default()
        },
        transform: Transform {
            translation: Vec3::ZERO,
            ..Default::default()
        },
        ..Default::default()
    });
    world.spawn().insert(Cursor).insert_bundle(SpriteBundle {
        transform: Transform {
            translation: Vec3::ZERO,
            ..Default::default()
        },
        ..Default::default()
    });
    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(-1.0 * BLOCK_SIZE / 2.0, 0.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(BlockColor::Red)
        .insert(Fixed);

    world.insert_resource(SwapAction(true));
    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 1);

    update_stage.run(&mut world);
    assert_eq!(world.query::<(&Block, &Move)>().iter(&world).len(), 1);
}

#[test]
fn test_move_tag_block_there_is_collide() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(move_tag_block);

    world.spawn().insert(Board).insert_bundle(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(
                BOARD_WIDTH as f32 * BLOCK_SIZE,
                BOARD_HEIGHT as f32 * BLOCK_SIZE,
            )),
            ..Default::default()
        },
        transform: Transform {
            translation: Vec3::ZERO,
            ..Default::default()
        },
        ..Default::default()
    });
    world.spawn().insert(Cursor).insert_bundle(SpriteBundle {
        transform: Transform {
            translation: Vec3::ZERO,
            ..Default::default()
        },
        ..Default::default()
    });
    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0, 1.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(BlockColor::Red);
    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(-1.0 * BLOCK_SIZE / 2.0, 0.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(BlockColor::Red)
        .insert(Fixed);

    world.insert_resource(SwapAction(true));

    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 1);

    update_stage.run(&mut world);
    assert_eq!(world.query::<(&Block, &Move)>().iter(&world).len(), 0);
}

#[test]
fn test_move_tag_block_not_fixed_block() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(move_tag_block);

    world.spawn().insert(Board).insert_bundle(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(
                BOARD_WIDTH as f32 * BLOCK_SIZE,
                BOARD_HEIGHT as f32 * BLOCK_SIZE,
            )),
            ..Default::default()
        },
        transform: Transform {
            translation: Vec3::ZERO,
            ..Default::default()
        },
        ..Default::default()
    });
    world.spawn().insert(Cursor).insert_bundle(SpriteBundle {
        transform: Transform {
            translation: Vec3::ZERO,
            ..Default::default()
        },
        ..Default::default()
    });
    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0, 0.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(BlockColor::Red)
        .insert(Fixed);
    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(-1.0 * BLOCK_SIZE / 2.0, 0.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(BlockColor::Blue);

    world.insert_resource(SwapAction(true));

    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 1);

    update_stage.run(&mut world);
    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 1);
    assert_eq!(world.query::<(&Block, &Move)>().iter(&world).len(), 0);
}

#[test]
fn test_move_block() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(move_block);

    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0, 0.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(BlockColor::Red)
        .insert(Move(-1.0 * BLOCK_SIZE / 2.0));
    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(-1.0 * BLOCK_SIZE / 2.0, 0.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(BlockColor::Blue)
        .insert(Move(BLOCK_SIZE / 2.0));

    assert_eq!(world.query::<(&Block, &Move)>().iter(&world).len(), 2);
    update_stage.run(&mut world);
    assert_eq!(world.query::<(&Block, &Move)>().iter(&world).len(), 0);
    assert_eq!(world.query::<(&Block, &Moving)>().iter(&world).len(), 2);
}

#[test]
fn test_match_row_block_three_matched() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(match_block);

    for i in 0..3 {
        world
            .spawn()
            .insert(Block)
            .insert_bundle(SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(
                        BLOCK_SIZE / 2.0 + BLOCK_SIZE * (i - 3) as f32,
                        -300.0,
                        0.0,
                    ),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(BlockColor::Red)
            .insert(Fixed);
    }
    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 3);
    update_stage.run(&mut world);
    assert_eq!(world.query::<(&Block, &Matched)>().iter(&world).len(), 3);
    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 0);
}

#[test]
fn test_match_row_block_four_matched() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(match_block);

    for i in 0..4 {
        world
            .spawn()
            .insert(Block)
            .insert_bundle(SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(
                        BLOCK_SIZE / 2.0 + BLOCK_SIZE * (i - 3) as f32,
                        -300.0,
                        0.0,
                    ),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(BlockColor::Red)
            .insert(Fixed);
    }
    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 4);
    update_stage.run(&mut world);
    assert_eq!(world.query::<(&Block, &Matched)>().iter(&world).len(), 4);
    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 0);
}

#[test]
fn test_match_row_block_three_matched_only() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(match_block);

    for i in 0..5 {
        match i {
            0 | 1 | 2 | 4 => {
                world
                    .spawn()
                    .insert(Block)
                    .insert_bundle(SpriteBundle {
                        transform: Transform {
                            translation: Vec3::new(
                                BLOCK_SIZE / 2.0 + BLOCK_SIZE * (i - 3) as f32,
                                -300.0,
                                0.0,
                            ),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .insert(BlockColor::Red)
                    .insert(Fixed);
            }
            3 => {
                world
                    .spawn()
                    .insert(Block)
                    .insert_bundle(SpriteBundle {
                        transform: Transform {
                            translation: Vec3::new(
                                BLOCK_SIZE / 2.0 + BLOCK_SIZE * (i - 3) as f32,
                                -300.0,
                                0.0,
                            ),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .insert(BlockColor::Blue)
                    .insert(Fixed);
            }
            _ => {
                unreachable!()
            }
        }
    }

    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 5);
    update_stage.run(&mut world);
    assert_eq!(world.query::<(&Block, &Matched)>().iter(&world).len(), 3);
    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 2);
}

#[test]
fn test_match_row_block_five_matched() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(match_block);

    for i in 0..5 {
        world
            .spawn()
            .insert(Block)
            .insert_bundle(SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(
                        BLOCK_SIZE / 2.0 + BLOCK_SIZE * (i - 3) as f32,
                        -300.0,
                        0.0,
                    ),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(BlockColor::Red)
            .insert(Fixed);
    }
    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 5);
    update_stage.run(&mut world);
    assert_eq!(world.query::<(&Block, &Matched)>().iter(&world).len(), 5);
    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 0);
}

#[test]
fn test_match_row_block_six_matched() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(match_block);

    for i in 0..6 {
        world
            .spawn()
            .insert(Block)
            .insert_bundle(SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(
                        BLOCK_SIZE / 2.0 + BLOCK_SIZE * (i - 3) as f32,
                        -300.0,
                        0.0,
                    ),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(BlockColor::Red)
            .insert(Fixed);
    }
    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 6);
    update_stage.run(&mut world);
    assert_eq!(world.query::<(&Block, &Matched)>().iter(&world).len(), 6);
    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 0);
}

#[test]
fn test_match_row_block_six_matched_two_colors() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(match_block);

    for i in 0..6 {
        if i < 3 {
            world
                .spawn()
                .insert(Block)
                .insert_bundle(SpriteBundle {
                    transform: Transform {
                        translation: Vec3::new(
                            BLOCK_SIZE / 2.0 + BLOCK_SIZE * (i - 3) as f32,
                            -300.0,
                            0.0,
                        ),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(BlockColor::Red)
                .insert(Fixed);
        } else {
            world
                .spawn()
                .insert(Block)
                .insert_bundle(SpriteBundle {
                    transform: Transform {
                        translation: Vec3::new(
                            BLOCK_SIZE / 2.0 + BLOCK_SIZE * (i - 3) as f32,
                            -300.0,
                            0.0,
                        ),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(BlockColor::Blue)
                .insert(Fixed);
        }
    }
    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 6);
    update_stage.run(&mut world);
    assert_eq!(world.query::<(&Block, &Matched)>().iter(&world).len(), 6);
    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 0);
}

#[test]
fn test_no_match_block() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(match_block);

    world
        .spawn()
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0 - BLOCK_SIZE * 2.0, -300.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Block)
        .insert(BlockColor::Red)
        .insert(Fixed);
    world
        .spawn()
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0 - BLOCK_SIZE, -300.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Block)
        .insert(BlockColor::Red)
        .insert(Fixed);
    world
        .spawn()
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0 + BLOCK_SIZE, -300.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Block)
        .insert(BlockColor::Blue)
        .insert(Fixed);
    world
        .spawn()
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0 + BLOCK_SIZE * 2.0, -300.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Block)
        .insert(BlockColor::Red)
        .insert(Fixed);
    world
        .spawn()
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0 + BLOCK_SIZE, BLOCK_SIZE - 300.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Block)
        .insert(BlockColor::Red)
        .insert(Fixed);
    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 5);
    update_stage.run(&mut world);
    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 5);
    assert_eq!(world.query::<(&Block, &Matched)>().iter(&world).len(), 0);
}

#[test]
fn test_match_column_block_three_matched() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(match_block);

    for i in 0..3 {
        world
            .spawn()
            .insert(Block)
            .insert_bundle(SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(BLOCK_SIZE / 2.0, 0.0 - BLOCK_SIZE * i as f32, 0.0),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(BlockColor::Red)
            .insert(Fixed);
    }
    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 3);
    update_stage.run(&mut world);
    assert_eq!(world.query::<(&Block, &Matched)>().iter(&world).len(), 3);
    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 0);
}

#[test]
fn test_match_row_and_column_block_five_matched() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(match_block);

    // row
    for i in 0..3 {
        world
            .spawn()
            .insert(Block)
            .insert_bundle(SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(
                        BLOCK_SIZE / 2.0 + BLOCK_SIZE * (i - 3) as f32,
                        0.0,
                        0.0,
                    ),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(BlockColor::Red)
            .insert(Fixed);
    }
    // column
    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0 - BLOCK_SIZE * 2.0, BLOCK_SIZE, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(BlockColor::Red)
        .insert(Fixed);
    // column
    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0 - BLOCK_SIZE * 2.0, -1.0 * BLOCK_SIZE, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(BlockColor::Red)
        .insert(Fixed);
    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 5);
    update_stage.run(&mut world);
    assert_eq!(world.query::<(&Block, &Matched)>().iter(&world).len(), 5);
    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 0);
}

#[test]
fn test_prepare_despawn_block() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(prepare_despawn_block);

    world.spawn().insert(Block).insert(Matched);
    let chain_counter = world.spawn().insert(ChainCounter(1)).id();
    update_stage.run(&mut world);
    assert_eq!(world.query::<(&Block, &Matched)>().iter(&world).len(), 0);
    assert_eq!(
        world.query::<(&Block, &Despawining)>().iter(&world).len(),
        1
    );
    assert_eq!(world.get::<ChainCounter>(chain_counter).unwrap().0, 1);
}

#[test]
fn test_prepare_despawn_block_chain() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(prepare_despawn_block);

    world
        .spawn()
        .insert(Block)
        .insert(Matched)
        .insert(Chain(Timer::from_seconds(0.04, false)));
    let chain_counter = world.spawn().insert(ChainCounter(1)).id();
    update_stage.run(&mut world);
    assert_eq!(world.query::<(&Block, &Matched)>().iter(&world).len(), 0);
    assert_eq!(
        world.query::<(&Block, &Despawining)>().iter(&world).len(),
        1
    );
    assert_eq!(world.get::<ChainCounter>(chain_counter).unwrap().0, 2);
}

#[test]
fn test_remove_chain() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(remove_chain);
    let mut time = Time::default();
    time.update();
    world.insert_resource(time);
    world
        .spawn()
        .insert(Block)
        .insert(Fixed)
        .insert(Chain(Timer::from_seconds(0.0, false)));
    assert_eq!(world.query::<(&Block, &Chain)>().iter(&world).len(), 1);
    update_stage.run(&mut world);
    assert_eq!(world.query::<(&Block, &Chain)>().iter(&world).len(), 0);
}

#[test]
fn test_remove_chain_not_fixed() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(remove_chain);
    let mut time = Time::default();
    time.update();
    world.insert_resource(time);
    world
        .spawn()
        .insert(Block)
        .insert(Matched)
        .insert(Chain(Timer::from_seconds(0.0, false)));
    world
        .spawn()
        .insert(Block)
        .insert(Despawining(Timer::from_seconds(1.0, false)))
        .insert(Chain(Timer::from_seconds(0.0, false)));

    assert_eq!(world.query::<(&Block, &Chain)>().iter(&world).len(), 2);
    update_stage.run(&mut world);
    assert_eq!(world.query::<(&Block, &Chain)>().iter(&world).len(), 2);
}

#[test]
fn test_reset_chain_counter() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(reset_chain_counter);
    let chain_counter = world.spawn().insert(ChainCounter(2)).id();
    update_stage.run(&mut world);
    assert_eq!(world.get::<ChainCounter>(chain_counter).unwrap().0, 1);
}

#[test]
fn test_reset_chain_counter_not_reset() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(reset_chain_counter);
    let chain_counter = world.spawn().insert(ChainCounter(2)).id();
    world
        .spawn()
        .insert(Block)
        .insert(Chain(Timer::from_seconds(0.04, false)));
    update_stage.run(&mut world);
    assert_eq!(world.get::<ChainCounter>(chain_counter).unwrap().0, 2);
}

#[test]
fn test_despawn_block() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(despawn_block);
    let time = Time::default();
    world.insert_resource(time);

    let block = world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::ZERO,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Despawining(Timer::from_seconds(0.0, false)))
        .id();

    update_stage.run(&mut world);
    assert!(world.get::<Block>(block).is_none());
}

#[test]
fn test_despawn_block_add_chain() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(despawn_block);
    let time = Time::default();
    world.insert_resource(time);

    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::ZERO,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Despawining(Timer::from_seconds(0.0, false)));
    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, BLOCK_SIZE, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Fixed);
    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, BLOCK_SIZE * 3.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Fixed);

    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, BLOCK_SIZE * -1.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Fixed);

    update_stage.run(&mut world);
    assert_eq!(world.query::<(&Block, &Chain)>().iter(&world).len(), 1);
}

#[test]
fn test_check_fall_block() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(check_fall_block);
    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0, 0.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Fixed);
    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 1);
    update_stage.run(&mut world);
    assert_eq!(
        world.query::<(&Block, &FallPrepare)>().iter(&world).len(),
        1
    );
}

#[test]
fn test_check_fall_block_there_isnot_between_block() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(check_fall_block);
    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0, BLOCK_SIZE * -5.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Fixed);
    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0 - BLOCK_SIZE, BLOCK_SIZE * -6.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Fixed);
    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0 + BLOCK_SIZE, BLOCK_SIZE * -6.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Fixed);
    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 3);
    update_stage.run(&mut world);
    assert_eq!(
        world.query::<(&Block, &FallPrepare)>().iter(&world).len(),
        1
    );
}

#[test]
fn test_check_fall_block_there_is_between_block() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(check_fall_block);
    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0, BLOCK_SIZE * -5.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Fixed);
    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0 - BLOCK_SIZE, BLOCK_SIZE * -6.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Fixed);
    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0, BLOCK_SIZE * -6.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Fixed);

    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0 + BLOCK_SIZE, BLOCK_SIZE * -6.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Fixed);
    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 4);
    update_stage.run(&mut world);
    assert_eq!(world.query::<(&Block, &Fall)>().iter(&world).len(), 0);
}

#[test]
fn test_check_fall_block_there_is_start_block_move() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(check_fall_block);
    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0, BLOCK_SIZE * -5.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Fixed);
    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0 - BLOCK_SIZE, BLOCK_SIZE * -6.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Move(BLOCK_SIZE / 2.0));
    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0, BLOCK_SIZE * -6.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Move(BLOCK_SIZE / 2.0));

    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0 + BLOCK_SIZE, BLOCK_SIZE * -6.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Fixed);
    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 2);
    update_stage.run(&mut world);
    assert_eq!(world.query::<(&Block, &Fall)>().iter(&world).len(), 0);
}

#[test]
fn test_check_fall_block_there_is_between_block_move() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(check_fall_block);
    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0, BLOCK_SIZE * -5.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Fixed);
    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0 - BLOCK_SIZE + 1.0, BLOCK_SIZE * -6.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Fixed);
    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0 - 1.0, BLOCK_SIZE * -6.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Fixed);

    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0 + BLOCK_SIZE, BLOCK_SIZE * -6.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Fixed);
    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 4);
    update_stage.run(&mut world);
    assert_eq!(world.query::<(&Block, &Fall)>().iter(&world).len(), 0);
}

#[test]
fn test_check_fall_block_bottom_block_not_fall() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(check_fall_block);
    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0, BLOCK_SIZE * -6.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Fixed);
    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 1);
    update_stage.run(&mut world);
    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 1);
}

#[test]
fn test_fall_upward() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(fall_upward);

    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0, 0.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(FallPrepare);
    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0, BLOCK_SIZE, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Fixed);

    update_stage.run(&mut world);
    assert_eq!(world.query::<(&Block, &Floating)>().iter(&world).len(), 2);
}

#[test]
fn test_fall_upward_divide() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(fall_upward);

    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0, 0.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(FallPrepare);
    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0, BLOCK_SIZE, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Fixed);
    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0, BLOCK_SIZE * 3.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Fixed);

    update_stage.run(&mut world);
    assert_eq!(world.query::<(&Block, &Floating)>().iter(&world).len(), 2);
    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 1);
}

#[test]
fn test_floating_to_fall() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(floating_to_fall);

    let time = Time::default();
    world.insert_resource(time);
    world
        .spawn()
        .insert(Block)
        .insert(Floating(Timer::from_seconds(0.0, false)));

    update_stage.run(&mut world);
    assert_eq!(world.query::<(&Block, &Fall)>().iter(&world).len(), 1);
}

#[test]
fn test_stop_fall_block() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(stop_fall_block);
    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0, 99.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Fall);
    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0, 50.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Fixed);

    assert_eq!(world.query::<(&Block, &Fall)>().iter(&world).len(), 1);
    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 1);
    update_stage.run(&mut world);
    assert_eq!(world.query::<(&Block, &Fall)>().iter(&world).len(), 0);
    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 1);
    assert_eq!(
        world.query::<(&Block, &FixedPrepare)>().iter(&world).len(),
        1
    );
}

#[test]
fn test_fixedprepare_to_fixed() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(fixedprepare_to_fixed);
    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0, 0.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(FixedPrepare);
    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0, BLOCK_SIZE, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Fall);
    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0, BLOCK_SIZE * 3.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Fall);
    update_stage.run(&mut world);
    assert_eq!(
        world.query::<(&Block, &FixedPrepare)>().iter(&world).len(),
        0
    );
    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 2);
    assert_eq!(world.query::<(&Block, &Fall)>().iter(&world).len(), 1);
}

#[test]
fn test_auto_liftup() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(auto_liftup);
    let mut time = Time::default();
    time.update();
    world.insert_resource(time);
    world.insert_resource(GameSpeed {
        current: 10.0,
        ..Default::default()
    });
    world
        .spawn()
        .insert(CountTimer(Timer::from_seconds(0.0, false)));

    let block = world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0, 0.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Fixed)
        .id();
    assert_eq!(world.get::<Transform>(block).unwrap().translation.y, 0.0);

    world.get_resource_mut::<Time>().unwrap().update();
    update_stage.run(&mut world);
    assert_ne!(world.get::<Transform>(block).unwrap().translation.y, 0.0);
}

#[test]
fn test_auto_liftup_stop_with_timer() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(auto_liftup);
    let mut time = Time::default();
    time.update();
    world.insert_resource(time);
    world.insert_resource(GameSpeed {
        current: 10.0,
        ..Default::default()
    });
    world
        .spawn()
        .insert(CountTimer(Timer::from_seconds(1.0, false)));

    let block = world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0, 0.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Fixed)
        .id();
    assert_eq!(world.get::<Transform>(block).unwrap().translation.y, 0.0);

    world.get_resource_mut::<Time>().unwrap().update();
    update_stage.run(&mut world);
    assert_eq!(world.get::<Transform>(block).unwrap().translation.y, 0.0);
}

#[test]
fn test_auto_liftup_stop_with_fall_block() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(auto_liftup);
    let mut time = Time::default();
    time.update();
    world.insert_resource(time);
    world.insert_resource(GameSpeed {
        current: 10.0,
        ..Default::default()
    });
    world
        .spawn()
        .insert(CountTimer(Timer::from_seconds(0.0, false)));

    let block = world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0, 0.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Fall)
        .id();
    assert_eq!(world.get::<Transform>(block).unwrap().translation.y, 0.0);
    world.get_resource_mut::<Time>().unwrap().update();
    update_stage.run(&mut world);
    assert_eq!(world.get::<Transform>(block).unwrap().translation.y, 0.0);
}

#[test]
fn test_manual_liftup() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(manual_liftup);
    world.insert_resource(GameSpeed {
        current: 10.0,
        origin: 10.0,
    });
    world.insert_resource(LiftAction {
        lift: true,
        ..Default::default()
    });
    let count_timer = world
        .spawn()
        .insert(CountTimer(Timer::from_seconds(1.0, false)))
        .id();

    update_stage.run(&mut world);
    assert_eq!(world.get_resource::<GameSpeed>().unwrap().current, 100.0);
    assert_eq!(
        world.get::<CountTimer>(count_timer).unwrap().0.duration(),
        Duration::from_secs_f32(0.0)
    );
}

#[ignore = "how to change state?"]
#[test]
fn test_check_game_over() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(check_game_over);
    let app_state = State::new(AppState::InGame);
    world.insert_resource(app_state);
    let count_timer = world
        .spawn()
        .insert(CountTimer(Timer::from_seconds(0.0, false)))
        .id();

    world
        .spawn()
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0, BLOCK_SIZE * 5.1, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Block);
    assert_eq!(
        world.get_resource::<State<AppState>>().unwrap().current(),
        &AppState::InGame
    );
    world
        .get_mut::<CountTimer>(count_timer)
        .unwrap()
        .0
        .tick(Duration::from_secs_f32(1.0));
    update_stage.run(&mut world);

    assert_eq!(
        world.get_resource::<State<AppState>>().unwrap().current(),
        &AppState::GameOver
    );
}

#[test]
fn test_spawning_to_fixed() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(spawning_to_fixed);
    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, BLOCK_SIZE * -5.9, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Spawning);
    assert_eq!(world.query::<(&Block, &Spawning)>().iter(&world).len(), 1);
    update_stage.run(&mut world);
    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 1);
    assert_eq!(world.query::<(&Block, &Spawning)>().iter(&world).len(), 0);
}

#[test]
fn test_bottom_down() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(bottom_down);
    let bottom = world
        .spawn()
        .insert(Bottom)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, -300.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .id();
    world.insert_resource(GameSpeed {
        current: 10.0,
        ..Default::default()
    });
    let mut time = Time::default();
    time.update();
    world.insert_resource(time);
    update_stage.run(&mut world);
    assert_eq!(
        world.get::<Transform>(bottom).unwrap().translation.y,
        -350.0
    );
}

#[test]
fn test_generate_spawning_block() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(generate_spawning_block);
    world.insert_resource(BlockMaterials {
        red_material: Handle::<Image>::default(),
        green_material: Handle::<Image>::default(),
        blue_material: Handle::<Image>::default(),
        yellow_material: Handle::<Image>::default(),
        purple_material: Handle::<Image>::default(),
        indigo_material: Handle::<Image>::default(),
    });
    let mut time = Time::default();
    time.update();
    world.insert_resource(time);
    world.insert_resource(GameSpeed {
        current: 10.0,
        ..Default::default()
    });
    world.spawn().insert(Board).insert_bundle(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(BLOCK_SIZE * 6.0, BLOCK_SIZE * 12.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    for i in 0..6 {
        world
            .spawn()
            .insert(Block)
            .insert_bundle(SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(BLOCK_SIZE * i as f32, BLOCK_SIZE * -6.0, 0.0),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Spawning);
    }
    update_stage.run(&mut world);
    assert_eq!(world.query::<(&Block, &Spawning)>().iter(&world).len(), 12);
}
