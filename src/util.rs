use bevy::prelude::*;

use crate::map::TiledMap;

pub fn map_to_world(map: &TiledMap, pos: Vec2, size: Vec2, z: f32) -> Transform {
    let map_height = map.map.height * map.map.tile_height;
    let map_width = map.map.width * map.map.tile_width;

    Transform::from_xyz(
        map_width as f32 / -2.0 + pos.x + size.x / 2.0,
        // Y axis in bevy/tiled are reversed.
        map_height as f32 / 2.0 - pos.y + size.y / 2.0,
        z,
    )
}
