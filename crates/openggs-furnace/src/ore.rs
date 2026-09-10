//! Stage 2 — **ore**: deterministic typed fact enumeration over the harvest.
//!
//! The intake arm is the `ruff_cpp_spo` behavioural harvest — four TSVs
//! (`events` / `scopes` / `methods` / `symbols`) written by a libclang walk.
//! This module reads them and enumerates one [`OreFact`] per event row, in
//! source order, interning the corpus axes so a facet rail has a stable index.
//!
//! **Nothing is classified here.** The ore is the complete, typed enumeration;
//! deciding what melts is [`crate::furnace`]'s job under the convention. Mixing
//! the two would make "this fact did not melt" indistinguishable from "this fact
//! was never enumerated", and the conservation ledger depends on telling them
//! apart.
//!
//! # Interning is sorted, not first-seen
//!
//! Unit and function indices come from a sorted key order, so two harvests of
//! the same corpus produce the same addresses. First-seen order would make a
//! facet depend on TU traversal order, and a config row pinned to
//! `function=31` would then drift to a different function between runs.

use std::collections::{BTreeMap, BTreeSet};

use crate::slag::EventKind;

/// One enumerated fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OreFact {
    Event(Event),
    /// A row that did not match the ore schema. Enumerated rather than skipped,
    /// so it reaches the furnace and becomes exactly one named residual.
    MalformedRow {
        ore_seq: u32,
    },
}

/// One event from `events.tsv`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Event {
    /// Index into [`Ore::units`].
    pub unit: u32,
    /// Index into [`Ore::functions`].
    pub function: u32,
    /// Scope id within the function.
    pub scope: u32,
    /// Event sequence within the function.
    pub seq: u32,
    pub kind: EventKind,
    /// Interned symbol id from the `subj` column; 0 when absent.
    pub symbol: u32,
    /// Source line.
    pub anchor: u32,
    /// Row number in `events.tsv`, for provenance.
    pub ore_seq: u32,
}

/// The read harvest.
#[derive(Debug, Clone, Default)]
pub struct Ore {
    /// Translation unit paths, sorted. The rail0 index is the position here.
    pub units: Vec<String>,
    /// Fully-qualified function iris, sorted. The rail1 index.
    pub functions: Vec<String>,
    /// `(function index, scope id) -> depth`, from `scopes.tsv`. A scope absent
    /// here is what [`crate::slag::ResidualReason::ScopeNotDeclared`] names.
    pub scope_depth: BTreeMap<(u32, u32), u32>,
    pub facts: Vec<OreFact>,
}

/// Why the harvest could not be read at all.
#[derive(Debug)]
pub enum OreError {
    Io(std::io::Error),
}

impl std::fmt::Display for OreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "reading the harvest: {e}"),
        }
    }
}

impl std::error::Error for OreError {}

impl From<std::io::Error> for OreError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

/// `s253` -> 253. `-` and anything unparsable -> 0, which the facet reads as an
/// unset symbol rail.
fn symbol_id(cell: &str) -> u32 {
    cell.strip_prefix('s')
        .and_then(|n| n.parse().ok())
        .unwrap_or(0)
}

/// `c4` -> 4.
fn scope_id(cell: &str) -> Option<u32> {
    cell.strip_prefix('c').and_then(|n| n.parse().ok())
}

/// The unit path of an iri: `/path/FILE.cpp.Fn()` -> `/path/FILE.cpp`.
///
/// Splitting on the LAST `.cpp.` rather than the first keeps a path containing
/// `.cpp.` in a directory name from cutting in the wrong place.
fn unit_of(iri: &str) -> &str {
    iri.rfind(".cpp.").map_or(iri, |i| &iri[..i + 4])
}

impl Ore {
    /// Read the four-TSV harvest from a directory.
    ///
    /// `methods.tsv` is not read: every fact this furnace melts is an event or a
    /// scope, and reading a file to ignore it would imply it had been consulted.
    pub fn read_dir(dir: &std::path::Path) -> Result<Self, OreError> {
        let events = std::fs::read_to_string(dir.join("events.tsv"))?;
        let scopes = std::fs::read_to_string(dir.join("scopes.tsv"))?;
        Ok(Self::parse(&events, &scopes))
    }

