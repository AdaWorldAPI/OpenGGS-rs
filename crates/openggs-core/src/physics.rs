//! Horizontal and vertical player physics.
//!
//! Transcoded from `PC_Run()` / `PC_Friction()`
//! (`OpenGGS/src/PLAYER_Run.cpp`) and `PC_Jump()` / `PC_Gravity()`
//! (`OpenGGS/src/PLAYER_Jump.cpp`).
//!
//! # Truncation
//!
//! The C++ derives its per-frame pixel steps with `int(velocity / N)` on a
//! `float`. That is a C cast: truncation toward zero, both signs. Rust's
//! `f32 as i32` truncates toward zero as well, so `(v / 4.0) as i32` is the
//! same value for every input the game can produce. This is the one place where
//! a "tidier" rounding choice would silently change the feel of the movement.

use crate::Sim;

impl Sim {
    /// `PC_Run()` — accelerate from input, then clamp to `run_strength`.
    ///
    /// The three acceleration arms stack: the base impulse always applies, a
    /// bonus applies once already moving fast in that direction, and a second
    /// bonus applies while still moving the OTHER way (the "allow faster
    /// direction changes" arm). Holding both directions cancels the direction
    /// writes but not the impulses — that is the C++ behaviour, preserved.
    pub fn run(&mut self) {
        let f = self.world.friction;
        if self.pc.accell_left {
            self.pc.direction = crate::player::Direction::Left;
        }
        if self.pc.accell_right {
            self.pc.direction = crate::player::Direction::Right;
        }

        let right_only = self.pc.accell_right && !self.pc.accell_left;
        let left_only = self.pc.accell_left && !self.pc.accell_right;

        if right_only {
            self.pc.run_velocity += 1.5 * f;
            if self.pc.run_velocity > 9.0 {
                self.pc.run_velocity += 0.5 * f;
            }
            if self.pc.run_velocity < 0.0 {
                self.pc.run_velocity += 0.5 * f;
            }
        }
        if left_only {
            self.pc.run_velocity -= 1.5 * f;
            if self.pc.run_velocity < -9.0 {
                self.pc.run_velocity -= 0.5 * f;
            }
            if self.pc.run_velocity > 0.0 {
                self.pc.run_velocity -= 0.5 * f;
            }
        }

        let max = self.pc.run_strength as f32;
        self.pc.run_velocity = self.pc.run_velocity.clamp(-max, max);
    }

    /// `PC_Friction()` — decay, move, and clamp to the stage bounds.
    ///
    /// With pixel-perfect running the horizontal move is a one-pixel-at-a-time
    /// loop that re-runs collision each step, so a wall is detected at the
    /// pixel it is hit rather than after a multi-pixel jump. The loop still
    /// counts down its full step budget once `hit_wall` is set; it just stops
    /// moving. Preserved, because the budget is not re-derived afterwards.
    pub fn friction(&mut self) {
        let f = self.world.friction;
        let no_input = (!self.pc.accell_right && !self.pc.accell_left)
            || (self.pc.accell_right && self.pc.accell_left);
        if no_input {
            if self.pc.run_velocity > 0.0 {
                self.pc.run_velocity -= f;
            }
            if self.pc.run_velocity < 0.0 {
                self.pc.run_velocity += f;
            }
        }

        if self.gv.pixel_perfect_running {
            let mut steps = (self.pc.run_velocity / 4.0) as i32;
            if self.pc.run_velocity > 0.0 {
                while steps != 0 {
                    steps -= 1;
                    if !self.pc.hit_wall {
                        self.check_tile_collision();
                        self.pc.pos_x += 1;
                    }
                }
            } else if self.pc.run_velocity < 0.0 {
                while steps != 0 {
                    steps += 1;
                    if !self.pc.hit_wall {
                        self.check_tile_collision();
                        self.pc.pos_x -= 1;
                    }
                }
            }
        } else {
            self.pc.pos_x += (self.pc.run_velocity / 4.0) as i32;
        }

        let right_bound = self.stage.stage_width_pixels() - self.pc.col_width_half() - 1;
        if self.pc.pos_x > right_bound {
            self.pc.pos_x = right_bound;
            self.pc.run_velocity = 0.0;
        }
        if self.pc.pos_x < 0 {
            self.pc.pos_x = 0;
            self.pc.run_velocity = 0.0;
        }
    }

    /// `PC_Jump()` — start a jump.
    ///
    /// Returns `true` when the caller should play the jump sound; the C++ calls
    /// `AUDIO_Sound_Play` inline, which this crate cannot do without an SDL
    /// dependency.
    ///
    /// `on_ground_delay > 0` is the coyote-time arm: for
    /// [`on_ground_delay_max`](crate::player::Pc::on_ground_delay_max)
    /// milliseconds after walking off a ledge the jump still starts.
    pub fn jump(&mut self) -> bool {
        if self.pc.jump_on_going && (self.pc.on_ground || self.pc.on_ground_delay > 0) {
            self.pc.jump_velocity = self.pc.jump_strength as f32;
            self.pc.on_ground = false;
            self.pc.on_ground_delay = 0;
            return true;
        }
        false
    }

    /// `PC_Gravity()` — apply gravity, then move vertically.
    ///
    /// `fast_fall` is the C++ `Key_DOWN` read and `jump_held` is `Unified.Jump`;
    /// both are input state, passed in rather than read from a global.
    ///
    /// Falling is pixel-perfect (one pixel per step, collision re-run each
    /// time); rising is a single jump. That asymmetry is in the original and is
    /// what stops a fast fall from tunnelling through a one-tile floor.
    pub fn gravity(&mut self, fast_fall: bool, jump_held: bool) {
        let g = self.world.gravity as f32;
        self.pc.jump_velocity -= g;
        let terminal = -(self.world.terminal_velocity as f32);
        if self.pc.jump_velocity < terminal {
            self.pc.jump_velocity = terminal;
        }

        self.pc.movement_vertical = (self.pc.jump_velocity / 2.0) as i32;

        // Falling.
        if !self.pc.dead && self.pc.movement_vertical < 0 {
            while self.pc.movement_vertical != 0 {
                self.pc.movement_vertical += 1;
                if !self.pc.on_ground {
                    self.pc.pos_y += 1;
                    self.check_tile_collision();
                }
            }
        }

        // Rising.
        if !self.pc.dead && self.pc.movement_vertical > 0 {
            self.pc.pos_y -= (self.pc.jump_velocity / 2.0) as i32;
        }

        // Above the top of the screen, keep moving by the same rule. The C++
        // applies this in ADDITION to the rising arm, so a rising player above
        // the screen top moves twice in one frame; that is transcribed as-is.
        if self.pc.pos_y < 0 {
            self.pc.pos_y -= (self.pc.jump_velocity / 2.0) as i32;
        }

        if fast_fall {
            self.pc.jump_velocity = terminal;
        }

        // Releasing the jump button cuts the rise short — the variable-height
        // jump.
        if !self.pc.dead && !jump_held && self.pc.jump_velocity > 0.0 {
            self.pc.jump_velocity -= 4.0 * g;
        }

        if self.pc.dead {
            self.pc.pos_y -= (self.pc.jump_velocity / 2.0) as i32;
            if self.pc.jump_velocity > 0.0 {
                self.pc.jump_velocity -= 4.0 * g;
            }
        }
    }
}
