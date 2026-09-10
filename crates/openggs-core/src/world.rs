//! World constants and the tile geometry.
//!
//! Transcoded from `GAME_ENVIRONMENT_Define()` in
//! `OpenGGS/src/GAME_ENVIRONMENT.cpp`. The C++ writes these into the file-scope
//! `World_Type World` / `GameVariables_Type GV` globals; here they are owned
//! values so a simulation step is a pure function of the state handed to it.

/// Tile edge length in pixels (`TS.Tile_Width` / `TS.Tile_Height`).
///
/// Both axes are 16 in the C++ and every collision probe divides by them, so
/// they are one constant here rather than two fields that could drift apart.
pub const TILE_SIZE: i32 = 16;

/// Stage extent in tiles, fixed by the `TileNumber[256][30]` array shape.
pub const STAGE_TILES_X: usize = 256;
pub const STAGE_TILES_Y: usize = 30;

/// The physics constants (`World_Type`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct World {
    /// Downward acceleration applied to `jump_velocity` each frame.
    pub gravity: i32,
    /// Maximum falling speed; `jump_velocity` is clamped to its negation.
    pub terminal_velocity: i32,
    /// Horizontal acceleration/decay unit.
    pub friction: f32,
    /// Terminal velocity while grinding down a wall.
    pub wall_friction: i32,
    pub tile_switch_speed: i32,
}

impl Default for World {
    /// The values `GAME_ENVIRONMENT_Define()` assigns.
    fn default() -> Self {
        Self {
            gravity: 1,
            terminal_velocity: 15,
            friction: 1.0,
            wall_friction: 7,
            tile_switch_speed: 50,
        }
    }
}

/// The subset of `GameVariables_Type` the simulation core actually reads.
///
/// Screen size, volumes and directory probes are presentation/shell concerns
/// and live in the SDL shell, not here.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GameVars {
    /// `GV.PixelPerfectRunning`. The C++ comment says there is no reason to
    /// ever switch it off, but both arms are transcoded because the off-arm
    /// changes the collision response (a tile-snap instead of a step loop).
    pub pixel_perfect_running: bool,
}

impl Default for GameVars {
    fn default() -> Self {
        Self {
            pixel_perfect_running: true,
        }
    }
}
