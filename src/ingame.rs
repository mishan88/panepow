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
            .add_system(tag_block.system().label("tag"))
            .add_system(move_block.system().after("tag"));
    }
}

const BOARD_WIDTH: usize = 6;
const BOARD_HEIGHT: usize = 13;
const BLOCK_SIZE: f32 = 50.0;

#[derive(Debug, PartialEq)]
enum BlockColor {
    Red,
    Green,
    Blue,
    Yellow,
    Purple,
    Indigo,
}

struct Block;

#[derive(Debug)]
struct Move(f32, f32);

struct Fixed;
struct Moving;
struct Matched;
struct Despawining;

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

pub struct CursorMaterials {
    cursor_material: Handle<ColorMaterial>,
}

pub struct Cursor;

struct Board {
    relative_x: f32,
    relative_y: f32,
}

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
        board_material: materials.add(Color::rgba(1.0, 1.0, 1.0, 0.0).into()),
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
                translation: Vec3::new(0.0, 0.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Board {
            relative_x: -125.0,
            relative_y: -300.0,
        });
}

fn setup_block(mut commands: Commands, block_materials: Res<BlockMaterials>, query: Query<&Board>) {
    if let Ok(board) = query.single() {
        let relative_x = board.relative_x;
        let relative_y = board.relative_y;
        commands
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
            .insert(Fixed);
        commands
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
            .insert(Fixed);
        commands
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
            .insert(BlockColor::Red)
            .insert(Fixed);
        commands
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
            .insert(BlockColor::Blue)
            .insert(Fixed);
        commands
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
            .insert(Fixed);
    }
}

pub fn setup_cursor(mut commands: Commands, materials: Res<CursorMaterials>) {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite::new(Vec2::new(BLOCK_SIZE * 2.0, BLOCK_SIZE)),
            material: materials.cursor_material.clone(),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Cursor);
}

pub fn move_cursor(
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
            println!(
                "cursor: {}, {}, {}",
                left_x, right_x, cursor_transform.translation.y
            );
            for (block_entity, block_transform) in block.iter_mut() {
                if (block_transform.translation.y - cursor_transform.translation.y).abs()
                    < f32::EPSILON
                {
                    // left -> right
                    if (block_transform.translation.x - left_x).abs() < f32::EPSILON {
                        println!("left block: {}, {}", left_x, cursor_transform.translation.y);
                        println!("left block: {:?}", block_entity.id());
                        commands
                            .entity(block_entity)
                            .remove::<Fixed>()
                            .insert(Move(right_x, cursor_transform.translation.y));
                    }
                    // right -> left
                    if (block_transform.translation.x - right_x).abs() < f32::EPSILON {
                        println!(
                            "right block: {}, {}",
                            right_x, cursor_transform.translation.y
                        );
                        println!("right block: {:?}", block_entity.id());
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
        println!("{}, {}, {:?}", entity.id(), transform.translation, target);
        commands
            .entity(entity)
            .insert(transform.ease_to(
                Transform::from_translation(Vec3::new(target.0, target.1, 0.0)),
                bevy_easings::EaseMethod::Linear,
                bevy_easings::EasingType::Once {
                    duration: std::time::Duration::from_millis(150),
                },
            ))
            .remove::<Move>()
            .insert(Fixed);
    }
}
