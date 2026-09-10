//! `openggs-core` — the deterministic, renderer-free game core of OpenGGS.
//!
//! A Rust transcode of the simulation half of Roman Högg's OpenGGS
//! (<https://github.com/bugix/OpenGGS>), a free-software remake of "The Great
//! Giana Sisters". Like its upstream, this crate is GPL-2.0-only.
//!
//! # What "core" means here
//!
//! The C++ is a single binary in which physics, collision, rendering, audio and
//! input all read and write the same file-scope globals (`PC`, `World`, `GV`,
//! `StageC64`, `TileType`, `Key_*`). This crate takes the half that is a pure
//! state transition — where the player is, how fast, and what it collides with
//! — and makes it a value:
//!
//! ```text
//!   Sim { world, gv, pc, stage, tiles }   ->  step  ->  Sim'
//! ```
//!
//! No SDL, no globals, no I/O. Rendering, audio and input belong to a shell
//! crate that owns a [`Sim`] and drives it. The routines that would call
//! `AUDIO_Sound_Play` inline return a `bool` instead, so the shell plays the
//! sound and this crate stays dependency-free.
//!
//! # Provenance
//!
//! Every module names the C++ file and function it came from. The mapping was
//! derived from a libclang harvest of all 55 translation units
//! (`openggs-spo`), which is also what identifies the parts NOT yet ported —
//! see `docs/TRANSCODE-LEDGER.md`.

pub mod collision;
pub mod physics;
pub mod player;
pub mod stage;
pub mod world;

use player::Pc;
use stage::{Stage, TileTable};
use world::{GameVars, World};

/// One frame's worth of player input.
///
/// The C++ reads a mix of `Key_*` globals and a `Unified` struct from inside
/// the physics routines. Gathering them here is what lets a physics step be a
/// function of its arguments — and what makes the step reproducible in a test.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Input {
    pub left: bool,
    pub right: bool,
    /// Jump is held this frame (`Unified.Jump`). Releasing it mid-rise cuts
    /// the jump short.
    pub jump: bool,
    /// `Key_DOWN` — slam to terminal velocity.
    pub down: bool,
}

/// The whole simulation state.
///
/// [`Default`] is the state `PC_Define()` + `GAME_ENVIRONMENT_Define()` leave
/// behind: a player on an empty stage with no solid tiles anywhere.
#[derive(Debug, Clone, Default)]
pub struct Sim {
    pub world: World,
    pub gv: GameVars,
    pub pc: Pc,
    pub stage: Stage,
    pub tiles: TileTable,
    /// `StopRunningLeft` / `StopRunningRight`, the file-scope flags
    /// `PLAYER_Collision_Solids.cpp` declares beside the collision routines.
    /// They are cleared on every `check_tile_collision` call.
    pub(crate) stop_running_left: bool,
    pub(crate) stop_running_right: bool,
}

/// Side effects a frame produced that the shell must act on.
///
/// The C++ fires these as inline `AUDIO_Sound_Play` calls from deep inside the
/// physics. Returning them keeps this crate free of an audio dependency without
/// dropping the event.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FrameEvents {
    pub play_jump_sound: bool,
}

