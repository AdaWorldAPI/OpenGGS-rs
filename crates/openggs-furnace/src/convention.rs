//! The longest-prefix-wins config tree over facet space — **data as config**.
//!
//! # This module ships ZERO C++ vocabulary
//!
//! No event kind is hardcoded as "melts", no concern is assigned by matching on
//! a kind, no function or file name appears anywhere below. Everything the
//! convention knows arrives as **data**, via [`Convention::from_config`] or
//! [`Convention::insert`]. That is the whole point: the config accumulates one
//! row at a time as the slag names what is missing, and a row is added to a
//! *file*, never to a match arm in Rust.
//!
//! If you find yourself wanting to write `EventKind::Cast => Concern::...` in
//! this module, the convention file is the place for it.
//!
//! # Not a flat table — a radix tree
//!
//! Rows attach at a prefix depth over [`Facet`] space: depth 0 is the
//! corpus-wide default, depth 1 a translation unit, depth 2 one function,
//! depth 3 one scope. [`Convention::resolve`] walks from the FINEST prefix down
//! to the coarsest and returns the first row it finds — longest matching prefix
//! wins, one rule at every level.
//!
//! An address the convention says nothing about resolves to `None`, which is
//! exactly the addressed-residual case [`crate::slag`] names: the proposer
//! emits a row AT that address and the next drill melts it.

use std::collections::{BTreeMap, BTreeSet};

use crate::facet::Facet;
use crate::slag::EventKind;

/// Which concern a melted row belongs to. Concern-separation is what lets a
/// census ask "is this function data or behaviour" without re-reading the ore.
///
/// Assigned by a convention ROW, never derived from the event kind in code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Concern {
    /// Reads, writes and declarations of state.
    State,
    /// Branching, conditions, loop exits, returns.
    Control,
    /// Scope entry and exit — the shape of the body.
    Structure,
    /// Calls and parameters — the edge to other functions.
    Interface,
    /// Type conversions.
    Conversion,
}

impl Concern {
    pub const ALL: &'static [Self] = &[
        Self::State,
        Self::Control,
        Self::Structure,
        Self::Interface,
        Self::Conversion,
    ];

    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "State" => Self::State,
            "Control" => Self::Control,
            "Structure" => Self::Structure,
            "Interface" => Self::Interface,
            "Conversion" => Self::Conversion,
            _ => return None,
        })
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::State => "State",
            Self::Control => "Control",
            Self::Structure => "Structure",
            Self::Interface => "Interface",
            Self::Conversion => "Conversion",
        }
    }
}

/// One convention row: what the convention knows at an address.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConventionRow {
    pub concern: Concern,
    /// Which event kind this row speaks for, or `None` for "any classified
    /// kind at this address".
    pub kind: Option<EventKind>,
}

/// A facet prefix: the address plus how deep the row attaches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FacetPrefix {
    pub depth: u8,
    pub facet: Facet,
}

impl FacetPrefix {
    /// Normalises the facet to the prefix depth, so two rows that name the same
    /// prefix with different trailing rails cannot both exist.
    #[must_use]
    pub fn new(depth: u8, facet: Facet) -> Self {
        Self {
            depth,
            facet: facet.prefix(usize::from(depth)),
        }
    }
}

/// The config tree.
#[derive(Debug, Clone, Default)]
pub struct Convention {
    classified: BTreeSet<EventKind>,
    /// Keyed by prefix AND the row's kind scope: two rows at one prefix are
    /// legitimate when they speak for different kinds, which is how a concern
    /// is assigned per kind at the corpus-wide default depth.
    rows: BTreeMap<(FacetPrefix, Option<EventKind>), ConventionRow>,
}

