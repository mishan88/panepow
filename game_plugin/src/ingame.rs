use std::{collections::VecDeque, time::Duration};

use bevy::{
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
};
use bevy_easings::*;

use rand::prelude::*;

use crate::{
    loading::{
        BlockMaterials, BoardBottomCoverMaterials, BoardMaterials, BottomMaterials, CursorMaterials,
    },
    AppState,
};

pub struct IngamePlugin;

impl Plugin for IngamePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(bevy_easings::EasingsPlugin)
            .add_system_set(
                SystemSet::on_enter(AppState::InGame)
                    .with_system(setup_camera.system())
                    .with_system(setup_board.system())
                    .with_system(setup_board_bottom_cover.system())
                    .with_system(setup_chaincounter.system()),
            )
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
                    .label("move_set")
                    .before("fall_set")
                    .with_system(move_tag_block.system())
                    .with_system(custom_ease_system::<Moving>.system())
                    .with_system(move_block.system().label("move_block"))
                    .with_system(moving_to_fixed.system().after("move_block")),
            )
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
                    .label("fall_set")
                    .after("move_set")
                    .with_system(check_fall_block.system().label("check_fall"))
                    .with_system(
                        fall_upward
                            .system()
                            .label("fall_upward")
                            .after("check_fall"),
                    )
                    .with_system(
                        floating_to_fall
                            .system()
                            .label("floating_to_fall")
                            .after("fall_upward"),
                    )
                    .with_system(
                        fall_block
                            .system()
                            .label("fall_block")
                            .after("floating_to_fall"),
                    )
                    .with_system(
                        stop_fall_block
                            .system()
                            .label("stop_fall_block")
                            .after("fall_block"),
                    )
                    .with_system(
                        fixedprepare_to_fixed
                            .system()
                            .label("fixedprepare_to_fixed")
                            .after("stop_fall_block"),
                    ),
            )
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
                    .label("spawning_set")
                    .with_system(spawning_to_fixed.system())
                    .with_system(bottom_down.system().label("bottom_down"))
                    .with_system(generate_spawning_block.system().before("bottom_down")),
            )
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
                    .after("fall_set")
                    .with_system(move_cursor.system())
                    .with_system(match_block.system().label("match_block"))
                    .with_system(
                        prepare_despawn_block
                            .system()
                            .label("prepare_despawn_block")
                            .after("match_block"),
                    )
                    .with_system(
                        despawn_block
                            .system()
                            .label("despawn_block")
                            .after("prepare_despawn_block"),
                    )
                    .with_system(
                        remove_chain
                            .system()
                            .label("remove_chain")
                            .after("despawn_block"),
                    )
                    .with_system(reset_chain_counter.system().after("despawn_block"))
                    .with_system(auto_liftup.system()),
            );
    }
}

const BOARD_WIDTH: usize = 6;
const BOARD_HEIGHT: usize = 13;
const BLOCK_SIZE: f32 = 50.0;

#[derive(Debug, PartialEq, Clone, Copy)]
enum BlockColor {
    Red,
    Green,
    Blue,
    Yellow,
    Purple,
    Indigo,
}

#[derive(Debug)]
struct Block;

struct Spawning;

#[derive(Debug)]
struct Move(f32);

#[derive(Default, Debug)]
struct Moving(f32);

impl Lerp for Moving {
    type Scalar = f32;
    fn lerp(&self, other: &Self, scalar: &Self::Scalar) -> Self {
        // std::f32::lerp is unstable
        Self(self.0 + (other.0 - self.0) * scalar)
    }
}

#[derive(Debug)]
struct Fixed;
struct Matched;
struct FallPrepare;
struct Floating(Timer);
struct Fall;
struct FixedPrepare;
struct Despawining(Timer);

struct Chain;

struct Bottom;

#[derive(Debug)]
struct Cursor;

#[derive(Debug)]
struct Board;

struct BoardBottomCover;

struct CountTimer(Timer);

