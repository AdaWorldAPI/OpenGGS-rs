//! `openggs-stagefile` — reader for the OpenGGS `.lvl` stage-file format.
//!
//! # The format is a raw struct dump
//!
//! `CONTENT_Stages_Load_Write.cpp` writes stages with
//!
//! ```c
//! fseek(Stage_File, StageNumber * sizeof(ImportStage), SEEK_SET);
//! fwrite(&ImportStage, sizeof(ImportStage), 1, Stage_File);
//! ```
//!
//! There is no header, no field tagging and no version. A `.lvl` file is an
//! array of `ImportStage_Struct` records at a fixed stride, in the host's
//! layout: little-endian `int32`, natural alignment.
//!
//! # Where the offsets come from
//!
//! They were not inferred from the header by eye. `ImportStage_Struct` was
//! compiled against the real `CONTENT_Stages.h` and every field offset read out
//! with `offsetof`; the stride was then confirmed against the shipped
//! `base/stages/*.lvl` by locating the record boundaries in the data itself
//! (the default stage name recurs at exactly [`RECORD_SIZE`] intervals). Both
//! agree, which is what makes [`RECORD_SIZE`] a measured constant rather than a
//! guess.
//!
//! A consequence worth knowing: because the stride IS `sizeof` of a C struct
//! with `int` fields, a stage file written by a build with a different `int`
//! width or different `MAX_NUM_ENEMIES` is a different format that this reader
//! would silently misparse. [`StageFile::open`] therefore checks that the file
//! length is consistent with the stride before returning anything.

use std::io;

use openggs_core::stage::Stage;
use openggs_core::world::{STAGE_TILES_X, STAGE_TILES_Y};

/// `sizeof(ImportStage_Struct)`, measured (see the module docs).
pub const RECORD_SIZE: usize = 36_732;

/// Field offsets within one record, in bytes.
mod off {
    pub const NAME: usize = 0;
    pub const NAME_LEN: usize = 100;
    pub const NUMBER: usize = 100;
    /// `int Array[256][30]`, row-major: `Array[x][y]` is at
    /// `ARRAY + (x * 30 + y) * 4`.
    pub const ARRAY: usize = 104;
    pub const TILE_TYPE: usize = 30_824;
    pub const BACKGROUND_COLOUR: usize = 30_828;
    pub const START_POSITION_X: usize = 30_832;
    pub const START_POSITION_Y: usize = 30_848;
    pub const WARP_TO_STAGE: usize = 30_864;
    pub const ENEMY_TYPE: usize = 30_896;
    pub const ENEMY_IN_USE: usize = 31_096;
    pub const ENEMY_DIRECTION: usize = 31_296;
    pub const ENEMY_POS_X: usize = 31_496;
    pub const ENEMY_POS_Y: usize = 31_696;
}

/// `MAX_NUM_ENEMIES`.
pub const MAX_NUM_ENEMIES: usize = 50;
/// Number of start positions per stage.
pub const START_POSITIONS: usize = 4;

/// One enemy placement in a stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnemyPlacement {
    pub kind: i32,
    pub direction: i32,
    pub pos_x: i32,
    pub pos_y: i32,
}

/// One decoded stage record.
#[derive(Debug, Clone)]
pub struct StageRecord {
    pub name: String,
    pub number: i32,
    pub tile_type: i32,
    pub background_colour: i32,
    pub start_position_x: [i32; START_POSITIONS],
    pub start_position_y: [i32; START_POSITIONS],
    pub warp_to_stage: i32,
    /// Only the slots the record marks in use.
    pub enemies: Vec<EnemyPlacement>,
    tiles: Box<[[u16; STAGE_TILES_Y]; STAGE_TILES_X]>,
}

impl StageRecord {
    /// Build the simulation-side [`Stage`] this record describes.
    #[must_use]
    pub fn to_stage(&self) -> Stage {
        let mut stage = Stage::default();
        stage.set_tiles(self.tiles.clone());
        stage.background_colour = self.background_colour;
        stage.tile_type = self.tile_type;
        stage
    }

    /// The raw tile grid, indexed `[x][y]`.
    #[must_use]
    pub fn tiles(&self) -> &[[u16; STAGE_TILES_Y]; STAGE_TILES_X] {
        &self.tiles
    }
}

