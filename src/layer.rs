// Enemies currently just "below" towers in z axis. This is okay for now
// because the current map never allows an enemy to be in front of a tower.
//
// We could likely get away with towers and enemies using LAYER + y / 100.0
// or something for their z values if it becomes necessary.

pub const BEHIND_TILES: f32 = -1.0;
// Tile Layers begin at 1.0 and correspond to their Layer ID in the Tiled map
pub const RANGE_INDICATOR: f32 = 8.0;
pub const ENEMY: f32 = 9.0;
pub const TOWER: f32 = 10.0;
pub const BULLET: f32 = 11.0;
pub const RETICLE: f32 = 20.0;
pub const HEALTHBAR_BG: f32 = 90.0;
pub const HEALTHBAR: f32 = 90.1;
#[allow(dead_code)]
pub const IN_FRONT_OF_CAMERA: f32 = 1000.1;
