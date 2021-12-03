use std::time::Duration;

use bevy::prelude::*;
use bevy_easings::*;

pub struct IngamePlugin;

impl Plugin for IngamePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(setup_assets.system())
            .add_startup_stage("setup_board", SystemStage::single(setup_board.system()))
            .add_startup_stage("setup_block", SystemStage::single(setup_block.system()))
            .add_startup_stage("setup_cursor", SystemStage::single(setup_cursor.system()))
            .add_system(move_cursor.system())
            .add_system(tag_block.system())
            .add_system(move_block.system())
            .add_system(match_block.system())
            .add_system(prepare_despawn_block.system())
            .add_system(despawn_block.system())
            .add_system(check_fall_block.system())
            .add_system(fall_block.system())
            .add_system(falling_to_fix.system());
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

#[derive(Debug)]
struct Move(f32, f32);

#[derive(Debug)]
struct Fixed;
struct Matched;
struct Fall;
struct Falling(Timer);
struct Despawining(Timer);

struct BlockMaterials {
    red_material: Handle<ColorMaterial>,
    green_material: Handle<ColorMaterial>,
    blue_material: Handle<ColorMaterial>,
    yellow_material: Handle<ColorMaterial>,
    purple_material: Handle<ColorMaterial>,
    indigo_material: Handle<ColorMaterial>,
}

struct BoardMaterials {
    board_material: Handle<ColorMaterial>,
}

struct CursorMaterials {
    cursor_material: Handle<ColorMaterial>,
}

#[derive(Debug)]
struct Cursor;

#[derive(Debug)]
struct Board;

fn setup_assets(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.insert_resource(BlockMaterials {
        red_material: materials.add(Color::RED.into()),
        green_material: materials.add(Color::GREEN.into()),
        blue_material: materials.add(Color::BLUE.into()),
        yellow_material: materials.add(Color::YELLOW.into()),
        purple_material: materials.add(Color::PURPLE.into()),
        indigo_material: materials.add(Color::INDIGO.into()),
    });
    commands.insert_resource(BoardMaterials {
        board_material: materials.add(Color::rgba(1.0, 1.0, 1.0, 0.1).into()),
    });
    commands.insert_resource(CursorMaterials {
        cursor_material: materials.add(Color::rgba(0.0, 0.0, 0.0, 0.7).into()),
    });
}

