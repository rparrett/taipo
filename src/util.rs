use bevy::prelude::*;

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

pub fn map_to_world(
    map: &bevy_tiled_prototype::Map,
    pos: Vec2,
    size: Vec2,
    z: f32,
    centered: bool,
) -> Transform {
    let mut transform = if centered {
        map.center(Transform::default())
    } else {
        Transform::default()
    };

    // Y axis in bevy/tiled are reversed.
    transform.translation.x += pos.x + size.x / 2.0;
    transform.translation.y -= pos.y - size.y / 2.0;
    transform.translation.z = z;

    transform
}