/// Why a convention file did not load. Named per failure, so a bad config says
/// which line and what was wrong.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    UnknownDirective {
        line: usize,
    },
    UnknownEventKind {
        line: usize,
    },
    UnknownConcern {
        line: usize,
    },
    BadField {
        line: usize,
    },
    DepthOutOfRange {
        line: usize,
    },
    /// Two rows claim the same prefix, so longest-prefix-wins would be
    /// ambiguous. Rejected rather than silently letting one win.
    DuplicatePrefix {
        line: usize,
    },
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (what, line) = match self {
            Self::UnknownDirective { line } => ("unknown directive", line),
            Self::UnknownEventKind { line } => ("unknown event kind", line),
            Self::UnknownConcern { line } => ("unknown concern", line),
            Self::BadField { line } => ("malformed field", line),
            Self::DepthOutOfRange { line } => ("depth must be 0..=6", line),
            Self::DuplicatePrefix { line } => ("two rows claim the same prefix", line),
        };
        write!(f, "convention line {line}: {what}")
    }
}

impl std::error::Error for ConfigError {}

impl Convention {
    /// Parse a convention file.
    ///
    /// Line-oriented, `#` comments, blank lines ignored:
    ///
    /// ```text
    /// classify Read
    /// row depth=0 concern=State
    /// row depth=2 unit=7 function=31 concern=Control kind=Branch
    /// ```
    ///
    /// The format is deliberately tiny and parsed here rather than pulled in
    /// with a TOML dependency: the crate has zero dependencies, and a config
    /// this shape does not need a parser library.
    pub fn from_config(text: &str) -> Result<Self, ConfigError> {
        let mut conv = Self::default();
        for (i, raw) in text.lines().enumerate() {
            let line = i + 1;
            let s = raw.split('#').next().unwrap_or("").trim();
            if s.is_empty() {
                continue;
            }
            let mut toks = s.split_whitespace();
            match toks.next() {
                Some("classify") => {
                    let k = toks.next().ok_or(ConfigError::BadField { line })?;
                    conv.classified
                        .insert(EventKind::parse(k).ok_or(ConfigError::UnknownEventKind { line })?);
                }
                Some("row") => {
                    let (mut depth, mut concern, mut kind) = (None, None, None);
                    let mut facet = Facet::default();
                    for t in toks {
                        let (k, v) = t.split_once('=').ok_or(ConfigError::BadField { line })?;
                        match k {
                            "depth" => {
                                let d: u8 =
                                    v.parse().map_err(|_| ConfigError::BadField { line })?;
                                if d > 6 {
                                    return Err(ConfigError::DepthOutOfRange { line });
                                }
                                depth = Some(d);
                            }
                            "concern" => {
                                concern = Some(
                                    Concern::parse(v)
                                        .ok_or(ConfigError::UnknownConcern { line })?,
                                );
                            }
                            "kind" => {
                                kind = Some(
                                    EventKind::parse(v)
                                        .ok_or(ConfigError::UnknownEventKind { line })?,
                                );
                            }
                            "unit" | "function" | "scope" => {
                                let n: u32 =
                                    v.parse().map_err(|_| ConfigError::BadField { line })?;
                                let rail = crate::facet::Rail::split(n)
                                    .ok_or(ConfigError::BadField { line })?;
                                let idx = match k {
                                    "unit" => 0,
                                    "function" => 1,
                                    _ => 2,
                                };
                                facet.rails[idx] = rail;
                            }
                            _ => return Err(ConfigError::BadField { line }),
                        }
                    }
                    let depth = depth.ok_or(ConfigError::BadField { line })?;
                    let concern = concern.ok_or(ConfigError::BadField { line })?;
                    let prefix = FacetPrefix::new(depth, facet);
                    if conv
                        .rows
                        .insert((prefix, kind), ConventionRow { concern, kind })
                        .is_some()
                    {
                        return Err(ConfigError::DuplicatePrefix { line });
                    }
                }
                _ => return Err(ConfigError::UnknownDirective { line }),
            }
        }
        Ok(conv)
    }

