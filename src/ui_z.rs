use bevy::{prelude::*, transform::TransformSystem};

// Super hacky "ui layers" plugin
//
// Just adds the value from UiZ to translation.z of that node, allowing
// us to layer one root node and its children on top of others. You must
// manually add UiZ to every node in the hierarchy.
//
// You must pick a UiZ value that's likely to be above the UI you want
// to layer above, but below the camera.
//
// UI is drawn in a separate pass, so the z value of other 2d sprites
// does not matter.
pub struct UiZPlugin;
#[derive(Component)]
pub struct UiZ(pub f32);

impl Plugin for UiZPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            CoreStage::PostUpdate,
            update.after(TransformSystem::TransformPropagate),
        );
    }
}

fn update(mut query: Query<(&UiZ, &mut GlobalTransform), (With<Node>, Changed<Transform>)>) {
    for (uiz, mut transform) in query.iter_mut() {
        let translation = transform.translation_mut();
        translation.z += uiz.0;
    }
}