fn setup_board(mut commands: Commands, board_materials: Res<BoardMaterials>) {
    commands
        .spawn_bundle(SpriteBundle {
            material: board_materials.board_material.clone(),
            sprite: Sprite::new(Vec2::new(
                BOARD_WIDTH as f32 * BLOCK_SIZE,
                BOARD_HEIGHT as f32 * BLOCK_SIZE,
            )),
            transform: Transform {
                translation: Vec3::ZERO,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Board);
}

// TODO: generate from some block patterns.
fn setup_block(
    mut commands: Commands,
    block_materials: Res<BlockMaterials>,
    board: Query<(Entity, &Transform, &Sprite), With<Board>>,
) {
    for (board_entity, board_transform, sprite) in board.iter() {
        let relative_x = board_transform.translation.x - sprite.size.x / 2.0 + BLOCK_SIZE / 2.0;
        let relative_y = board_transform.translation.y - sprite.size.y / 2.0 + BLOCK_SIZE / 2.0;

        let block = commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
                material: block_materials.red_material.clone(),
                transform: Transform {
                    translation: Vec3::new(relative_x, relative_y, 0.0),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Block)
            .insert(BlockColor::Red)
            .insert(Fixed)
            .id();
        commands.entity(board_entity).push_children(&[block]);

        let block = commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
                material: block_materials.red_material.clone(),
                transform: Transform {
                    translation: Vec3::new(relative_x + BLOCK_SIZE, relative_y, 0.0),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Block)
            .insert(BlockColor::Red)
            .insert(Fixed)
            .id();
        commands.entity(board_entity).push_children(&[block]);

        let block = commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
                material: block_materials.blue_material.clone(),
                transform: Transform {
                    translation: Vec3::new(relative_x + BLOCK_SIZE * 2.0, relative_y, 0.0),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Block)
            .insert(BlockColor::Blue)
            .insert(Fixed)
            .id();
        commands.entity(board_entity).push_children(&[block]);
        let block = commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
                material: block_materials.red_material.clone(),
                transform: Transform {
                    translation: Vec3::new(relative_x + BLOCK_SIZE * 3.0, relative_y, 0.0),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Block)
            .insert(BlockColor::Red)
            .insert(Fixed)
            .id();
        commands.entity(board_entity).push_children(&[block]);

        let block = commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
                material: block_materials.red_material.clone(),
                transform: Transform {
                    translation: Vec3::new(relative_x + BLOCK_SIZE * 4.0, relative_y, 0.0),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Block)
            .insert(BlockColor::Red)
            .insert(Fixed)
            .id();
        commands.entity(board_entity).push_children(&[block]);

        let block = commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
                material: block_materials.red_material.clone(),
                transform: Transform {
                    translation: Vec3::new(
                        relative_x + BLOCK_SIZE * 2.0,
                        relative_y + BLOCK_SIZE,
                        0.0,
                    ),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Block)
            .insert(BlockColor::Red)
            .insert(Fixed)
            .id();
        commands.entity(board_entity).push_children(&[block]);
        let block = commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite::new(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
                material: block_materials.blue_material.clone(),
                transform: Transform {
                    translation: Vec3::new(
                        relative_x + BLOCK_SIZE * 2.0,
                        relative_y + BLOCK_SIZE * 2.0,
                        0.0,
                    ),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Block)
            .insert(BlockColor::Blue)
            .insert(Fixed)
            .id();
        commands.entity(board_entity).push_children(&[block]);

    }
}

fn setup_cursor(
    mut commands: Commands,
    materials: Res<CursorMaterials>,
    board: Query<Entity, With<Board>>,
) {
    for board_entity in board.iter() {
        let cursor = commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite::new(Vec2::new(BLOCK_SIZE * 2.0, BLOCK_SIZE)),
                material: materials.cursor_material.clone(),
                transform: Transform {
                    translation: Vec3::ZERO,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Cursor)
            .id();
        commands.entity(board_entity).push_children(&[cursor]);
    }
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

fn tag_block(
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
    cursor: Query<&Transform, With<Cursor>>,
    mut block: Query<(Entity, &Transform), (With<Block>, With<Fixed>)>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        if let Ok(cursor_transform) = cursor.single() {
            let x = cursor_transform.translation.x;
            let left_x = x - BLOCK_SIZE / 2.0;
            let right_x = x + BLOCK_SIZE / 2.0;
            for (block_entity, block_transform) in block.iter_mut() {
                if (block_transform.translation.y - cursor_transform.translation.y).abs()
                    < f32::EPSILON
                {
                    // left -> right
                    if (block_transform.translation.x - left_x).abs() < f32::EPSILON {
                        commands
                            .entity(block_entity)
                            .remove::<Fixed>()
                            .insert(Move(right_x, cursor_transform.translation.y));
                    }
                    // right -> left
                    if (block_transform.translation.x - right_x).abs() < f32::EPSILON {
                        commands
                            .entity(block_entity)
                            .remove::<Fixed>()
                            .insert(Move(left_x, cursor_transform.translation.y));
                    }
                }
            }
        }
    }
    if keyboard_input.just_pressed(KeyCode::A) {
        for (block_entity, transform) in block.iter() {
            println!("{}: {}", block_entity.id(), transform.translation);
        }
    }
}

fn move_block(
    mut commands: Commands,
    mut block_query: Query<(Entity, &Transform, &Move), (With<Block>, With<Move>)>,
) {
    for (entity, transform, target) in block_query.iter_mut() {
        commands
            .entity(entity)
            .insert(transform.ease_to(
                Transform::from_translation(Vec3::new(target.0, target.1, 0.0)),
                bevy_easings::EaseMethod::Linear,
                bevy_easings::EasingType::Once {
                    duration: std::time::Duration::from_millis(60),
                },
            ))
            .remove::<Move>()
            .insert(Fixed);
    }
}

// TODO: which fast?
// can not use collide
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
                < f32::EPSILON
                && (transform.translation.y - other_transform.translation.y).abs() < f32::EPSILON
                && block_color == other_block_color
            {
                row_matched_entities.push(entity);
                row_matched_entities.push(other_entity);
            }
            // right next to
            if (transform.translation.x - other_transform.translation.x + BLOCK_SIZE).abs()
                < f32::EPSILON
                && (transform.translation.y - other_transform.translation.y).abs() < f32::EPSILON
                && block_color == other_block_color
            {
                row_matched_entities.push(entity);
                row_matched_entities.push(other_entity);
            }
            // top next to
            if (transform.translation.y - other_transform.translation.y + BLOCK_SIZE).abs()
                < f32::EPSILON
                && (transform.translation.x - other_transform.translation.x).abs() < f32::EPSILON
                && block_color == other_block_color
            {
                column_matched_entities.push(entity);
                column_matched_entities.push(other_entity);
            }
            // down next to
            if (transform.translation.y - other_transform.translation.y - BLOCK_SIZE).abs()
                < f32::EPSILON
                && (transform.translation.x - other_transform.translation.x).abs() < f32::EPSILON
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

fn _match_block(
    mut commands: Commands,
    mut block: Query<
        (Entity, &Transform, &BlockColor),
        (With<Block>, With<Fixed>, With<BlockColor>),
    >,
) {
    let mut table: Vec<Vec<(Option<BlockColor>, Option<Entity>)>> =
        vec![vec![(None, None); BOARD_WIDTH]; BOARD_HEIGHT];
    let mut matched_entity: Vec<Entity> = Vec::new();
    // create match table
    for (entity, transform, block_color) in block.iter_mut() {
        let column_index = ((transform.translation.x + 125.0) / BLOCK_SIZE).floor() as usize;
        let row_index = ((transform.translation.y + 300.0) / BLOCK_SIZE).floor() as usize;
        if let Some(column_vec) = table.get_mut(row_index) {
            let _ = std::mem::replace(
                &mut column_vec[column_index],
                (Some(*block_color), Some(entity)),
            );
        }
    }
    // x-axis matches
    for row in table.iter() {
        let mut row_matched_entity: Vec<Entity> = Vec::new();
        let mut matched_color: Option<BlockColor> = None;

        for (row_block_color, row_block_entity) in row.iter() {
            match row_block_color {
                None => {
                    // end matches
                    if row_matched_entity.len() >= 3 {
                        matched_entity.append(&mut row_matched_entity);
                    } else {
                        row_matched_entity.clear();
                    }
                    matched_color = None;
                }
                Some(colored_block) => {
                    // check is same color
                    if matched_color == Some(*colored_block) {
                    } else {
                        // end matches
                        if row_matched_entity.len() >= 3 {
                            matched_entity.append(&mut row_matched_entity);
                        } else {
                            row_matched_entity.clear();
                        }
                        matched_color = Some(*colored_block);
                    }
                    if let Some(en) = row_block_entity {
                        row_matched_entity.push(*en);
                    }
                }
            }
        }
        if row_matched_entity.len() >= 3 {
            matched_entity.append(&mut row_matched_entity);
        } else {
            row_matched_entity.clear();
        }
    }

    // y-axis matches
    for column_idx in 0..BOARD_WIDTH {
        let mut column_matched_entity = Vec::new();
        let mut matched_color = None;
        for row in table.iter() {
            match row[column_idx].0 {
                None => {
                    // end matches
                    if column_matched_entity.len() >= 3 {
                        matched_entity.append(&mut column_matched_entity);
                    } else {
                        column_matched_entity.clear();
                    }
                    matched_color = None;
                }
                Some(colored_block) => {
                    if matched_color == Some(colored_block) {
                    } else {
                        // end matches
                        if column_matched_entity.len() >= 3 {
                            matched_entity.append(&mut column_matched_entity);
                        } else {
                            column_matched_entity.clear();
                        }
                        matched_color = Some(colored_block);
                    }
                    if let Some(en) = row[column_idx].1 {
                        column_matched_entity.push(en);
                    }
                }
            }
        }
    }

    // not necessary
    // matched_entity.dedup();

    // match_entry
    if !matched_entity.is_empty() {
        for en in matched_entity {
            commands.entity(en).insert(Matched).remove::<Fixed>();
        }
    }
}

fn prepare_despawn_block(
    mut commands: Commands,
    mut block: Query<Entity, (With<Block>, With<Matched>)>,
) {
    // TODO: duration should be `matched_blocks * some_duration`
    // TODO: despawning animation
    for entity in block.iter_mut() {
        commands
            .entity(entity)
            .remove::<Matched>()
            .insert(Despawining(Timer::from_seconds(1.0, false)));
    }
}

fn despawn_block(
    mut commands: Commands,
    time: Res<Time>,
    mut block: Query<(Entity, &mut Despawining), (With<Block>, With<Despawining>)>,
) {
    for (entity, mut despawning) in block.iter_mut() {
        despawning
            .0
            .tick(Duration::from_secs_f32(time.delta_seconds()));
        if despawning.0.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}

// TODO: integrate match block
// TODO: fall same time above the block
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
                    < f32::EPSILON
                    && (transform.translation.x - other_transform.translation.x).abs() < BLOCK_SIZE
                {
                    is_exist = true;
                    break;
                }
            }
            if !is_exist {
                commands.entity(entity).remove::<Fixed>().insert(Fall);
            }
        }
    }
}

