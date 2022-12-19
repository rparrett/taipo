use bevy::prelude::*;

use crate::{typing_target_finished_event, TaipoState, TowerSelection, TowerSlot};

pub struct ReticlePlugin;

impl Plugin for ReticlePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(TaipoState::Playing)
                .with_system(animate_reticle)
                .with_system(move_reticle)
                .after(typing_target_finished_event),
        );
    }
}

#[derive(Component)]
pub struct Reticle;

fn move_reticle(
    mut reticle_query: Query<(&mut Transform, &mut Visibility), With<Reticle>>,
    transform_query: Query<&Transform, (With<TowerSlot>, Without<Reticle>)>,
    selection: ResMut<TowerSelection>,
) {
    if !selection.is_changed() {
        return;
    }

    for (mut reticle_transform, mut reticle_visible) in reticle_query.iter_mut() {
        if let Some(tower) = selection.selected {
            if let Ok(transform) = transform_query.get(tower) {
                reticle_transform.translation.x = transform.translation.x;
                reticle_transform.translation.y = transform.translation.y;
            }
            reticle_visible.is_visible = true;
        } else {
            reticle_visible.is_visible = false;
        }
    }
}

fn animate_reticle(mut query: Query<&mut Transform, With<Reticle>>, time: Res<Time>) {
    for mut transform in query.iter_mut() {
        let delta = time.delta_seconds();
        transform.rotate(Quat::from_rotation_z(-2.0 * delta));
    }
}
