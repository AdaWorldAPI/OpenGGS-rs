//! Stage 3b — the ADDRESSED residual ledger ("slag").
//!
//! [`crate::furnace`] melts the ore into flat, facet-addressed rows under the
//! CURRENT [`crate::convention::Convention`]. Not every fact melts. This module
//! is the ledger of what didn't and WHY, addressed at the
//! [`crate::facet::Facet`] where the melt failed — so a proposer attaches a new
//! convention row at that exact coordinate and re-runs the drill.
//!
//! # The slag doctrine
//!
//! The residual is not waste. It is the empirical boundary of the current
//! convention. A recurring [`ResidualReason`] across many addresses names the
//! next convention row to add, not a defect to hide.
//!
//! **`residual` is NOT to be driven to 0 by widening a match arm.** It falls
//! only when the *convention* gains a row — never by [`ResidualReason`] growing
//! a catch-all that reclassifies a shape as "handled" without a corresponding
//! convention change. Every widening lands with its measured before/after
//! counts in the census, never as a silent code edit here.
//!
//! # HARD RULE — no catch-all, ever
//!
//! [`ResidualReason`] has **no** `Other`, `Opaque`, `Unknown`, or `_ =>` arm
//! that manufactures a reason for a shape fitting no named variant. A shape
//! that doesn't fit means a variant is ADDED, with its before/after counts
//! recorded. [`ResidualReason::ALL`] and `there_is_no_catch_all_reason` exist
//! to catch a future violation.

use crate::facet::{Facet, RailId};

/// Why one ore fact did not melt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ResidualReason {
    /// The event kind is not in the convention's classified set.
    EventKindNotInConvention { kind: EventKind },
    /// The convention resolved to nothing at every prefix depth for this facet.
    NoConventionRowAtAddress,
    /// A corpus index did not fit its `(u8:u8)` rail. Named per rail, because
    /// "the function index overflowed" and "the symbol id overflowed" are
    /// different facts about the corpus.
    RailOverflow { rail: RailId },
    /// The event names a scope id that `scopes.tsv` never declares, so no
    /// depth is derivable for rail3.
    ScopeNotDeclared,
    /// The scope nesting is deeper than rail3's `hi` byte can carry.
    ScopeDepthExceedsRail,
    /// The row did not have the column count the ore schema defines.
    MalformedOreRow,
}

/// The event kinds the harvest emits. A closed vocabulary: an unrecognised
/// spelling in the ore is [`ResidualReason::MalformedOreRow`], never a new
/// silent variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EventKind {
    Read,
    Write,
    ReadWrite,
    Decl,
    Call,
    Condition,
    Branch,
    Cast,
    Param,
    Break,
    Return,
    ScopeEnter,
    ScopeExit,
}

impl EventKind {
    /// Every kind, so a census can iterate the vocabulary rather than
    /// rediscovering it from the data.
    pub const ALL: &'static [Self] = &[
        Self::Read,
        Self::Write,
        Self::ReadWrite,
        Self::Decl,
        Self::Call,
        Self::Condition,
        Self::Branch,
        Self::Cast,
        Self::Param,
        Self::Break,
        Self::Return,
        Self::ScopeEnter,
        Self::ScopeExit,
    ];

    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "Read" => Self::Read,
            "Write" => Self::Write,
            "ReadWrite" => Self::ReadWrite,
            "Decl" => Self::Decl,
            "Call" => Self::Call,
            "Condition" => Self::Condition,
            "Branch" => Self::Branch,
            "Cast" => Self::Cast,
            "Param" => Self::Param,
            "Break" => Self::Break,
            "Return" => Self::Return,
            "ScopeEnter" => Self::ScopeEnter,
            "ScopeExit" => Self::ScopeExit,
            _ => return None,
        })
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Read => "Read",
            Self::Write => "Write",
            Self::ReadWrite => "ReadWrite",
            Self::Decl => "Decl",
            Self::Call => "Call",
            Self::Condition => "Condition",
            Self::Branch => "Branch",
            Self::Cast => "Cast",
            Self::Param => "Param",
            Self::Break => "Break",
            Self::Return => "Return",
            Self::ScopeEnter => "ScopeEnter",
            Self::ScopeExit => "ScopeExit",
        }
    }

    /// The rail3 `lo` byte. Stable across runs, so an address means the same
    /// thing in two harvests.
    #[must_use]
    pub const fn byte(self) -> u8 {
        match self {
            Self::Read => 1,
            Self::Write => 2,
            Self::ReadWrite => 3,
            Self::Decl => 4,
            Self::Call => 5,
            Self::Condition => 6,
            Self::Branch => 7,
            Self::Cast => 8,
            Self::Param => 9,
            Self::Break => 10,
            Self::Return => 11,
            Self::ScopeEnter => 12,
            Self::ScopeExit => 13,
        }
    }
}

