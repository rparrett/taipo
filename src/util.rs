use bevy::prelude::*;

use crate::map::TiledMap;

pub fn set_visible_recursive(
    is_visible: bool,
    entity: Entity,
    visible_query: &mut Query<&mut Visible>,
    children_query: &Query<&Children>,
) {
    if let Ok(mut visible) = visible_query.get_mut(entity) {
        visible.is_visible = is_visible;
    }

    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            set_visible_recursive(is_visible, *child, visible_query, children_query);
        }
    }
}

pub fn map_to_world(map: &TiledMap, pos: Vec2, size: Vec2, z: f32) -> Transform {
    let mut transform = Transform::default();

    let map_height = map.map.height * map.map.tile_height;
    let map_width = map.map.width * map.map.tile_width;

    // Y axis in bevy/tiled are reversed.
    transform.translation.x -= map_width as f32 / 2.0 - pos.x - size.x / 2.0;
    transform.translation.y += map_height as f32 / 2.0 - pos.y - size.y / 2.0;
    transform.translation.z = z;

    transform
}