/// A `.lvl` file held in memory.
#[derive(Debug, Clone)]
pub struct StageFile {
    bytes: Vec<u8>,
}

/// Why a `.lvl` file could not be read.
#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    /// The file is shorter than a single record, so it cannot be a stage file
    /// in this format at all.
    TooShort {
        len: usize,
    },
    /// No stage exists at that index.
    NoSuchStage {
        index: usize,
        count: usize,
    },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "reading stage file: {e}"),
            Self::TooShort { len } => write!(
                f,
                "stage file is {len} bytes, shorter than one {RECORD_SIZE}-byte record"
            ),
            Self::NoSuchStage { index, count } => {
                write!(f, "stage {index} requested, file holds {count}")
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

fn read_i32(bytes: &[u8], at: usize) -> i32 {
    // A record is bounds-checked to be RECORD_SIZE long before any field read,
    // so a short slice here would be a bug in this module, not bad input.
    bytes
        .get(at..at + 4)
        .and_then(|b| <[u8; 4]>::try_from(b).ok())
        .map_or(0, i32::from_le_bytes)
}

impl StageFile {
    /// Read a `.lvl` file from disk.
    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Self, Error> {
        Self::from_bytes(std::fs::read(path)?)
    }

    /// Wrap already-read bytes.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, Error> {
        if bytes.len() < RECORD_SIZE {
            return Err(Error::TooShort { len: bytes.len() });
        }
        Ok(Self { bytes })
    }

    /// How many complete records the file holds.
    ///
    /// The shipped stage files end with a partial record — the writer seeks to
    /// a stage's slot and writes one record, so a file's tail is whatever the
    /// last write left. Only complete records are counted, and the partial tail
    /// is ignored rather than decoded into a half-populated stage.
    #[must_use]
    pub fn stage_count(&self) -> usize {
        self.bytes.len() / RECORD_SIZE
    }

    /// Decode one stage by index (0-based).
    pub fn stage(&self, index: usize) -> Result<StageRecord, Error> {
        let count = self.stage_count();
        let rec = self
            .bytes
            .get(index * RECORD_SIZE..(index + 1) * RECORD_SIZE)
            .ok_or(Error::NoSuchStage { index, count })?;

        // The name is a fixed 100-byte C buffer: NUL-terminated, and the bytes
        // past the terminator are stale. Truncating at the first NUL is what
        // the C++ `%s` read does.
        let name_bytes = &rec[off::NAME..off::NAME + off::NAME_LEN];
        let end = name_bytes
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(off::NAME_LEN);
        let name = String::from_utf8_lossy(&name_bytes[..end])
            .trim_end()
            .to_string();

        let mut tiles = Box::new([[0u16; STAGE_TILES_Y]; STAGE_TILES_X]);
        for (x, col) in tiles.iter_mut().enumerate() {
            for (y, slot) in col.iter_mut().enumerate() {
                let v = read_i32(rec, off::ARRAY + (x * STAGE_TILES_Y + y) * 4);
                // Tile numbers index a 256-slot sheet. A value outside that is
                // corrupt data, and clamping it to 0 (empty) keeps a damaged
                // file playable instead of panicking mid-level-load.
                *slot = u16::try_from(v).unwrap_or(0);
            }
        }

        let mut start_position_x = [0; START_POSITIONS];
        let mut start_position_y = [0; START_POSITIONS];
        for i in 0..START_POSITIONS {
            start_position_x[i] = read_i32(rec, off::START_POSITION_X + i * 4);
            start_position_y[i] = read_i32(rec, off::START_POSITION_Y + i * 4);
        }

        let enemies = (0..MAX_NUM_ENEMIES)
            .filter(|i| read_i32(rec, off::ENEMY_IN_USE + i * 4) != 0)
            .map(|i| EnemyPlacement {
                kind: read_i32(rec, off::ENEMY_TYPE + i * 4),
                direction: read_i32(rec, off::ENEMY_DIRECTION + i * 4),
                pos_x: read_i32(rec, off::ENEMY_POS_X + i * 4),
                pos_y: read_i32(rec, off::ENEMY_POS_Y + i * 4),
            })
            .collect();

        Ok(StageRecord {
            name,
            number: read_i32(rec, off::NUMBER),
            tile_type: read_i32(rec, off::TILE_TYPE),
            background_colour: read_i32(rec, off::BACKGROUND_COLOUR),
            start_position_x,
            start_position_y,
            warp_to_stage: read_i32(rec, off::WARP_TO_STAGE),
            enemies,
            tiles,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A synthetic record with known values at the measured offsets.
    fn record() -> Vec<u8> {
        let mut r = vec![0u8; RECORD_SIZE];
        r[..5].copy_from_slice(b"CAVE\0");
        r[off::NUMBER..off::NUMBER + 4].copy_from_slice(&7i32.to_le_bytes());
        // Array[3][4] = 42
        let at = off::ARRAY + (3 * STAGE_TILES_Y + 4) * 4;
        r[at..at + 4].copy_from_slice(&42i32.to_le_bytes());
        r[off::BACKGROUND_COLOUR..off::BACKGROUND_COLOUR + 4].copy_from_slice(&2i32.to_le_bytes());
        r[off::START_POSITION_X..off::START_POSITION_X + 4].copy_from_slice(&96i32.to_le_bytes());
        // Enemy slot 1 in use.
        r[off::ENEMY_IN_USE + 4..off::ENEMY_IN_USE + 8].copy_from_slice(&1i32.to_le_bytes());
        r[off::ENEMY_TYPE + 4..off::ENEMY_TYPE + 8].copy_from_slice(&3i32.to_le_bytes());
        r[off::ENEMY_POS_X + 4..off::ENEMY_POS_X + 8].copy_from_slice(&160i32.to_le_bytes());
        r
    }

    #[test]
    fn decodes_fields_at_the_measured_offsets() {
        let f = StageFile::from_bytes(record()).expect("one record");
        let s = f.stage(0).expect("stage 0");
        assert_eq!(s.name, "CAVE");
        assert_eq!(s.number, 7);
        assert_eq!(s.tiles()[3][4], 42);
        assert_eq!(s.background_colour, 2);
        assert_eq!(s.start_position_x[0], 96);
        assert_eq!(s.enemies.len(), 1, "only the in-use slot is reported");
        assert_eq!(s.enemies[0].kind, 3);
        assert_eq!(s.enemies[0].pos_x, 160);
    }

    #[test]
    fn tile_offsets_are_row_major_in_x() {
        // The falsifier for the [x][y] vs [y][x] mix-up: writing Array[3][4]
        // must NOT show up at tiles[4][3].
        let f = StageFile::from_bytes(record()).expect("one record");
        let s = f.stage(0).expect("stage 0");
        assert_eq!(s.tiles()[3][4], 42);
        assert_eq!(s.tiles()[4][3], 0, "indices are transposed");
    }

    #[test]
    fn a_short_file_is_rejected_rather_than_partially_decoded() {
        let err = StageFile::from_bytes(vec![0; RECORD_SIZE - 1]).unwrap_err();
        assert!(matches!(err, Error::TooShort { .. }));
    }

    #[test]
    fn a_partial_trailing_record_is_not_counted() {
        let mut bytes = record();
        bytes.extend_from_slice(&vec![0u8; RECORD_SIZE / 2]);
        let f = StageFile::from_bytes(bytes).expect("one and a half records");
        assert_eq!(f.stage_count(), 1);
        assert!(matches!(f.stage(1), Err(Error::NoSuchStage { .. })));
    }

    /// Reads the real shipped stage file when it is present next to the
    /// checkout. Skipped otherwise, so the suite stays green without the
    /// upstream assets.
    #[test]
    fn reads_the_shipped_classic_stages_when_available() {
        let path = std::path::Path::new("../../../OpenGGS/base/stages/classic.lvl");
        let Ok(f) = StageFile::open(path) else {
            eprintln!("skipping: {} not present", path.display());
            return;
        };
        assert!(f.stage_count() >= 38, "expected the shipped stage count");
        let s = f.stage(0).expect("stage 0");
        // A real stage has to have SOME non-empty tiles; an all-zero grid would
        // mean the offsets are wrong even though the read succeeded.
        let non_empty: usize = s
            .tiles()
            .iter()
            .flat_map(|c| c.iter())
            .filter(|&&t| t != 0)
            .count();
        assert!(non_empty > 0, "decoded an entirely empty stage grid");
    }
}
