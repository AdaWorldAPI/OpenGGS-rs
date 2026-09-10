//! Tile collision.
//!
//! Transcoded from `PC_Check_Tilecollision()`, `PC_Collision_Down()`,
//! `PC_Collision_Up()`, `PC_Collision_Right()` and `PC_Collision_Left()` in
//! `OpenGGS/src/PLAYER_Collision_Solids.cpp`.
//!
//! # What this pass covers, and what it does not
//!
//! The SOLID-TILE arms are transcoded exactly, including the probe sets that
//! differ per velocity sign and the ledge special case. The arms that consult
//! other entities — moving platforms (`Platform.inUse`), drop stones
//! (`Sprite_DropStone[i]`), and the block-contents side effects the C++ fires
//! from inside `PC_Collision_Up` (`PC_Collision_CoinBlock`,
//! `PC_Collision_PowerUpBlock`, `PC_Collision_Breakable`, …) — are NOT here:
//! those subsystems are not ported yet, and a stub that silently answers
//! "no collision" would read like a transcode defect later. They are tracked
//! in `docs/TRANSCODE-LEDGER.md`.

use crate::Sim;
use crate::world::TILE_SIZE;

impl Sim {
    /// Is the tile at this pixel position solid?
    fn solid_at(&self, px: i32, py: i32) -> bool {
        self.tiles.get(self.stage.tile_at_pixel(px, py)).solid
    }

    /// `PC_Check_Tilecollision()` — the dispatcher.
    ///
    /// The C++ clears the four result flags first and then lets each direction
    /// probe set them, so a probe that does not fire leaves the flag false.
    /// That ordering is load-bearing: `hit_wall` is re-derived on every call,
    /// which is what lets the pixel-perfect step loops in
    /// [`Sim::friction`](crate::Sim::friction) re-test it each pixel.
    pub fn check_tile_collision(&mut self) {
        self.stop_running_left = false;
        self.stop_running_right = false;
        self.pc.on_ground = false;
        self.pc.hit_wall = false;
        self.pc.wall_grinding = false;
        self.pc.on_platform = false;

        if self.collision_right() {
            self.pc.run_velocity = 0.0;
            self.pc.hit_wall = true;
            if !self.gv.pixel_perfect_running {
                self.pc.pos_x -= (self.pc.pos_x + self.pc.col_width_half()) % TILE_SIZE;
            }
        }

        if self.collision_left() {
            self.pc.run_velocity = 0.0;
            self.pc.hit_wall = true;
            if !self.gv.pixel_perfect_running {
                self.pc.pos_x +=
                    TILE_SIZE - 1 - ((self.pc.pos_x - self.pc.col_width_half()) % TILE_SIZE);
            }
        }

        if self.collision_down() {
            self.pc.jump_on_going = false;
            self.pc.on_ground = true;
            self.pc.on_ground_delay = self.pc.on_ground_delay_max;
            self.pc.jump_velocity = 0.0;
        }

        if self.collision_up() {
            self.pc.jump_on_going = false;
            self.pc.jump_velocity = 0.0;
        }
    }

    /// `PC_Collision_Down()`, solid-tile arms.
    ///
    /// The three velocity arms use DIFFERENT probe sets — moving right drops
    /// the rightmost probe, moving left drops the leftmost — which is the C++
    /// comment's "prevents getting stuck on the wall". Collapsing them to one
    /// five-probe set would change behaviour at wall corners, so they stay
    /// three arms.
    fn collision_down(&mut self) -> bool {
        let mut hit = false;
        let y = self.pc.pos_y;
        let (x, half, quart) = (
            self.pc.pos_x,
            self.pc.col_width_half(),
            self.pc.col_width_quart(),
        );

        let left_edge = x - half + 1;
        let right_edge = x + half - 1;

        if self.pc.jump_velocity < 0.0 {
            let probes: &[i32] = if self.pc.run_velocity == 0.0 {
                &[left_edge, x - quart, x, x + quart, right_edge]
            } else if self.pc.run_velocity > 0.0 {
                &[left_edge, x - quart, x, x + quart]
            } else {
                &[x - quart, x, x + quart, right_edge]
            };
            if probes.iter().any(|&px| self.solid_at(px, y)) {
                hit = true;
            }
        }

        // "Make it easier to stand on a ledge": a solid tile under an outer
        // edge with empty space one pixel above it counts as ground regardless
        // of velocity. Without it the C++ comment reports the player sinking
        // into the ground.
        for &px in &[right_edge, left_edge] {
            if self.solid_at(px, y) && !self.solid_at(px, y - 1) {
                hit = true;
            }
        }

        // Falling off the bottom of the screen kills, and is reported as a
        // collision so the caller stops the fall.
        if self.pc.pos_y > 479 {
            hit = true;
            self.pc.got_killed = true;
        }
        // Above the top of the screen never collides, so the player cannot get
        // stuck there.
        if self.pc.pos_y < 0 {
            hit = false;
        }
        hit
    }