// TODO: fix falling time
fn fall_block(
    mut commands: Commands,
    mut block: Query<(Entity, &Transform), (With<Block>, With<Fall>)>,
) {
    for (entity, transform) in block.iter_mut() {
        commands
            .entity(entity)
            .insert(transform.ease_to(
                Transform::from_translation(Vec3::new(
                    transform.translation.x,
                    transform.translation.y - BLOCK_SIZE,
                    transform.translation.z,
                )),
                bevy_easings::EaseMethod::Linear,
                bevy_easings::EasingType::Once {
                    duration: std::time::Duration::from_millis(150),
                },
            ))
            .remove::<Fall>()
            .insert(Falling(Timer::from_seconds(0.15, false)));
    }
}

fn falling_to_fix(
    mut commands: Commands,
    time: Res<Time>,
    mut block: Query<(Entity, &mut Falling), (With<Block>, With<Falling>)>,
) {
    for (entity, mut falling) in block.iter_mut() {
        falling
            .0
            .tick(Duration::from_secs_f32(time.delta_seconds()));
        if falling.0.just_finished() {
            commands.entity(entity).remove::<Falling>().insert(Fixed);
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
    update_stage.run(&mut world);
    assert!(world.is_resource_added::<BoardMaterials>());
    assert_eq!(world.query::<&Board>().iter(&world).len(), 1);
}

#[test]
fn test_setup_cursor() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(setup_cursor.system());

    world.insert_resource(CursorMaterials {
        cursor_material: Handle::<ColorMaterial>::default(),
    });
    world.spawn().insert(Board);
    update_stage.run(&mut world);
    assert!(world.is_resource_added::<CursorMaterials>());
    assert_eq!(world.query::<&Cursor>().iter(&world).len(), 1);
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
fn test_setup_block() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(setup_block.system());

    world.insert_resource(BlockMaterials {
        red_material: Handle::<ColorMaterial>::default(),
        green_material: Handle::<ColorMaterial>::default(),
        blue_material: Handle::<ColorMaterial>::default(),
        yellow_material: Handle::<ColorMaterial>::default(),
        purple_material: Handle::<ColorMaterial>::default(),
        indigo_material: Handle::<ColorMaterial>::default(),
    });
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
    update_stage.run(&mut world);
    assert_eq!(
        world
            .query::<(&Board, Entity, &Transform, &Sprite)>()
            .iter(&world)
            .len(),
        1
    );
    assert!(world.is_resource_added::<BlockMaterials>());
    assert!(world.query::<&Block>().iter(&world).len() > 5);
}

#[test]
fn test_tag_block() {
    let mut world = World::default();
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(tag_block.system());

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
        .insert(Move(-1.0 * BLOCK_SIZE / 2.0, 0.0));
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
        .insert(Move(BLOCK_SIZE / 2.0, 0.0));

    assert_eq!(world.query::<(&Block, &Move)>().iter(&world).len(), 2);
    update_stage.run(&mut world);
    assert_eq!(world.query::<(&Block, &Move)>().iter(&world).len(), 0);
    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 2);
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
            3 => {
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
                            0.0,
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
                            0.0,
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
                translation: Vec3::new(BLOCK_SIZE / 2.0 - BLOCK_SIZE * 2.0, 0.0, 0.0),
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
                translation: Vec3::new(BLOCK_SIZE / 2.0 - BLOCK_SIZE, 0.0, 0.0),
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
                translation: Vec3::new(BLOCK_SIZE / 2.0 + BLOCK_SIZE, 0.0, 0.0),
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
                translation: Vec3::new(BLOCK_SIZE / 2.0 + BLOCK_SIZE * 2.0, 0.0, 0.0),
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
                translation: Vec3::new(BLOCK_SIZE / 2.0 + BLOCK_SIZE * 3.0, 0.0, 0.0),
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
                translation: Vec3::new(BLOCK_SIZE / 2.0 + BLOCK_SIZE, BLOCK_SIZE, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Block)
        .insert(BlockColor::Red)
        .insert(Fixed);
    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 6);
    update_stage.run(&mut world);
    assert_eq!(world.query::<(&Block, &Fixed)>().iter(&world).len(), 6);
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
    update_stage.run(&mut world);
    assert_eq!(world.query::<(&Block, &Matched)>().iter(&world).len(), 0);
    assert_eq!(
        world.query::<(&Block, &Despawining)>().iter(&world).len(),
        1
    );
}

#[ignore = "how to update time?"]
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
        .insert(Despawining(Timer::from_seconds(0.009, false)))
        .id();

    update_stage.run(&mut world);
    assert!(world.get::<Block>(block).is_none());
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
    assert_eq!(world.query::<(&Block, &Fall)>().iter(&world).len(), 1);
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
    assert_eq!(world.query::<(&Block, &Fall)>().iter(&world).len(), 1);
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
        .insert(Move(BLOCK_SIZE / 2.0, BLOCK_SIZE * -6.0));
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
        .insert(Move(BLOCK_SIZE / 2.0, BLOCK_SIZE * -6.0));

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