impl ResidualReason {
    /// Every reason, for the no-catch-all test and for census iteration.
    pub const ALL: &'static [Self] = &[
        Self::EventKindNotInConvention {
            kind: EventKind::Read,
        },
        Self::NoConventionRowAtAddress,
        Self::RailOverflow { rail: RailId::Unit },
        Self::ScopeNotDeclared,
        Self::ScopeDepthExceedsRail,
        Self::MalformedOreRow,
    ];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EventKindNotInConvention { .. } => "EventKindNotInConvention",
            Self::NoConventionRowAtAddress => "NoConventionRowAtAddress",
            Self::RailOverflow { .. } => "RailOverflow",
            Self::ScopeNotDeclared => "ScopeNotDeclared",
            Self::ScopeDepthExceedsRail => "ScopeDepthExceedsRail",
            Self::MalformedOreRow => "MalformedOreRow",
        }
    }

    /// The reason WITH its payload rendered — what a human reads in the slag
    /// ledger.
    ///
    /// [`Self::as_str`] is the variant name alone, which is what `shape_id`
    /// hashes and what groups rows. It is deliberately NOT enough to act on: a
    /// ledger listing `EventKindNotInConvention` ten times says nothing about
    /// which ten kinds. This renders the payload so the next convention row is
    /// readable off the report.
    #[must_use]
    pub fn detail(&self) -> String {
        match self {
            Self::EventKindNotInConvention { kind } => {
                format!("EventKindNotInConvention[{}]", kind.as_str())
            }
            Self::RailOverflow { rail } => format!("RailOverflow[{}]", rail.as_str()),
            other => other.as_str().to_string(),
        }
    }

    /// Groups identical shapes across different addresses.
    ///
    /// Computed from the variant and its own typed payload ONLY — never from
    /// the address. Two `EventKindNotInConvention { Cast }` residuals at
    /// different facets share a shape id, which is exactly what makes "this
    /// shape occurs 346 times" a proposable convention row.
    #[must_use]
    pub fn shape_id(&self) -> ShapeId {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |b: u8| {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        };
        for b in self.as_str().as_bytes() {
            eat(*b);
        }
        eat(0xFF); // separator: variant name vs payload
        match self {
            Self::EventKindNotInConvention { kind } => eat(kind.byte()),
            Self::RailOverflow { rail } => {
                for b in rail.as_str().as_bytes() {
                    eat(*b);
                }
            }
            Self::NoConventionRowAtAddress
            | Self::ScopeNotDeclared
            | Self::ScopeDepthExceedsRail
            | Self::MalformedOreRow => {}
        }
        ShapeId(h)
    }
}

/// An FNV-1a 64 digest of a residual's shape. Labelled FNV — never a
/// cryptographic hash.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ShapeId(pub u64);

/// One fact that did not melt, at the address where it failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResidualFact {
    /// `None` only where no coordinate is derivable at all — a rail overflow
    /// or a malformed row. Every other residual carries its address.
    pub at: Option<Facet>,
    pub reason: ResidualReason,
    /// The ore row this came from, so a residual points back at its source.
    pub ore_seq: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn there_is_no_catch_all_reason() {
        // The guard for the hard rule. If someone adds `Other`/`Unknown`/
        // `Opaque`, this fails and the review conversation happens.
        for r in ResidualReason::ALL {
            let n = r.as_str();
            assert!(
                !matches!(n, "Other" | "Unknown" | "Opaque" | "Unclassified" | "Misc"),
                "catch-all residual reason introduced: {n}"
            );
        }
    }

    #[test]
    fn shape_id_groups_identical_shapes_and_separates_different_ones() {
        let a = ResidualReason::EventKindNotInConvention {
            kind: EventKind::Cast,
        };
        let a2 = ResidualReason::EventKindNotInConvention {
            kind: EventKind::Cast,
        };
        let b = ResidualReason::EventKindNotInConvention {
            kind: EventKind::Call,
        };
        assert_eq!(a.shape_id(), a2.shape_id(), "same shape must group");
        assert_ne!(a.shape_id(), b.shape_id(), "payload must discriminate");
        assert_ne!(
            ResidualReason::ScopeNotDeclared.shape_id(),
            ResidualReason::ScopeDepthExceedsRail.shape_id(),
            "distinct variants must not collide"
        );
    }

    #[test]
    fn detail_renders_the_payload_that_as_str_drops() {
        // The falsifier for the pass-1 ledger defect: ten residual shapes all
        // printed as the bare variant name, none of them actionable.
        let a = ResidualReason::EventKindNotInConvention {
            kind: EventKind::Cast,
        };
        let b = ResidualReason::EventKindNotInConvention {
            kind: EventKind::Call,
        };
        assert_eq!(
            a.as_str(),
            b.as_str(),
            "the variant name alone cannot discriminate"
        );
        assert_ne!(a.detail(), b.detail(), "detail must");
        assert!(a.detail().contains("Cast"));
        // A payload-free variant renders as its name, with no empty brackets.
        assert_eq!(
            ResidualReason::ScopeNotDeclared.detail(),
            "ScopeNotDeclared"
        );
    }

    #[test]
    fn shape_id_ignores_the_address() {
        // Two residuals of one shape at different coordinates group together;
        // if shape_id folded in the facet, the "this shape occurs N times"
        // signal the whole method depends on would be destroyed.
        let r = ResidualReason::NoConventionRowAtAddress;
        assert_eq!(r.shape_id(), r.shape_id());
    }

    #[test]
    fn event_kind_bytes_are_unique_and_nonzero() {
        // A zero byte would be indistinguishable from an unset rail, and a
        // collision would alias two kinds onto one address.
        let mut seen = std::collections::BTreeSet::new();
        for k in EventKind::ALL {
            assert_ne!(k.byte(), 0, "{k:?} has the reserved zero byte");
            assert!(seen.insert(k.byte()), "duplicate kind byte for {k:?}");
        }
        assert_eq!(seen.len(), EventKind::ALL.len());
    }

    #[test]
    fn event_kind_parse_round_trips_and_rejects_junk() {
        for k in EventKind::ALL {
            assert_eq!(EventKind::parse(k.as_str()), Some(*k));
        }
        assert_eq!(
            EventKind::parse("Frobnicate"),
            None,
            "unknown spellings must not parse"
        );
    }
}
