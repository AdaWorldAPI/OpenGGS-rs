//! The player character.
//!
//! Transcoded from `PC_Type` (`OpenGGS/src/CONTENT_Player.h`) and
//! `PC_Define()` / `PC_PowerUp_Set()` (`OpenGGS/src/PLAYER.cpp`).
//!
//! The C++ struct carries 60+ fields, including the `FrameX/Y/W/H[10][10][10]`
//! sprite atlas and the `x1..y3T` scratch probes. Those are presentation and
//! per-call temporaries respectively, so they are deliberately NOT fields here:
//! the atlas belongs to the renderer, and the probes are locals in
//! [`crate::collision`]. Keeping them out is what makes [`Pc`] a state value
//! rather than a scratchpad.

/// Facing / sprite row (`PC_RIGHT` / `PC_LEFT` / `PC_MORPH`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Right,
    Left,
}

/// Power-up strength (`PC_MODE1` / `PC_MODE2` / `PC_MODE3` / `PC_DEAD`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// `PC_MODE1` — starting strength.
    Starting,
    /// `PC_MODE2` — punk.
    Punk,
    /// `PC_MODE3` — morphing into punk.
    Morphing,
    /// `PC_DEAD`.
    Dead,
}

/// `PC_MORPHING_DURATION`, in milliseconds.
pub const MORPHING_DURATION: i32 = 250;
/// `PC_WARPING_DURATION`, in milliseconds.
pub const WARPING_DURATION: i32 = 1000;

/// The player state a physics step reads and writes.
#[derive(Debug, Clone, PartialEq)]
pub struct Pc {
    pub pos_x: i32,
    pub pos_y: i32,
    pub jump_velocity: f32,
    pub jump_strength: i32,
    pub run_velocity: f32,
    pub run_strength: i32,

    /// Collision box, and the halves/quarters the tile probes use.
    ///
    /// The C++ precomputes these into `PC.ColWidthHalf` etc. once in
    /// `PC_Define()`; they are derived here in [`Pc::col_width_half`] and
    /// friends so a change to `col_width` cannot leave a stale half behind.
    pub col_width: i32,
    pub col_height: i32,

    pub on_ground: bool,
    /// Milliseconds the player may still jump after walking off a ledge
    /// (coyote time). Counts down from [`Pc::on_ground_delay_max`].
    pub on_ground_delay: i32,
    pub on_ground_delay_max: i32,
    pub hit_wall: bool,
    pub jump_on_going: bool,
    pub wall_grinding: bool,
    pub on_platform: bool,

    pub direction: Direction,
    pub mode: Mode,
    pub morphing_counter: i32,
    pub warp_counter: i32,

    pub accell_left: bool,
    pub accell_right: bool,

    pub dead: bool,
    pub got_killed: bool,
    pub god_mode: bool,
    pub exit_reached: bool,

    pub power_up: i32,
    pub coins: i32,
    pub lives: i32,
    pub points: i32,
    pub stage: i32,

    /// `PC.MovementVertical` — the per-frame vertical step the gravity routine
    /// derives from `jump_velocity` and then walks off one pixel at a time.
    pub movement_vertical: i32,
}

impl Default for Pc {
    /// The field assignments in `PC_Define()`, minus the sprite-atlas load.
    ///
    /// `PC_Define()` also calls `PC_PowerUp_Set(0)`, which is why `mode` is
    /// `Starting` and `power_up` is 0 here.
    fn default() -> Self {
        Self {
            pos_x: 100,
            pos_y: 300,
            jump_velocity: 0.0,
            jump_strength: 24,
            run_velocity: 0.0,
            run_strength: 25,
            col_width: 28,
            col_height: 42,
            on_ground: true,
            on_ground_delay: 0,
            on_ground_delay_max: 80,
            hit_wall: true,
            jump_on_going: false,
            wall_grinding: false,
            on_platform: false,
            direction: Direction::Right,
            mode: Mode::Starting,
            morphing_counter: 0,
            warp_counter: 0,
            accell_left: false,
            accell_right: false,
            dead: false,
            got_killed: false,
            god_mode: false,
            exit_reached: false,
            power_up: 0,
            coins: 0,
            lives: 10,
            points: 0,
            stage: 1,
            movement_vertical: 0,
        }
    }
}

impl Pc {
    #[must_use]
    pub const fn col_width_half(&self) -> i32 {
        self.col_width / 2
    }

    #[must_use]
    pub const fn col_width_quart(&self) -> i32 {
        self.col_width / 4
    }

    #[must_use]
    pub const fn col_height_half(&self) -> i32 {
        self.col_height / 2
    }

    /// `PC_PowerUp_Set()`.
    ///
    /// Returns `true` when the caller should play the power-up sound, because
    /// audio is a shell concern: the C++ calls `AUDIO_Sound_Play` inline, which
    /// is exactly the coupling that would drag SDL into this crate.
    pub fn set_power_up(&mut self, mut power_up: i32) -> bool {
        if power_up == 8 {
            self.lives += 1;
        }
        power_up = power_up.clamp(0, 7);
        // Both C++ arms are `if`, not `if/else`; with the clamp above they are
        // exhaustive and mutually exclusive, so the match is equivalent.
        self.mode = if power_up < 1 {
            Mode::Starting
        } else {
            Mode::Punk
        };
        let play_sound = power_up == 1;
        if play_sound {
            self.morphing_counter = MORPHING_DURATION;
        }
        self.power_up = power_up;
        play_sound
    }
}