    /// `PC_Collision_Up()`, solid-tile arms.
    fn collision_up(&mut self) -> bool {
        let mut hit = false;
        let y = self.pc.pos_y - self.pc.col_height;
        let (x, half, quart) = (
            self.pc.pos_x,
            self.pc.col_width_half(),
            self.pc.col_width_quart(),
        );

        if self.pc.jump_velocity > 0.0 {
            let probes: &[i32] = if self.pc.run_velocity == 0.0 {
                &[x - half + 1, x - quart, x, x + quart, x + half - 1]
            } else if self.pc.run_velocity > 0.0 {
                &[x - half + 1, x - quart, x, x + quart]
            } else {
                &[x - quart, x, x + quart, x + half - 1]
            };
            if probes.iter().any(|&px| self.solid_at(px, y)) {
                hit = true;
            }
        }

        // Jumping out of the top of the screen is allowed.
        if self.pc.pos_y < self.pc.col_height {
            hit = false;
        }
        hit
    }

    /// `PC_Collision_Right()`, solid-tile arms.
    ///
    /// Note the in-probe mutation: a hit nudges `pos_x` back by one pixel while
    /// pixel-perfect running is on. That write happens inside the predicate in
    /// the C++ too — it is how the player is pushed out of the wall — so this
    /// takes `&mut self` rather than being a pure query.
    fn collision_right(&mut self) -> bool {
        let mut hit = false;
        if self.pc.run_velocity > 0.0
            && (self.pc.pos_y > self.pc.col_height || !self.gv.pixel_perfect_running)
        {
            let px = self.pc.pos_x + self.pc.col_width_half();
            if self.solid_at(px, self.pc.pos_y - self.pc.col_height_half())
                || self.solid_at(px, self.pc.pos_y - 1)
            {
                hit = true;
                if self.gv.pixel_perfect_running {
                    self.pc.pos_x -= 1;
                }
                if self.pc.accell_right {
                    self.pc.wall_grinding = true;
                }
            }
            // Falling fast past a wall: only the bottom probe counts, so the
            // player slides down rather than catching on the wall's centre.
            if self.pc.jump_velocity < -5.0 {
                hit = false;
                if self.solid_at(self.pc.pos_x + self.pc.col_width_half(), self.pc.pos_y - 1) {
                    hit = true;
                    if self.pc.accell_right {
                        self.pc.wall_grinding = true;
                    }
                }
            }
        }
        hit
    }

    /// `PC_Collision_Left()`, solid-tile arms — the mirror of
    /// [`Sim::collision_right`].
    fn collision_left(&mut self) -> bool {
        let mut hit = false;
        if self.pc.run_velocity < 0.0
            && (self.pc.pos_y > self.pc.col_height || !self.gv.pixel_perfect_running)
        {
            let px = self.pc.pos_x - self.pc.col_width_half();
            if self.solid_at(px, self.pc.pos_y - self.pc.col_height_half())
                || self.solid_at(px, self.pc.pos_y - 1)
            {
                hit = true;
                if self.gv.pixel_perfect_running {
                    self.pc.pos_x += 1;
                }
                if self.pc.accell_left {
                    self.pc.wall_grinding = true;
                }
            }
            if self.pc.jump_velocity < -5.0 {
                hit = false;
                if self.solid_at(self.pc.pos_x - self.pc.col_width_half(), self.pc.pos_y - 1) {
                    hit = true;
                    if self.pc.accell_left {
                        self.pc.wall_grinding = true;
                    }
                }
            }
        }
        hit
    }
}
