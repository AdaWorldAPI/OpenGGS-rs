//! The tile grid and the tile-type table.
//!
//! Transcoded from `StageDef_C64` and `TileTypesDefinition`
//! (`OpenGGS/src/CONTENT_Stages.h`, `OpenGGS/src/CONTENT_Tiles_and_Sprites.h`).

use crate::world::{STAGE_TILES_X, STAGE_TILES_Y, TILE_SIZE};

/// Number of tile slots in a sheet (`NUMBER_OF_TILES`).
pub const NUMBER_OF_TILES: usize = 256;

/// The per-tile behaviour flags the C++ keeps in `TileType[NUMBER_OF_TILES]`.
///
/// The `x, y, w, h` atlas rectangle of `TileTypesDefinition` is omitted: it is
/// the sheet coordinate the renderer blits from, and nothing in the simulation
/// reads it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TileType {
    pub solid: bool,
    pub exit: bool,
    pub lethal: bool,
    pub fire: bool,
    pub coin: bool,
    pub coin_block: bool,
    pub breakable: bool,
    pub power_up_block: bool,
    pub coin_block_helmet: bool,
    pub colour_changing: bool,
    pub sub_stage_entrance: bool,
    pub drop_stone: bool,
    pub warp_stone: bool,
}

/// The tile-type lookup table, indexed by tile number.
#[derive(Debug, Clone)]
pub struct TileTable {
    types: Box<[TileType; NUMBER_OF_TILES]>,
}

impl Default for TileTable {
    fn default() -> Self {
        Self {
            types: Box::new([TileType::default(); NUMBER_OF_TILES]),
        }
    }
}

impl TileTable {
    /// The flags for `tile`, or the all-false default when the tile number is
    /// out of range.
    ///
    /// The C++ indexes `TileType[...]` with whatever the stage array holds; a
    /// corrupt stage file there is an out-of-bounds read. Treating an unknown
    /// tile as "no flags set" keeps that case a non-event instead of a panic.
    #[must_use]
    pub fn get(&self, tile: u16) -> TileType {
        self.types
            .get(usize::from(tile))
            .copied()
            .unwrap_or_default()
    }

    pub fn set(&mut self, tile: u16, ty: TileType) {
        if let Some(slot) = self.types.get_mut(usize::from(tile)) {
            *slot = ty;
        }
    }
}

/// One loaded stage (`StageDef_C64`).
#[derive(Debug, Clone)]
pub struct Stage {
    /// `StageC64.TileNumber[256][30]`, indexed `[x][y]`.
    tiles: Box<[[u16; STAGE_TILES_Y]; STAGE_TILES_X]>,
    pub stage_width: i32,
    pub stage_height: i32,
    pub background_colour: i32,
    pub tile_type: i32,
}

impl Default for Stage {
    fn default() -> Self {
        Self {
            tiles: Box::new([[0; STAGE_TILES_Y]; STAGE_TILES_X]),
            stage_width: STAGE_TILES_X as i32,
            stage_height: STAGE_TILES_Y as i32,
            background_colour: 0,
            tile_type: 0,
        }
    }
}

impl Stage {
    /// `StageC64.StageWidthPixels`.
    #[must_use]
    pub const fn stage_width_pixels(&self) -> i32 {
        self.stage_width * TILE_SIZE
    }

    /// `StageC64.StageHeightPixels`.
    #[must_use]
    pub const fn stage_height_pixels(&self) -> i32 {
        self.stage_height * TILE_SIZE
    }

    /// The tile number at a tile coordinate, or tile 0 when outside the grid.
    ///
    /// Out-of-range is reachable in normal play — the collision probes divide
    /// raw pixel positions by [`TILE_SIZE`] and a player at the stage edge
    /// probes past it — so this is the ordinary path, not an error path.
    #[must_use]
    pub fn tile_at(&self, tx: i32, ty: i32) -> u16 {
        let (Ok(tx), Ok(ty)) = (usize::try_from(tx), usize::try_from(ty)) else {
            return 0;
        };
        self.tiles
            .get(tx)
            .and_then(|col| col.get(ty))
            .copied()
            .unwrap_or(0)
    }

    /// The tile number at a pixel coordinate.
    ///
    /// C++ writes `(int)(PosX / TS.Tile_Width)`, a truncating divide on a
    /// non-negative pixel position. Rust's `/` on `i32` truncates toward zero
    /// identically, so negative positions round toward 0 the same way both
    /// sides — and then fall out of range in [`Stage::tile_at`] either way.
    #[must_use]
    pub fn tile_at_pixel(&self, px: i32, py: i32) -> u16 {
        self.tile_at(px / TILE_SIZE, py / TILE_SIZE)
    }

    pub fn set_tile(&mut self, tx: usize, ty: usize, tile: u16) {
        if let Some(slot) = self.tiles.get_mut(tx).and_then(|col| col.get_mut(ty)) {
            *slot = tile;
        }
    }

    /// Replace the whole grid, e.g. from a loaded stage file.
    pub fn set_tiles(&mut self, tiles: Box<[[u16; STAGE_TILES_Y]; STAGE_TILES_X]>) {
        self.tiles = tiles;
    }
}
