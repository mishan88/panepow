use bevy::prelude::*;

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

enum BlockStatus {
    Fixed,
    Matched,
    Despawning,
}

struct Block {
    color: BlockColor,
    status: BlockStatus,
}

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
        board_material: materials.add(Color::WHITE.into()),
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
            .insert(Block {
                color: BlockColor::Red,
                status: BlockStatus::Fixed,
            });
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
            .insert(Block {
                color: BlockColor::Red,
                status: BlockStatus::Fixed,
            });
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
            .insert(Block {
                color: BlockColor::Blue,
                status: BlockStatus::Fixed,
            });
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
            .insert(Block {
                color: BlockColor::Red,
                status: BlockStatus::Fixed,
            });
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
            .insert(Block {
                color: BlockColor::Red,
                status: BlockStatus::Fixed,
            });
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
    mut cursor: Query<(&Cursor, &mut Transform)>,
) {
    if let Ok((_, mut transform)) = cursor.single_mut() {
        if keyboard_input.just_pressed(KeyCode::Left) {
            if transform.translation.x > -75.0 {
                transform.translation.x -= BLOCK_SIZE;
            }
        }
        if keyboard_input.just_pressed(KeyCode::Right) {
            if transform.translation.x < 75.0 {
                transform.translation.x += BLOCK_SIZE;
            }
        }
        if keyboard_input.just_pressed(KeyCode::Up) {
            if transform.translation.y < 300.0 {
                transform.translation.y += BLOCK_SIZE;
            }
        }
        if keyboard_input.just_pressed(KeyCode::Down) {
            if transform.translation.y > -300.0 {
                transform.translation.y -= BLOCK_SIZE;
            }
        }
    }
}

fn swap_block(
    keyboard_input: Res<Input<KeyCode>>,
    cursor: Query<&Cursor>,
    mut block: Query<&mut Block>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {}
}

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: String::from("PanelPow"),
            width: 1920.0,
            height: 1080.0,
            ..Default::default()
        })
        .add_startup_system(setup_assets.system())
        .add_startup_stage("setup_board", SystemStage::single(setup_board.system()))
        .add_startup_stage("setup_block", SystemStage::single(setup_block.system()))
        .add_startup_stage("setup_cursor", SystemStage::single(setup_cursor.system()))
        .add_system(move_cursor.system())
        .add_plugins(DefaultPlugins)
        .run();
}
