//! `openggs-spo` — the harvested inventory of the C++ corpus, and the
//! transcode coverage ledger built on it.
//!
//! # Why a crate and not a markdown table
//!
//! The port's progress is a claim about two sets: what the C++ contains, and
//! what Rust has reproduced. A hand-written table drifts the moment either side
//! changes, and a drifted table reads as evidence. So the C++ side is not
//! written by hand at all — it is the output of a libclang walk over all 55
//! translation units, and it lives in `ore/functions.tsv` verbatim:
//!
//! ```sh
//! cmake -S OpenGGS -B build -DCMAKE_EXPORT_COMPILE_COMMANDS=ON
//! ORE_CC_JSON=build/compile_commands.json ORE_OUT=/tmp/ore \
//!   LIBCLANG_PATH=/usr/lib/llvm-18/lib \
//!   cargo run -p ruff_cpp_spo --features libclang --example harvest_events
//! ```
//!
//! That run reported 55/55 translation units parsed with zero error
//! diagnostics, 181 function definitions and 34,326 ordered behavioural events.
//! The function column of `methods.tsv` is what this crate embeds.
//!
//! The Rust side — [`PORTED`] — IS hand-written, because only a human knows
//! that `Sim::run` reproduces `PC_Run()`. What keeps it honest is
//! [`unported`] plus the test below: every name in [`PORTED`] must exist in the
//! harvest. A typo, a rename upstream, or a function that never existed fails
//! the suite instead of inflating the coverage number.
//!
//! # What this deliberately does not claim
//!
//! Coverage here means "a Rust routine exists that was transcoded from this C++
//! function". It is NOT behavioural parity: nothing in this crate runs the C++.
//! Parity against the original binary is unmeasured — see
//! `docs/TRANSCODE-LEDGER.md`.

/// The harvested inventory: `file<TAB>signature`, one per line.
const INVENTORY: &str = include_str!("../../../ore/functions.tsv");

/// One harvested C++ function definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CppFunction<'a> {
    /// The translation unit's stem, e.g. `PLAYER_Run`.
    pub file: &'a str,
    /// The signature as libclang spelled it, e.g. `PC_Run()`.
    pub signature: &'a str,
}

impl CppFunction<'_> {
    /// The bare name, without the parameter list.
    #[must_use]
    pub fn name(&self) -> &str {
        self.signature.split('(').next().unwrap_or(self.signature)
    }
}

/// Every function definition the harvest found, in file order.
#[must_use]
pub fn harvested() -> Vec<CppFunction<'static>> {
    INVENTORY
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| {
            let (file, signature) = l.split_once('\t')?;
            Some(CppFunction { file, signature })
        })
        .collect()
}

/// The C++ functions this workspace has transcoded, by signature, paired with
/// the Rust item that replaced them.
///
/// Ordered by C++ source file so a reader can diff it against the harvest.
pub const PORTED: &[(&str, &str)] = &[
    ("PC_Define()", "openggs_core::player::Pc::default"),
    (
        "PC_PowerUp_Set(int)",
        "openggs_core::player::Pc::set_power_up",
    ),
    (
        "PC_Check_Tilecollision()",
        "openggs_core::Sim::check_tile_collision",
    ),
    ("PC_Collision_Down()", "openggs_core::Sim::collision_down"),
    ("PC_Collision_Up()", "openggs_core::Sim::collision_up"),
    ("PC_Collision_Left()", "openggs_core::Sim::collision_left"),
    ("PC_Collision_Right()", "openggs_core::Sim::collision_right"),
    ("PC_Gravity()", "openggs_core::Sim::gravity"),
    ("PC_Jump()", "openggs_core::Sim::jump"),
    ("PC_Run()", "openggs_core::Sim::run"),
    ("PC_Friction()", "openggs_core::Sim::friction"),
    (
        "GAME_ENVIRONMENT_Define()",
        "openggs_core::world::World::default",
    ),
    ("Load_Stagefile(int)", "openggs_stagefile::StageFile::stage"),
];

/// The harvested functions that have no entry in [`PORTED`].
#[must_use]
pub fn unported() -> Vec<CppFunction<'static>> {
    harvested()
        .into_iter()
        .filter(|f| !PORTED.iter().any(|(sig, _)| *sig == f.signature))
        .collect()
}

/// Ported count over harvested count.
#[must_use]
pub fn coverage() -> (usize, usize) {
    let total = harvested().len();
    (total - unported().len(), total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_harvest_parses() {
        let all = harvested();
        // The harvest run reported 181 function definitions. If the embedded
        // file is regenerated and this number moves, that is a real change in
        // the corpus and the ledger should be re-read, not silently re-pinned.
        assert_eq!(
            all.len(),
            181,
            "embedded inventory no longer matches the reported harvest"
        );
        assert!(
            all.iter()
                .any(|f| f.file == "PLAYER_Run" && f.signature == "PC_Run()")
        );
    }

    #[test]
    fn every_ported_name_exists_in_the_harvest() {
        // The teeth of the ledger: a coverage claim for a function the corpus
        // does not contain is a defect, not an achievement.
        let all = harvested();
        let missing: Vec<_> = PORTED
            .iter()
            .map(|(sig, _)| *sig)
            .filter(|sig| !all.iter().any(|f| f.signature == *sig))
            .collect();
        assert!(
            missing.is_empty(),
            "PORTED names absent from the harvest: {missing:?}"
        );
    }

    #[test]
    fn ported_has_no_duplicates() {
        // A duplicate would inflate nothing (unported() filters by membership)
        // but would mislead a reader counting the list by eye.
        let mut sigs: Vec<_> = PORTED.iter().map(|(s, _)| *s).collect();
        sigs.sort_unstable();
        let before = sigs.len();
        sigs.dedup();
        assert_eq!(before, sigs.len(), "duplicate entry in PORTED");
    }

    #[test]
    fn coverage_is_consistent_and_partial() {
        let (ported, total) = coverage();
        assert_eq!(ported, PORTED.len());
        assert_eq!(ported + unported().len(), total);
        // The port is explicitly incomplete. This asserts the ledger REPORTS
        // that rather than quietly claiming a finished transcode.
        assert!(ported < total, "ledger claims a complete port");
    }

    #[test]
    fn the_unported_set_still_contains_the_known_gaps() {
        // Named in docs/TRANSCODE-LEDGER.md as not-yet-ported. If one of these
        // gets ported, this test is the reminder to update the ledger prose too.
        let unported: Vec<&str> = unported().iter().map(|f| f.signature).collect();
        for gap in [
            "PC_Draw(int,int)",
            "PC_Killed()",
            "PC_Check_Collision_Sprites()",
        ] {
            assert!(
                unported.contains(&gap),
                "{gap} is no longer unported — update the ledger"
            );
        }
    }
}