    /// Render the convention back out, so an emitted artifact and a loaded file
    /// are the same language.
    #[must_use]
    pub fn to_config(&self) -> String {
        let mut out = String::new();
        for k in &self.classified {
            out.push_str("classify ");
            out.push_str(k.as_str());
            out.push('\n');
        }
        for ((p, _), r) in &self.rows {
            out.push_str(&format!("row depth={}", p.depth));
            if p.depth >= 1 {
                out.push_str(&format!(" unit={}", p.facet.unit()));
            }
            if p.depth >= 2 {
                out.push_str(&format!(" function={}", p.facet.function()));
            }
            if p.depth >= 3 {
                out.push_str(&format!(" scope={}", p.facet.scope()));
            }
            out.push_str(&format!(" concern={}", r.concern.as_str()));
            if let Some(k) = r.kind {
                out.push_str(&format!(" kind={}", k.as_str()));
            }
            out.push('\n');
        }
        out
    }

    pub fn classify(&mut self, kind: EventKind) {
        self.classified.insert(kind);
    }

    #[must_use]
    pub fn classifies(&self, kind: EventKind) -> bool {
        self.classified.contains(&kind)
    }

    pub fn classified_kinds(&self) -> impl Iterator<Item = &EventKind> {
        self.classified.iter()
    }

    pub fn insert(&mut self, prefix: FacetPrefix, row: ConventionRow) {
        self.rows.insert((prefix, row.kind), row);
    }

