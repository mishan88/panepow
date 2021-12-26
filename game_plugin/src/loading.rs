use bevy::prelude::*;
use bevy_asset_loader::{AssetCollection, AssetLoader};

use crate::AppState;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        AssetLoader::new(AppState::Loading, AppState::Menu)
            .with_collection::<BlockMaterials>()
            .with_collection::<CursorMaterials>()
            .with_collection::<BoardBottomCoverMaterials>()
            .with_collection::<BoardMaterials>()
            .with_collection::<BottomMaterials>()
            .with_collection::<FontAssets>()
            .build(app);
    }
}

#[derive(AssetCollection)]
pub struct FontAssets {
    #[asset(path = "fonts/IBMPlexSansJP-Regular.ttf")]
    pub font: Handle<Font>,
}

#[derive(AssetCollection)]
pub struct BlockMaterials {
    #[asset(color_material)]
    #[asset(path = "images/red_block.png")]
    pub red_material: Handle<ColorMaterial>,
    #[asset(color_material)]
    #[asset(path = "images/green_block.png")]
    pub green_material: Handle<ColorMaterial>,
    #[asset(color_material)]
    #[asset(path = "images/blue_block.png")]
    pub blue_material: Handle<ColorMaterial>,
    #[asset(color_material)]
    #[asset(path = "images/yellow_block.png")]
    pub yellow_material: Handle<ColorMaterial>,
    #[asset(color_material)]
    #[asset(path = "images/purple_block.png")]
    pub purple_material: Handle<ColorMaterial>,
    #[asset(color_material)]
    #[asset(path = "images/indigo_block.png")]
    pub indigo_material: Handle<ColorMaterial>,
}

#[derive(AssetCollection)]
pub struct BoardMaterials {
    #[asset(color_material)]
    #[asset(path = "images/transparent.png")]
    pub board_material: Handle<ColorMaterial>,
}

#[derive(AssetCollection)]
pub struct BoardBottomCoverMaterials {
    #[asset(color_material)]
    #[asset(path = "images/bottom_cover.png")]
    pub board_bottom_cover_material: Handle<ColorMaterial>,
}

#[derive(AssetCollection)]
pub struct CursorMaterials {
    #[asset(color_material)]
    #[asset(path = "images/cursor.png")]
    pub cursor_material: Handle<ColorMaterial>,
}

#[derive(AssetCollection)]
pub struct BottomMaterials {
    #[asset(color_material)]
    #[asset(path = "images/bottom.png")]
    pub bottom_material: Handle<ColorMaterial>,
}