impl Sim {
    /// Advance one frame.
    ///
    /// The call order mirrors the per-frame block in
    /// `OpenGGS/src/LOOP_Gameloop.cpp`: input latch, run, jump, gravity,
    /// friction (which is also where the horizontal move happens), collision.
    pub fn step(&mut self, input: Input, elapsed_ms: i32) -> FrameEvents {
        self.pc.accell_left = input.left;
        self.pc.accell_right = input.right;
        self.pc.jump_on_going = input.jump;

        self.run();
        let play_jump_sound = self.jump();
        self.gravity(input.down, input.jump);
        self.friction();
        self.check_tile_collision();

        // Coyote time counts down in wall-clock milliseconds, and only while
        // airborne — landing re-arms it in `check_tile_collision`.
        if !self.pc.on_ground {
            self.pc.on_ground_delay = (self.pc.on_ground_delay - elapsed_ms).max(0);
        }

        FrameEvents { play_jump_sound }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stage::TileType;

    /// A stage with a solid floor along one tile row, and the tile table entry
    /// that makes it solid.
    fn floor_at(tile_row: usize) -> Sim {
        let mut sim = Sim::default();
        sim.tiles.set(
            1,
            TileType {
                solid: true,
                ..TileType::default()
            },
        );
        for x in 0..world::STAGE_TILES_X {
            sim.stage.set_tile(x, tile_row, 1);
        }
        sim
    }

    #[test]
    fn run_accelerates_and_clamps_to_run_strength() {
        let mut sim = Sim::default();
        sim.pc.accell_right = true;
        // Far more frames than the clamp needs, so the assertion is about the
        // ceiling and not about the ramp.
        for _ in 0..200 {
            sim.run();
        }
        assert_eq!(sim.pc.run_velocity, sim.pc.run_strength as f32);
        assert_eq!(sim.pc.direction, player::Direction::Right);
    }

    #[test]
    fn run_bonus_arms_make_acceleration_nonlinear() {
        // The falsifier for the three stacked arms: if they collapsed to a
        // single 1.5*friction impulse, every frame would add the same amount.
        let mut sim = Sim::default();
        sim.pc.accell_right = true;
        sim.run();
        let first = sim.pc.run_velocity;
        // Get above the +9 threshold, where the second arm starts contributing.
        while sim.pc.run_velocity <= 9.0 {
            sim.run();
        }
        let before = sim.pc.run_velocity;
        sim.run();
        let fast = sim.pc.run_velocity - before;
        assert_eq!(first, 1.5, "base impulse");
        assert_eq!(fast, 2.0, "base + above-9 bonus");
    }

    #[test]
    fn holding_both_directions_applies_friction_not_thrust() {
        let mut sim = Sim::default();
        sim.pc.run_velocity = 10.0;
        sim.pc.accell_left = true;
        sim.pc.accell_right = true;
        sim.run();
        assert_eq!(sim.pc.run_velocity, 10.0, "neither impulse arm fires");
        sim.friction();
        assert_eq!(sim.pc.run_velocity, 9.0, "decays as if no input");
    }

    #[test]
    fn friction_decays_to_rest_and_does_not_overshoot_into_reverse() {
        let mut sim = Sim::default();
        sim.pc.run_velocity = 3.0;
        for _ in 0..10 {
            sim.friction();
        }
        assert_eq!(sim.pc.run_velocity, 0.0);
    }

    #[test]
    fn gravity_clamps_at_terminal_velocity() {
        let mut sim = Sim::default();
        sim.pc.on_ground = false;
        for _ in 0..100 {
            sim.gravity(false, false);
        }
        assert_eq!(sim.pc.jump_velocity, -(sim.world.terminal_velocity as f32));
    }

    #[test]
    fn releasing_jump_cuts_the_rise_short() {
        let mut held = Sim::default();
        let mut released = Sim::default();
        for s in [&mut held, &mut released] {
            s.pc.jump_on_going = true;
            s.jump();
        }
        held.gravity(false, true);
        released.gravity(false, false);
        assert!(
            released.pc.jump_velocity < held.pc.jump_velocity,
            "a released jump must lose upward velocity faster: held={} released={}",
            held.pc.jump_velocity,
            released.pc.jump_velocity
        );
    }

    #[test]
    fn jump_requires_ground_or_coyote_time() {
        let mut sim = Sim::default();
        sim.pc.on_ground = false;
        sim.pc.on_ground_delay = 0;
        sim.pc.jump_on_going = true;
        assert!(!sim.jump(), "airborne with no coyote time cannot jump");

        sim.pc.on_ground_delay = 40;
        assert!(sim.jump(), "coyote time still allows the jump");
        assert_eq!(sim.pc.on_ground_delay, 0, "and consumes it");
    }

    #[test]
    fn a_falling_player_lands_on_a_solid_floor_and_stops() {
        let mut sim = floor_at(20);
        sim.pc.pos_x = 100;
        sim.pc.pos_y = 100;
        sim.pc.on_ground = false;

        for _ in 0..200 {
            sim.step(Input::default(), 16);
            if sim.pc.on_ground {
                break;
            }
        }

        assert!(sim.pc.on_ground, "never landed; y={}", sim.pc.pos_y);
        assert_eq!(
            sim.pc.jump_velocity, 0.0,
            "landing zeroes vertical velocity"
        );
        // The floor's top edge is at y = 20*16 = 320. Landing must happen at
        // the surface, not somewhere inside the tile column.
        assert!(
            (320..=320 + world::TILE_SIZE).contains(&sim.pc.pos_y),
            "landed at y={}, not on the floor surface",
            sim.pc.pos_y
        );

        // And it stays landed rather than sinking frame after frame.
        let settled = sim.pc.pos_y;
        for _ in 0..30 {
            sim.step(Input::default(), 16);
        }
        assert_eq!(sim.pc.pos_y, settled, "player sinks through the floor");
    }

    #[test]
    fn a_runner_is_stopped_flush_against_a_wall_and_does_not_tunnel_through_it() {
        let mut sim = floor_at(20);
        // A solid wall column just to the right of the spawn. Its left face is
        // at x = 9 * 16 = 144.
        for y in 0..20 {
            sim.stage.set_tile(9, y, 1);
        }
        sim.pc.pos_x = 100;
        sim.pc.pos_y = 319;
        sim.pc.on_ground = true;

        for _ in 0..60 {
            sim.step(
                Input {
                    right: true,
                    ..Input::default()
                },
                16,
            );
        }
        let settled = sim.pc.pos_x;

        // Flush, not merely stopped: the collision box's right edge must be
        // within a pixel of the wall face. Asserting only `< 144` would also
        // pass if the player never moved at all.
        let right_edge = settled + sim.pc.col_width_half();
        assert_eq!(right_edge, 143, "not flush against the wall face at 144");

        // Still pushing right for another second changes nothing.
        for _ in 0..60 {
            sim.step(
                Input {
                    right: true,
                    ..Input::default()
                },
                16,
            );
        }
        assert_eq!(sim.pc.pos_x, settled, "crept into the wall over time");

        // `hit_wall` is deliberately NOT asserted here. Contact zeroes
        // `run_velocity`, and the direction probes only fire while the velocity
        // is non-zero, so the flag alternates between frames while the player
        // leans on a wall. That is the C++ behaviour; the stable fact is the
        // position, which is what this test pins.
    }

    #[test]
    fn power_up_set_clamps_and_reports_the_sound() {
        let mut pc = Pc::default();
        assert!(!pc.set_power_up(0));
        assert_eq!(pc.mode, player::Mode::Starting);

        assert!(pc.set_power_up(1), "power-up 1 plays the sound");
        assert_eq!(pc.mode, player::Mode::Punk);
        assert_eq!(pc.morphing_counter, player::MORPHING_DURATION);

        pc.set_power_up(99);
        assert_eq!(pc.power_up, 7, "clamped to the top of the range");

        let lives = pc.lives;
        pc.set_power_up(8);
        assert_eq!(pc.lives, lives + 1, "8 is the extra-life pickup");
        assert_eq!(pc.power_up, 7, "and still clamps");
    }

    #[test]
    fn truncation_matches_the_c_cast_toward_zero() {
        // `int(v/4)` in C truncates toward zero for both signs; a rounding or
        // floor conversion here would change the movement feel asymmetrically.
        assert_eq!((7.9_f32 / 4.0) as i32, 1);
        assert_eq!((-7.9_f32 / 4.0) as i32, -1);
    }
}