    #[must_use]
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Longest matching prefix wins: walk depth 6 down to 0, return the first
    /// row whose prefix matches and whose `kind` admits this event.
    #[must_use]
    pub fn resolve(&self, facet: &Facet, kind: EventKind) -> Option<&ConventionRow> {
        // At each depth: a row scoped to THIS kind wins over the any-kind row,
        // and only then does the walk fall to the next coarser prefix. Checking
        // any-kind first would let a corpus-wide default shadow a per-kind row
        // at the same address.
        for depth in (0..=6u8).rev() {
            let p = FacetPrefix::new(depth, *facet);
            if let Some(r) = self.rows.get(&(p, Some(kind))) {
                return Some(r);
            }
            if let Some(r) = self.rows.get(&(p, None)) {
                return Some(r);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::facet::Rail;

    fn facet(unit: u16, function: u16, scope: u16) -> Facet {
        Facet {
            classid: 0,
            rails: [
                Rail::split(u32::from(unit)).expect("fits"),
                Rail::split(u32::from(function)).expect("fits"),
                Rail::split(u32::from(scope)).expect("fits"),
                Rail::default(),
                Rail::default(),
                Rail::default(),
            ],
        }
    }

    #[test]
    fn a_config_round_trips_through_text() {
        let src = "classify Read\nclassify Branch\nrow depth=0 concern=State\n\
                   row depth=2 unit=7 function=31 concern=Control kind=Branch\n";
        let c = Convention::from_config(src).expect("parses");
        let again = Convention::from_config(&c.to_config()).expect("re-parses its own output");
        assert_eq!(again.row_count(), c.row_count());
        assert!(again.classifies(EventKind::Read));
        assert!(again.classifies(EventKind::Branch));
        assert!(
            !again.classifies(EventKind::Cast),
            "only what the file said"
        );
    }

    #[test]
    fn longest_prefix_wins() {
        let src = "classify Read\nrow depth=0 concern=State\n\
                   row depth=2 unit=7 function=31 concern=Control\n";
        let c = Convention::from_config(src).expect("parses");
        // The specific function gets the deep row...
        assert_eq!(
            c.resolve(&facet(7, 31, 4), EventKind::Read)
                .map(|r| r.concern),
            Some(Concern::Control)
        );
        // ...and everything else falls back to the corpus-wide default.
        assert_eq!(
            c.resolve(&facet(7, 32, 4), EventKind::Read)
                .map(|r| r.concern),
            Some(Concern::State)
        );
    }

    #[test]
    fn a_kind_scoped_row_does_not_answer_for_another_kind() {
        // The falsifier for `kind=`: without the filter, the Branch row would
        // capture Reads at the same address too.
        let src = "classify Read\nclassify Branch\n\
                   row depth=2 unit=1 function=2 concern=Control kind=Branch\n";
        let c = Convention::from_config(src).expect("parses");
        assert!(c.resolve(&facet(1, 2, 0), EventKind::Branch).is_some());
        assert!(
            c.resolve(&facet(1, 2, 0), EventKind::Read).is_none(),
            "the Branch row must not answer for a Read"
        );
    }

    #[test]
    fn an_address_the_convention_is_silent_about_resolves_to_none() {
        // This is the addressed-residual case, and it must stay reachable: a
        // convention that answers everything produces no slag and therefore no
        // signal about what it is missing.
        let c = Convention::from_config("classify Read\n").expect("parses");
        assert!(c.resolve(&facet(1, 1, 1), EventKind::Read).is_none());
    }

    #[test]
    fn two_rows_at_one_prefix_are_fine_when_they_scope_different_kinds() {
        // This is how a concern is assigned per kind at the default depth. It
        // must NOT read as a duplicate.
        let c = Convention::from_config(
            "classify Read\nclassify Branch\n\
             row depth=0 concern=State kind=Read\n\
             row depth=0 concern=Control kind=Branch\n",
        )
        .expect("per-kind rows coexist");
        assert_eq!(
            c.resolve(&facet(1, 1, 1), EventKind::Read)
                .map(|r| r.concern),
            Some(Concern::State)
        );
        assert_eq!(
            c.resolve(&facet(1, 1, 1), EventKind::Branch)
                .map(|r| r.concern),
            Some(Concern::Control)
        );
    }

    #[test]
    fn a_kind_scoped_row_beats_the_any_kind_row_at_the_same_depth() {
        let c = Convention::from_config(
            "classify Read\nrow depth=0 concern=State\nrow depth=0 concern=Control kind=Read\n",
        )
        .expect("config");
        assert_eq!(
            c.resolve(&facet(1, 1, 1), EventKind::Read)
                .map(|r| r.concern),
            Some(Concern::Control),
            "the more specific row must win"
        );
    }

    #[test]
    fn duplicate_prefixes_are_rejected_not_silently_overwritten() {
        let src = "row depth=1 unit=3 concern=State\nrow depth=1 unit=3 concern=Control\n";
        assert_eq!(
            Convention::from_config(src).unwrap_err(),
            ConfigError::DuplicatePrefix { line: 2 }
        );
    }

    #[test]
    fn malformed_config_is_named_by_line() {
        assert_eq!(
            Convention::from_config("classify Nonsense\n").unwrap_err(),
            ConfigError::UnknownEventKind { line: 1 }
        );
        assert_eq!(
            Convention::from_config("frobnicate\n").unwrap_err(),
            ConfigError::UnknownDirective { line: 1 }
        );
        assert_eq!(
            Convention::from_config("row depth=9 concern=State\n").unwrap_err(),
            ConfigError::DepthOutOfRange { line: 1 }
        );
        // Comments and blank lines are not errors.
        assert!(Convention::from_config("# a note\n\n  \nclassify Read\n").is_ok());
    }

    #[test]
    fn this_module_hardcodes_no_cpp_vocabulary() {
        // The data-as-config guard, checked against this file's own source: no
        // event kind may be wired to a concern here. The concern assignment
        // lives in the convention FILE.
        let src = include_str!("convention.rs");
        let body = src.split("mod tests").next().expect("module body");
        for k in EventKind::ALL {
            for c in Concern::ALL {
                let arm = format!("{}::{} => Concern::{}", "EventKind", k.as_str(), c.as_str());
                assert!(
                    !body.contains(&arm),
                    "concern hardcoded for {k:?} in convention.rs"
                );
            }
        }
    }
}