struct ChainCounter(u32);

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
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
    let board_sprite = Sprite::new(Vec2::new(
        BOARD_WIDTH as f32 * BLOCK_SIZE,
        BOARD_HEIGHT as f32 * BLOCK_SIZE,
    ));
    let board_entity = commands
        .spawn_bundle(SpriteBundle {
            material: board_materials.board_material.clone(),
            sprite: board_sprite.clone(),
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

    let relative_x = board_transform.translation.x - board_sprite.size.x / 2.0 + BLOCK_SIZE / 2.0;
    let relative_y = board_transform.translation.y - board_sprite.size.y / 2.0 + BLOCK_SIZE / 2.0;
    let bottom_y = board_transform.translation.y - board_sprite.size.y / 2.0 - BLOCK_SIZE / 2.0;

    if let Some(pattern) = patterns.iter().choose(&mut rng) {
        for (row_idx, row) in pattern.iter().rev().enumerate() {
            for (column_idx, one_block) in row.iter().enumerate() {
                match one_block {
                    None => {}
                    Some(num) => {
                        let block = commands
                            .spawn_bundle(SpriteBundle {
                                sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
                                material: block_colors[*num].1.clone(),
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
                    sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
                    material: block_colors[number].1.clone(),
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
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE * BOARD_WIDTH as f32, BLOCK_SIZE)),
            material: bottom_materials.bottom_material.clone(),
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
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE * 2.0, BLOCK_SIZE)),
            material: cursor_materials.cursor_material.clone(),
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
            material: board_bottom_cover_materials
                .board_bottom_cover_material
                .clone(),
            sprite: Sprite::new(Vec2::new(BOARD_WIDTH as f32 * BLOCK_SIZE, 2.0 * BLOCK_SIZE)),
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

fn move_cursor(
    keyboard_input: Res<Input<KeyCode>>,
    mut cursor: Query<&mut Transform, With<Cursor>>,
) {
    if let Ok(mut transform) = cursor.single_mut() {
        if keyboard_input.just_pressed(KeyCode::Left) && transform.translation.x > -75.0 {
            transform.translation.x -= BLOCK_SIZE;
        }
        if keyboard_input.just_pressed(KeyCode::Right) && transform.translation.x < 75.0 {
            transform.translation.x += BLOCK_SIZE;
        }
        if keyboard_input.just_pressed(KeyCode::Up) && transform.translation.y < 300.0 {
            transform.translation.y += BLOCK_SIZE;
        }
        if keyboard_input.just_pressed(KeyCode::Down) && transform.translation.y > -300.0 {
            transform.translation.y -= BLOCK_SIZE;
        }
    }
}

// TODO: if there is no fixed block -> check block and cancel tag.
fn move_tag_block(
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
    cursor: Query<&Transform, With<Cursor>>,
    mut block: Query<(Entity, &Transform, Option<&Fixed>), With<Block>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        if let Ok(cursor_transform) = cursor.single() {
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
                else if block_transform.translation.y - cursor_transform.translation.y
                    < BLOCK_SIZE
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
    if keyboard_input.just_pressed(KeyCode::A) {
        println!("-------------------");
        for (block_entity, transform, fixed) in block.iter() {
            println!(
                "{}: {}: {:?}",
                block_entity.id(),
                transform.translation,
                fixed
            );
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
        if let Ok(mut cc) = chain_counter.single_mut() {
            cc.0 += 1;
            dbg!(cc.0);
        }
    }

    let combo = match_block.iter().count();
    for (entity, _chain) in match_block.iter() {
        commands
            .entity(entity)
            .remove::<Matched>()
            .insert(Despawining(Timer::from_seconds(combo as f32 * 0.3, false)));
    }
}

fn remove_chain(
    mut commands: Commands,
    chain_block: Query<(Entity, Option<&Chain>), (With<Block>, With<Fixed>)>,
) {
    for (entity, _ch) in chain_block.iter().filter(|(_en, ch)| ch.is_some()) {
        commands.entity(entity).remove::<Chain>();
    }
}

fn reset_chain_counter(
    chain_block: Query<&Chain, (With<Block>, With<Chain>)>,
    mut chain_counter: Query<&mut ChainCounter>,
) {
    if chain_block.iter().next().is_none() {
        if let Ok(mut cc) = chain_counter.single_mut() {
            cc.0 = 1;
        }
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
                    commands.entity(*en).insert(Chain);
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
    mut fall_block: Query<(Entity, &mut Transform, &Sprite), (With<Block>, With<Fall>)>,
    other_block: Query<(&Transform, &Sprite), (With<Block>, Without<Fall>)>,
) {
    for (fall_block_entity, mut fall_block_transform, fall_block_sprite) in fall_block.iter_mut() {
        for (other_block_transform, other_block_sprite) in other_block.iter() {
            if let Some(Collision::Top) = collide(
                fall_block_transform.translation,
                fall_block_sprite.size,
                other_block_transform.translation,
                other_block_sprite.size,
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

fn auto_liftup(
    time: Res<Time>,
    mut state: ResMut<State<AppState>>,
    mut count_timer: Query<&mut CountTimer>,
    mut query_set: QuerySet<(
        Query<
            Entity,
            (
                Without<Fixed>,
                Without<Spawning>,
                Without<Moving>,
                Without<Move>,
                With<Block>,
            ),
        >,
        Query<&Transform, (With<Fixed>, With<Block>)>,
        Query<&mut Transform, Or<(With<Cursor>, With<Block>, With<Bottom>)>>,
    )>,
) {
    if let Ok(mut count_timer) = count_timer.single_mut() {
        count_timer
            .0
            .tick(Duration::from_secs_f32(time.delta_seconds()));
        let max_bl = query_set
            .q1()
            .iter()
            .max_by(|a_tr, b_tr| a_tr.translation.y.partial_cmp(&b_tr.translation.y).unwrap());
        if let Some(max_tr) = max_bl {
            if count_timer.0.finished() {
                // lift up
                if max_tr.translation.y > BLOCK_SIZE * 5.0 {
                    state.set(AppState::GameOver).unwrap();
                }
                if max_tr.translation.y < BLOCK_SIZE * 5.0 && query_set.q0().iter().next().is_none()
                {
                    for mut transform in query_set.q2_mut().iter_mut() {
                        transform.translation.y += time.delta_seconds() * 10.0;
                    }
                }
            }
        }
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

fn bottom_down(mut bottom: Query<&mut Transform, With<Bottom>>) {
    for mut transform in bottom.iter_mut() {
        if transform.translation.y >= BLOCK_SIZE * -6.0 {
            transform.translation.y = BLOCK_SIZE * -7.0;
        }
    }
}

fn generate_spawning_block(
    mut commands: Commands,
    block_materials: Res<BlockMaterials>,
    board: Query<(Entity, &Transform, &Sprite), With<Board>>,
    bottom: Query<&Transform, With<Bottom>>,
) {
    for (board_entity, board_transform, sprite) in board.iter() {
        for transform in bottom.iter() {
            if transform.translation.y >= BLOCK_SIZE * -6.0 {
                let relative_x =
                    board_transform.translation.x - sprite.size.x / 2.0 + BLOCK_SIZE / 2.0;
                let bottom_y =
                    board_transform.translation.y - sprite.size.y / 2.0 - BLOCK_SIZE / 2.0;
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
                            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
                            material: block_colors[number].1.clone(),
                            transform: Transform {
                                translation: Vec3::new(
                                    relative_x + BLOCK_SIZE * column_idx as f32,
                                    bottom_y - BLOCK_SIZE as f32,
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
    update_stage.add_system(setup_board.system());

    world.insert_resource(BoardMaterials {
        board_material: Handle::<ColorMaterial>::default(),
    });
    world.insert_resource(BlockMaterials {
        red_material: Handle::<ColorMaterial>::default(),
        green_material: Handle::<ColorMaterial>::default(),
        blue_material: Handle::<ColorMaterial>::default(),
        yellow_material: Handle::<ColorMaterial>::default(),
        purple_material: Handle::<ColorMaterial>::default(),
        indigo_material: Handle::<ColorMaterial>::default(),
    });
    world.insert_resource(BottomMaterials {
        bottom_material: Handle::<ColorMaterial>::default(),
    });
    world.insert_resource(CursorMaterials {
        cursor_material: Handle::<ColorMaterial>::default(),
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
    update_stage.add_system(move_cursor.system());
    world.spawn().insert(Board);
    world.spawn().insert(Cursor).insert_bundle(SpriteBundle {
        sprite: Sprite::new(Vec2::new(BLOCK_SIZE * 2.0, BLOCK_SIZE)),
        transform: Transform {
            translation: Vec3::ZERO,
            ..Default::default()
        },
        ..Default::default()
    });

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
    let mut input = Input::<KeyCode>::default();
    input.press(KeyCode::Left);
    world.insert_resource(input);

    update_stage.run(&mut world);
    world.get_resource_mut::<Input<KeyCode>>().unwrap();
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
    let mut input = Input::<KeyCode>::default();
    input.press(KeyCode::Left);
    world.insert_resource(input);
    update_stage.run(&mut world);
    world.get_resource_mut::<Input<KeyCode>>().unwrap();
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
    let mut input = Input::<KeyCode>::default();
    input.press(KeyCode::Left);
    world.insert_resource(input);
    update_stage.run(&mut world);
    world.get_resource_mut::<Input<KeyCode>>().unwrap();
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
    update_stage.add_system(move_cursor.system());
    world.spawn().insert(Board);
    world.spawn().insert(Cursor).insert_bundle(SpriteBundle {
        sprite: Sprite::new(Vec2::new(BLOCK_SIZE * 2.0, BLOCK_SIZE)),
        transform: Transform {
            translation: Vec3::ZERO,
            ..Default::default()
        },
        ..Default::default()
    });

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
    let mut input = Input::<KeyCode>::default();
    input.press(KeyCode::Right);
    world.insert_resource(input);

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
    let mut input = Input::<KeyCode>::default();
    input.press(KeyCode::Right);
    world.insert_resource(input);

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
    let mut input = Input::<KeyCode>::default();
    input.press(KeyCode::Right);
    world.insert_resource(input);

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
    update_stage.add_system(move_cursor.system());

    world.spawn().insert(Board);
    world.spawn().insert(Cursor).insert_bundle(SpriteBundle {
        sprite: Sprite::new(Vec2::new(BLOCK_SIZE * 2.0, BLOCK_SIZE)),
        transform: Transform {
            translation: Vec3::ZERO,
            ..Default::default()
        },
        ..Default::default()
    });

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
    let mut input = Input::<KeyCode>::default();
    input.press(KeyCode::Down);
    world.insert_resource(input);

    update_stage.run(&mut world);
    world.get_resource_mut::<Input<KeyCode>>().unwrap();
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
        let mut input = Input::<KeyCode>::default();
        input.press(KeyCode::Down);
        world.insert_resource(input);
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
        Vec3::new(0.0, -6.0 * BLOCK_SIZE, 0.0)
    );
}

#[test]
fn test_up_move_cursor() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(move_cursor.system());

    world.spawn().insert(Board);
    world.spawn().insert(Cursor).insert_bundle(SpriteBundle {
        sprite: Sprite::new(Vec2::new(BLOCK_SIZE * 2.0, BLOCK_SIZE)),
        transform: Transform {
            translation: Vec3::ZERO,
            ..Default::default()
        },
        ..Default::default()
    });

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
    let mut input = Input::<KeyCode>::default();
    input.press(KeyCode::Up);
    world.insert_resource(input);

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
        let mut input = Input::<KeyCode>::default();
        input.press(KeyCode::Up);
        world.insert_resource(input);
        update_stage.run(&mut world);
    }

    world.get_resource_mut::<Input<KeyCode>>().unwrap();
    assert_eq!(
        world
            .query::<(&Cursor, &Transform)>()
            .iter(&world)
            .next()
            .unwrap()
            .1
            .translation,
        Vec3::new(0.0, 6.0 * BLOCK_SIZE, 0.0)
    );
}

#[test]
fn test_move_tag_block_both_fix() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(move_tag_block.system());

    world.spawn().insert(Board).insert_bundle(SpriteBundle {
        sprite: Sprite::new(Vec2::new(
            BOARD_WIDTH as f32 * BLOCK_SIZE,
            BOARD_HEIGHT as f32 * BLOCK_SIZE,
        )),
        transform: Transform {
            translation: Vec3::ZERO,
            ..Default::default()
        },
        ..Default::default()
    });
    world.spawn().insert(Cursor).insert_bundle(SpriteBundle {
        sprite: Sprite::new(Vec2::new(BLOCK_SIZE * 2.0, BLOCK_SIZE)),
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
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
            transform: Transform {
                translation: Vec3::new(-1.0 * BLOCK_SIZE / 2.0, 0.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(BlockColor::Blue)
        .insert(Fixed);

    let mut input = Input::<KeyCode>::default();
    input.press(KeyCode::Space);
    world.insert_resource(input);

    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 2);

    update_stage.run(&mut world);
    world.get_resource_mut::<Input<KeyCode>>().unwrap();
    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 0);
    assert_eq!(world.query::<(&Block, &Move)>().iter(&world).len(), 2);
}

#[test]
fn test_move_tag_block_left_one_fix() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(move_tag_block.system());

    world.spawn().insert(Board).insert_bundle(SpriteBundle {
        sprite: Sprite::new(Vec2::new(
            BOARD_WIDTH as f32 * BLOCK_SIZE,
            BOARD_HEIGHT as f32 * BLOCK_SIZE,
        )),
        transform: Transform {
            translation: Vec3::ZERO,
            ..Default::default()
        },
        ..Default::default()
    });
    world.spawn().insert(Cursor).insert_bundle(SpriteBundle {
        sprite: Sprite::new(Vec2::new(BLOCK_SIZE * 2.0, BLOCK_SIZE)),
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
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0, 0.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(BlockColor::Red)
        .insert(Fixed);

    let mut input = Input::<KeyCode>::default();
    input.press(KeyCode::Space);
    world.insert_resource(input);

    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 1);

    update_stage.run(&mut world);
    world.get_resource_mut::<Input<KeyCode>>().unwrap();
    assert_eq!(world.query::<(&Block, &Move)>().iter(&world).len(), 1);
}

#[test]
fn test_move_tag_block_right_one_fix() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(move_tag_block.system());

    world.spawn().insert(Board).insert_bundle(SpriteBundle {
        sprite: Sprite::new(Vec2::new(
            BOARD_WIDTH as f32 * BLOCK_SIZE,
            BOARD_HEIGHT as f32 * BLOCK_SIZE,
        )),
        transform: Transform {
            translation: Vec3::ZERO,
            ..Default::default()
        },
        ..Default::default()
    });
    world.spawn().insert(Cursor).insert_bundle(SpriteBundle {
        sprite: Sprite::new(Vec2::new(BLOCK_SIZE * 2.0, BLOCK_SIZE)),
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
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
            transform: Transform {
                translation: Vec3::new(-1.0 * BLOCK_SIZE / 2.0, 0.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(BlockColor::Red)
        .insert(Fixed);

    let mut input = Input::<KeyCode>::default();
    input.press(KeyCode::Space);
    world.insert_resource(input);

    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 1);

    update_stage.run(&mut world);
    world.get_resource_mut::<Input<KeyCode>>().unwrap();
    assert_eq!(world.query::<(&Block, &Move)>().iter(&world).len(), 1);
}

#[test]
fn test_move_tag_block_there_is_collide() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(move_tag_block.system());

    world.spawn().insert(Board).insert_bundle(SpriteBundle {
        sprite: Sprite::new(Vec2::new(
            BOARD_WIDTH as f32 * BLOCK_SIZE,
            BOARD_HEIGHT as f32 * BLOCK_SIZE,
        )),
        transform: Transform {
            translation: Vec3::ZERO,
            ..Default::default()
        },
        ..Default::default()
    });
    world.spawn().insert(Cursor).insert_bundle(SpriteBundle {
        sprite: Sprite::new(Vec2::new(BLOCK_SIZE * 2.0, BLOCK_SIZE)),
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
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
            transform: Transform {
                translation: Vec3::new(-1.0 * BLOCK_SIZE / 2.0, 0.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(BlockColor::Red)
        .insert(Fixed);

    let mut input = Input::<KeyCode>::default();
    input.press(KeyCode::Space);
    world.insert_resource(input);

    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 1);

    update_stage.run(&mut world);
    world.get_resource_mut::<Input<KeyCode>>().unwrap();
    assert_eq!(world.query::<(&Block, &Move)>().iter(&world).len(), 0);
}

#[test]
fn test_move_tag_block_not_fixed_block() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(move_tag_block.system());

    world.spawn().insert(Board).insert_bundle(SpriteBundle {
        sprite: Sprite::new(Vec2::new(
            BOARD_WIDTH as f32 * BLOCK_SIZE,
            BOARD_HEIGHT as f32 * BLOCK_SIZE,
        )),
        transform: Transform {
            translation: Vec3::ZERO,
            ..Default::default()
        },
        ..Default::default()
    });
    world.spawn().insert(Cursor).insert_bundle(SpriteBundle {
        sprite: Sprite::new(Vec2::new(BLOCK_SIZE * 2.0, BLOCK_SIZE)),
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
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
            transform: Transform {
                translation: Vec3::new(-1.0 * BLOCK_SIZE / 2.0, 0.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(BlockColor::Blue);

    let mut input = Input::<KeyCode>::default();
    input.press(KeyCode::Space);
    world.insert_resource(input);

    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 1);

    update_stage.run(&mut world);
    world.get_resource_mut::<Input<KeyCode>>().unwrap();
    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 1);
    assert_eq!(world.query::<(&Block, &Move)>().iter(&world).len(), 0);
}

#[test]
fn test_move_block() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(move_block.system());

    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
    update_stage.add_system(match_block.system());

    for i in 0..3 {
        world
            .spawn()
            .insert(Block)
            .insert_bundle(SpriteBundle {
                sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
    update_stage.add_system(match_block.system());

    for i in 0..4 {
        world
            .spawn()
            .insert(Block)
            .insert_bundle(SpriteBundle {
                sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
    update_stage.add_system(match_block.system());

    for i in 0..5 {
        match i {
            0 | 1 | 2 | 4 => {
                world
                    .spawn()
                    .insert(Block)
                    .insert_bundle(SpriteBundle {
                        sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
                        sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
    update_stage.add_system(match_block.system());

    for i in 0..5 {
        world
            .spawn()
            .insert(Block)
            .insert_bundle(SpriteBundle {
                sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
    update_stage.add_system(match_block.system());

    for i in 0..6 {
        world
            .spawn()
            .insert(Block)
            .insert_bundle(SpriteBundle {
                sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
    update_stage.add_system(match_block.system());

    for i in 0..6 {
        if i < 3 {
            world
                .spawn()
                .insert(Block)
                .insert_bundle(SpriteBundle {
                    sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
                    sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
    update_stage.add_system(match_block.system());

    world
        .spawn()
        .insert_bundle(SpriteBundle {
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
    update_stage.add_system(match_block.system());

    for i in 0..3 {
        world
            .spawn()
            .insert(Block)
            .insert_bundle(SpriteBundle {
                sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
    update_stage.add_system(match_block.system());

    // row
    for i in 0..3 {
        world
            .spawn()
            .insert(Block)
            .insert_bundle(SpriteBundle {
                sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
    update_stage.add_system(prepare_despawn_block.system());

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
    update_stage.add_system(prepare_despawn_block.system());

    world.spawn().insert(Block).insert(Matched).insert(Chain);
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
    update_stage.add_system(remove_chain.system());
    world.spawn().insert(Block).insert(Fixed).insert(Chain);
    assert_eq!(world.query::<(&Block, &Chain)>().iter(&world).len(), 1);
    update_stage.run(&mut world);
    assert_eq!(world.query::<(&Block, &Chain)>().iter(&world).len(), 0);
}

#[test]
fn test_remove_chain_not_fixed() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(remove_chain.system());
    world.spawn().insert(Block).insert(Matched).insert(Chain);
    world.spawn().insert(Block).insert(Despawining).insert(Chain);

    assert_eq!(world.query::<(&Block, &Chain)>().iter(&world).len(), 2);
    update_stage.run(&mut world);
    assert_eq!(world.query::<(&Block, &Chain)>().iter(&world).len(), 2);
}

#[test]
fn test_reset_chain_counter() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(reset_chain_counter.system());
    let chain_counter = world.spawn().insert(ChainCounter(2)).id();
    update_stage.run(&mut world);
    assert_eq!(world.get::<ChainCounter>(chain_counter).unwrap().0, 1);
}

#[test]
fn test_reset_chain_counter_not_reset() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(reset_chain_counter.system());
    let chain_counter = world.spawn().insert(ChainCounter(2)).id();
    world.spawn().insert(Block).insert(Chain);
    update_stage.run(&mut world);
    assert_eq!(world.get::<ChainCounter>(chain_counter).unwrap().0, 2);
}

#[test]
fn test_despawn_block() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(despawn_block.system());
    let time = Time::default();
    world.insert_resource(time);

    let block = world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
    update_stage.add_system(despawn_block.system());
    let time = Time::default();
    world.insert_resource(time);

    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
    update_stage.add_system(check_fall_block.system());
    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
    update_stage.add_system(check_fall_block.system());
    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
    update_stage.add_system(check_fall_block.system());
    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
    update_stage.add_system(check_fall_block.system());
    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
    update_stage.add_system(check_fall_block.system());
    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
    update_stage.add_system(check_fall_block.system());
    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
    update_stage.add_system(fall_upward.system());

    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
    update_stage.add_system(fall_upward.system());

    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
    update_stage.add_system(floating_to_fall.system());

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
    update_stage.add_system(stop_fall_block.system());
    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
    update_stage.add_system(fixedprepare_to_fixed.system());
    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
    update_stage.add_system(auto_liftup.system());
    let app_state = State::new(AppState::InGame);
    world.insert_resource(app_state);
    let mut time = Time::default();
    time.update();
    world.insert_resource(time);
    world
        .spawn()
        .insert(CountTimer(Timer::from_seconds(0.0, false)));

    let block = world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
    update_stage.add_system(auto_liftup.system());
    let app_state = State::new(AppState::InGame);
    world.insert_resource(app_state);
    let mut time = Time::default();
    time.update();
    world.insert_resource(time);
    world
        .spawn()
        .insert(CountTimer(Timer::from_seconds(1.0, false)));

    let block = world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
    update_stage.add_system(auto_liftup.system());
    let app_state = State::new(AppState::InGame);
    world.insert_resource(app_state);
    let mut time = Time::default();
    time.update();
    world.insert_resource(time);
    world
        .spawn()
        .insert(CountTimer(Timer::from_seconds(0.0, false)));

    let block = world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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

#[ignore = "how to change state?"]
#[test]
fn test_auto_liftup_gameover() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(auto_liftup.system());
    let app_state = State::new(AppState::InGame);
    world.insert_resource(app_state);
    let mut time = Time::default();
    time.update();
    world.insert_resource(time);
    world
        .spawn()
        .insert(CountTimer(Timer::from_seconds(0.0, false)));

    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0, BLOCK_SIZE * 5.0 + 0.1, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Fixed);
    assert_eq!(
        world.get_resource::<State<AppState>>().unwrap().current(),
        &AppState::InGame
    );
    world.get_resource_mut::<Time>().unwrap().update();
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
    update_stage.add_system(spawning_to_fixed.system());
    world
        .spawn()
        .insert(Block)
        .insert_bundle(SpriteBundle {
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
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
    update_stage.add_system(bottom_down.system());
    let bottom = world
        .spawn()
        .insert(Bottom)
        .insert_bundle(SpriteBundle {
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE * 6.0, BLOCK_SIZE)),
            transform: Transform {
                translation: Vec3::new(0.0, -300.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .id();
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
    update_stage.add_system(generate_spawning_block.system());
    world.insert_resource(BlockMaterials {
        red_material: Handle::<ColorMaterial>::default(),
        green_material: Handle::<ColorMaterial>::default(),
        blue_material: Handle::<ColorMaterial>::default(),
        yellow_material: Handle::<ColorMaterial>::default(),
        purple_material: Handle::<ColorMaterial>::default(),
        indigo_material: Handle::<ColorMaterial>::default(),
    });
    world.spawn().insert(Board).insert_bundle(SpriteBundle {
        ..Default::default()
    });
    world.spawn().insert(Bottom).insert_bundle(SpriteBundle {
        sprite: Sprite::new(Vec2::new(BLOCK_SIZE * 6.0, BLOCK_SIZE)),
        transform: Transform {
            translation: Vec3::new(0.0, -300.0, 0.0),
            ..Default::default()
        },
        ..Default::default()
    });
    update_stage.run(&mut world);
    assert_eq!(world.query::<(&Block, &Spawning)>().iter(&world).len(), 6);
}