    /// Parse the two TSVs that carry facts.
    #[must_use]
    pub fn parse(events: &str, scopes: &str) -> Self {
        // Pass 1: intern the axes in sorted order.
        let mut unit_set = BTreeSet::new();
        let mut fn_set = BTreeSet::new();
        for line in events.lines().filter(|l| !l.trim().is_empty()) {
            if let Some(iri) = line.split('\t').next() {
                unit_set.insert(unit_of(iri).to_string());
                fn_set.insert(iri.to_string());
            }
        }
        let units: Vec<String> = unit_set.into_iter().collect();
        let functions: Vec<String> = fn_set.into_iter().collect();
        let unit_ix: BTreeMap<&str, u32> = units
            .iter()
            .enumerate()
            .map(|(i, s)| (s.as_str(), i as u32))
            .collect();
        let fn_ix: BTreeMap<&str, u32> = functions
            .iter()
            .enumerate()
            .map(|(i, s)| (s.as_str(), i as u32))
            .collect();

        // Pass 2: scope depths.
        let mut scope_depth = BTreeMap::new();
        for line in scopes.lines().filter(|l| !l.trim().is_empty()) {
            let c: Vec<&str> = line.split('\t').collect();
            if c.len() < 5 {
                continue;
            }
            let (Some(&f), Some(id), Ok(depth)) =
                (fn_ix.get(c[0]), scope_id(c[1]), c[4].parse::<u32>())
            else {
                continue;
            };
            scope_depth.insert((f, id), depth);
        }

        // Pass 3: the facts, in file order.
        //
        // EVERY line is enumerated, blank ones included. `str::lines` already
        // drops the empty element a terminal newline would produce, so an empty
        // line here is an interior blank row — a malformed row, not a nothing.
        // Filtering it out would both lose a fact (breaking conservation) and
        // shift every later `ore_seq`, so the provenance column would point at
        // the wrong source row.
        let mut facts = Vec::new();
        for (row, line) in events.lines().enumerate() {
            let ore_seq = row as u32;
            let c: Vec<&str> = line.split('\t').collect();
            // The schema is 11 columns. `== 11`, not `>= 11`: a permissive arity
            // check is a silent misread waiting for a schema change.
            if c.len() != 11 {
                facts.push(OreFact::MalformedRow { ore_seq });
                continue;
            }
            let (Some(&function), Ok(seq), Some(kind), Some(scope), Ok(anchor)) = (
                fn_ix.get(c[0]),
                c[1].parse::<u32>(),
                EventKind::parse(c[2]),
                scope_id(c[5]),
                c[7].parse::<u32>(),
            ) else {
                facts.push(OreFact::MalformedRow { ore_seq });
                continue;
            };
            let unit = unit_ix.get(unit_of(c[0])).copied().unwrap_or(0);
            facts.push(OreFact::Event(Event {
                unit,
                function,
                scope,
                seq,
                kind,
                symbol: symbol_id(c[3]),
                anchor,
                ore_seq,
            }));
        }

        Self {
            units,
            functions,
            scope_depth,
            facts,
        }
    }

    #[must_use]
    pub fn fact_count(&self) -> usize {
        self.facts.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EVENTS: &str = "\
/p/A.cpp.f()\t0\tScopeEnter\ts0\t-\tc0\t-\t10\t-\t-\twalk
/p/A.cpp.f()\t1\tWrite\ts5\t-\tc0\t-\t11\t-\t-\tclang
/p/B.cpp.g()\t0\tBranch\ts9\t-\tc1\tc0\t20\tif\t-\tclang
";
    const SCOPES: &str = "\
/p/A.cpp.f()\tc0\t-\tFunction\t0\t0\t9
/p/B.cpp.g()\tc1\tc0\tIf\t1\t0\t4
";

    #[test]
    fn parses_events_and_interns_axes_in_sorted_order() {
        let ore = Ore::parse(EVENTS, SCOPES);
        assert_eq!(ore.units, vec!["/p/A.cpp", "/p/B.cpp"]);
        assert_eq!(ore.functions.len(), 2);
        assert_eq!(ore.fact_count(), 3);

        let OreFact::Event(e) = ore.facts[1] else {
            panic!("expected an event")
        };
        assert_eq!(e.kind, EventKind::Write);
        assert_eq!(e.symbol, 5);
        assert_eq!(e.anchor, 11);
        assert_eq!(e.unit, 0, "A.cpp sorts first");
    }

    #[test]
    fn interning_is_stable_under_input_reordering() {
        // The whole point of sorted interning: a config row pinned to
        // function=N must mean the same function in the next harvest.
        let reordered = "\
/p/B.cpp.g()\t0\tBranch\ts9\t-\tc1\tc0\t20\tif\t-\tclang
/p/A.cpp.f()\t0\tScopeEnter\ts0\t-\tc0\t-\t10\t-\t-\twalk
/p/A.cpp.f()\t1\tWrite\ts5\t-\tc0\t-\t11\t-\t-\tclang
";
        let a = Ore::parse(EVENTS, SCOPES);
        let b = Ore::parse(reordered, SCOPES);
        assert_eq!(a.units, b.units);
        assert_eq!(a.functions, b.functions);
    }

    #[test]
    fn scope_depths_are_read() {
        let ore = Ore::parse(EVENTS, SCOPES);
        let g = ore
            .functions
            .iter()
            .position(|f| f.contains("g()"))
            .expect("g") as u32;
        assert_eq!(ore.scope_depth.get(&(g, 1)), Some(&1));
    }

    #[test]
    fn an_interior_blank_row_is_malformed_and_does_not_shift_provenance() {
        // Conservation AND provenance: the blank must become its own fact, and
        // the row after it must still report its true file position.
        let with_blank = "\
/p/A.cpp.f()\t0\tScopeEnter\ts0\t-\tc0\t-\t10\t-\t-\twalk

/p/A.cpp.f()\t1\tWrite\ts5\t-\tc0\t-\t11\t-\t-\tclang
";
        let ore = Ore::parse(with_blank, SCOPES);
        assert_eq!(ore.fact_count(), 3, "the blank row must be enumerated");
        assert!(matches!(ore.facts[1], OreFact::MalformedRow { ore_seq: 1 }));
        let OreFact::Event(e) = ore.facts[2] else {
            panic!("expected an event")
        };
        assert_eq!(e.ore_seq, 2, "ore_seq must stay aligned with the file");
    }

    #[test]
    fn a_trailing_newline_does_not_invent_a_malformed_row() {
        // The other half: only an INTERIOR blank is a fact. A file ending in a
        // newline must not report a phantom row, or every harvest would carry
        // one fabricated residual.
        let ore = Ore::parse(
            "/p/A.cpp.f()\t0\tWrite\ts5\t-\tc0\t-\t11\t-\t-\tclang\n",
            SCOPES,
        );
        assert_eq!(ore.fact_count(), 1);
        assert!(matches!(ore.facts[0], OreFact::Event(_)));
    }

    #[test]
    fn a_short_row_becomes_a_malformed_fact_not_a_skip() {
        // Conservation depends on this: a dropped row would make
        // melted + residual != enumerated, with nothing naming the gap.
        let ore = Ore::parse("/p/A.cpp.f()\t0\tWrite\n", SCOPES);
        assert_eq!(ore.fact_count(), 1);
        assert!(matches!(ore.facts[0], OreFact::MalformedRow { ore_seq: 0 }));
    }

    #[test]
    fn an_unknown_event_kind_is_malformed_not_invented() {
        let ore = Ore::parse(
            "/p/A.cpp.f()\t0\tFrobnicate\ts1\t-\tc0\t-\t1\t-\t-\tclang\n",
            SCOPES,
        );
        assert!(matches!(ore.facts[0], OreFact::MalformedRow { .. }));
    }

    #[test]
    fn extra_columns_are_malformed_too() {
        // `== 11` rather than `>= 11`, asserted.
        let row = "/p/A.cpp.f()\t0\tWrite\ts1\t-\tc0\t-\t1\t-\t-\tclang\textra\n";
        let ore = Ore::parse(row, SCOPES);
        assert!(matches!(ore.facts[0], OreFact::MalformedRow { .. }));
    }

    #[test]
    fn unit_of_splits_on_the_last_cpp_marker() {
        assert_eq!(unit_of("/a/x.cpp.Fn()"), "/a/x.cpp");
        // A directory that itself contains `.cpp.` must not cut early.
        assert_eq!(unit_of("/a.cpp.d/x.cpp.Fn()"), "/a.cpp.d/x.cpp");
        assert_eq!(unit_of("no-marker"), "no-marker");
    }
}
