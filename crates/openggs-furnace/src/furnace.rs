//! Stage 3 — the **furnace**: melts the ore into flat, facet-addressed,
//! concern-separated rows.
//!
//! [`FlatFact`] is `FactId + Facet + Concern + fixed scalar payload` — a plain
//! flat `Vec<FlatFact>`. **It must never grow back into a nested object graph.**
//! No `Vec`, no `Box`, no map, no `String` inside a `FlatFact`; cardinality is
//! handled by EMITTING MORE ROWS, never by nesting a collection. A row refers to
//! another only by its address.
//!
//! # The melt ladder — nothing else classifies
//!
//! An [`crate::ore::OreFact::Event`] melts iff **all** of:
//!
//! 1. every corpus index fits its `(u8:u8)` rail
//!    (else [`ResidualReason::RailOverflow`], no address derivable);
//! 2. its scope is declared in `scopes.tsv`
//!    (else [`ResidualReason::ScopeNotDeclared`]);
//! 3. that scope's depth fits rail3's `hi` byte
//!    (else [`ResidualReason::ScopeDepthExceedsRail`]);
//! 4. the convention classifies its kind
//!    (else [`ResidualReason::EventKindNotInConvention`]);
//! 5. the convention resolves a row at some prefix of its address
//!    (else [`ResidualReason::NoConventionRowAtAddress`]).
//!
//! An [`crate::ore::OreFact::MalformedRow`] never melts —
//! [`ResidualReason::MalformedOreRow`], with no address.
//!
//! The gates run IN THAT ORDER and each pushes exactly one residual, so a fact
//! blocked by two conditions is reported by the first — the one a proposer must
//! fix before the next becomes observable. Reporting both would name a
//! convention row that cannot be attached yet, because the address it would
//! attach at does not exist.
//!
//! # Conservation
//!
//! Every ore fact produces EXACTLY one [`FlatFact`] or EXACTLY one
//! [`ResidualFact`]. `enumerated == melted + residual` holds by construction of
//! [`smelt`]'s control flow — every path through the loop ends in exactly one
//! push — not by a separate bookkeeping counter that could itself be wrong.
//!
//! # `FlatFact` payload table
//!
//! | `concern` | `a` | `b` |
//! |---|---|---|
//! | any | interned symbol id | source line (anchor) |
//!
//! Both payload slots are the same for every concern today. They are kept as
//! two named `u64`s rather than collapsed into one field because the next
//! convention row that distinguishes concerns will want the second slot, and
//! widening a row later is a layout change where filling a reserved slot is not.

use std::collections::BTreeMap;

use crate::convention::{Concern, Convention};
use crate::facet::{Facet, Rail, RailId};
use crate::ore::{Ore, OreFact};
use crate::slag::{ResidualFact, ResidualReason, ShapeId};

/// A melted row's identity: its position in the melt, not a hash.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FactId(pub u32);

/// One melted fact. Flat by construction — see the module docs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlatFact {
    pub id: FactId,
    pub at: Facet,
    pub concern: Concern,
    pub a: u64,
    pub b: u64,
}

/// The measured result of one drill pass.
#[derive(Debug, Clone)]
pub struct HarvestReport {
    pub melted: Vec<FlatFact>,
    pub residual: Vec<ResidualFact>,
    /// How many ore facts went in.
    pub enumerated: usize,
    pub census: Census,
}

impl HarvestReport {
    /// `melted + residual == enumerated`, and nothing was dropped.
    ///
    /// True by construction; asserted anyway, because "by construction" is a
    /// claim about control flow that an edit can quietly break.
    #[must_use]
    pub fn conserved(&self) -> bool {
        self.melted.len() + self.residual.len() == self.enumerated
    }

    /// Residual shapes, most frequent first. **This is the repeat signal**: the
    /// top shape names the next convention row to add.
    #[must_use]
    pub fn slag_shapes(&self) -> Vec<(ShapeId, ResidualReason, usize)> {
        let mut by: BTreeMap<ShapeId, (ResidualReason, usize)> = BTreeMap::new();
        for r in &self.residual {
            let e = by.entry(r.reason.shape_id()).or_insert((r.reason, 0));
            e.1 += 1;
        }
        let mut v: Vec<_> = by.into_iter().map(|(s, (r, n))| (s, r, n)).collect();
        v.sort_by(|a, b| b.2.cmp(&a.2).then(a.0.cmp(&b.0)));
        v
    }
}

/// Per-function concern counts, derived from the melted rows alone.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FunctionCensus {
    pub function: u32,
    /// Indexed by [`Concern::ALL`] position.
    pub by_concern: [u32; 5],
    pub melted: u32,
    pub residual: u32,
}

impl FunctionCensus {
    #[must_use]
    pub fn count(&self, c: Concern) -> u32 {
        let i = Concern::ALL.iter().position(|x| *x == c).unwrap_or(0);
        self.by_concern[i]
    }

    /// Control rows as a fraction of melted rows.
    ///
    /// The quantity that separates a table from an algorithm: a function that
    /// assigns 131 fields and branches twice is data; one that branches ten
    /// times in twenty-one scopes is behaviour.
    #[must_use]
    pub fn control_density(&self) -> f64 {
        if self.melted == 0 {
            return 0.0;
        }
        f64::from(self.count(Concern::Control)) / f64::from(self.melted)
    }
}

/// The census over all functions.
#[derive(Debug, Clone, Default)]
pub struct Census {
    pub functions: BTreeMap<u32, FunctionCensus>,
}

impl Census {
    /// Functions whose control density is at or below `threshold`.
    ///
    /// This is a MEASUREMENT, not a transcode decision, and it changes nothing
    /// about what melted.
    ///
    /// # Measured result: this does NOT cleanly split data from behaviour
    ///
    /// The hope was a bimodal corpus — tables near 0, algorithms far from it —
    /// so that a threshold would fall in an empty valley. Over all 181 OpenGGS
    /// functions the distribution is **unimodal and smooth**: min 0.000,
    /// p25 0.045, median 0.087, p75 0.134, max 0.255, with the bucket counts
    /// decaying monotonically (58 / 49 / 39 / 30 / 4 / 1 in 0.05-wide bins).
    /// There is no valley, so **any** threshold here is an arbitrary cut and
    /// `data_shaped` must not be read as a classifier.
    ///
    /// The one non-arbitrary criterion is `threshold = 0.0` — functions with no
    /// `Control` row at all, i.e. straight-line bodies. That selects 17 of 181
    /// and is defensible because zero is not a tuned number. It is also NARROW:
    /// `PC_Define`, the corpus's most table-shaped function (131 field
    /// assignments), is NOT among them, because its single file-open error
    /// check contributes two Control rows. A criterion that misses the
    /// canonical positive case is a weak criterion, and saying so is the point
    /// of writing the numbers down.
    #[must_use]
    pub fn data_shaped(&self, threshold: f64) -> Vec<u32> {
        let mut v: Vec<u32> = self
            .functions
            .values()
            .filter(|f| f.melted > 0 && f.control_density() <= threshold)
            .map(|f| f.function)
            .collect();
        v.sort_unstable();
        v
    }
}

/// Derive the address of an event, or name the rail that refused it.
fn address(e: &crate::ore::Event, depth: u32) -> Result<Facet, RailId> {
    let rail = |v: u32, id: RailId| Rail::split(v).ok_or(id);
    let depth_byte = u8::try_from(depth).map_err(|_| RailId::Scope)?;
    Ok(Facet {
        classid: 0,
        rails: [
            rail(e.unit, RailId::Unit)?,
            rail(e.function, RailId::Function)?,
            rail(e.scope, RailId::Scope)?,
            Rail::pair(depth_byte, e.kind.byte()),
            rail(e.seq, RailId::Sequence)?,
            rail(e.symbol, RailId::Symbol)?,
        ],
    })
}

/// Melt the ore under a convention.
pub fn smelt(ore: &Ore, conv: &Convention) -> HarvestReport {
    let mut melted = Vec::new();
    let mut residual = Vec::new();
    let mut census: BTreeMap<u32, FunctionCensus> = BTreeMap::new();

    for fact in &ore.facts {
        let e = match fact {
            OreFact::MalformedRow { ore_seq } => {
                residual.push(ResidualFact {
                    at: None,
                    reason: ResidualReason::MalformedOreRow,
                    ore_seq: *ore_seq,
                });
                continue;
            }
            OreFact::Event(e) => e,
        };

        let entry = census.entry(e.function).or_insert(FunctionCensus {
            function: e.function,
            ..FunctionCensus::default()
        });

        // Gate 2: the scope must be declared before a depth exists.
        let Some(&depth) = ore.scope_depth.get(&(e.function, e.scope)) else {
            entry.residual += 1;
            residual.push(ResidualFact {
                at: None,
                reason: ResidualReason::ScopeNotDeclared,
                ore_seq: e.ore_seq,
            });
            continue;
        };

        // Gate 3 folds into gate 1: the depth byte is part of the address.
        if depth > u32::from(u8::MAX) {
            entry.residual += 1;
            residual.push(ResidualFact {
                at: None,
                reason: ResidualReason::ScopeDepthExceedsRail,
                ore_seq: e.ore_seq,
            });
            continue;
        }

        // Gate 1: every index must fit its rail, or there is no address at all.
        let at = match address(e, depth) {
            Ok(f) => f,
            Err(rail) => {
                entry.residual += 1;
                residual.push(ResidualFact {
                    at: None,
                    reason: ResidualReason::RailOverflow { rail },
                    ore_seq: e.ore_seq,
                });
                continue;
            }
        };

        // Gate 4: the convention must classify the kind.
        if !conv.classifies(e.kind) {
            entry.residual += 1;
            residual.push(ResidualFact {
                at: Some(at),
                reason: ResidualReason::EventKindNotInConvention { kind: e.kind },
                ore_seq: e.ore_seq,
            });
            continue;
        }

        // Gate 5: a row must resolve at some prefix.
        let Some(row) = conv.resolve(&at, e.kind) else {
            entry.residual += 1;
            residual.push(ResidualFact {
                at: Some(at),
                reason: ResidualReason::NoConventionRowAtAddress,
                ore_seq: e.ore_seq,
            });
            continue;
        };

        let i = Concern::ALL
            .iter()
            .position(|c| *c == row.concern)
            .unwrap_or(0);
        entry.by_concern[i] += 1;
        entry.melted += 1;
        melted.push(FlatFact {
            id: FactId(melted.len() as u32),
            at,
            concern: row.concern,
            a: u64::from(e.symbol),
            b: u64::from(e.anchor),
        });
    }

    HarvestReport {
        enumerated: ore.facts.len(),
        melted,
        residual,
        census: Census { functions: census },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::slag::EventKind;

    const EVENTS: &str = "\
/p/A.cpp.f()\t0\tScopeEnter\ts0\t-\tc0\t-\t10\t-\t-\twalk
/p/A.cpp.f()\t1\tWrite\ts5\t-\tc0\t-\t11\t-\t-\tclang
/p/A.cpp.f()\t2\tCast\ts6\t-\tc0\t-\t12\t-\t-\tclang
/p/A.cpp.f()\t3\tRead\ts7\t-\tc9\t-\t13\t-\t-\tclang
";
    const SCOPES: &str = "/p/A.cpp.f()\tc0\t-\tFunction\t0\t0\t9\n";

    fn ore() -> Ore {
        Ore::parse(EVENTS, SCOPES)
    }

    #[test]
    fn conservation_holds_and_nothing_is_dropped() {
        let conv =
            Convention::from_config("classify Write\nrow depth=0 concern=State\n").expect("config");
        let r = smelt(&ore(), &conv);
        assert_eq!(r.enumerated, 4);
        assert!(
            r.conserved(),
            "melted {} + residual {} != 4",
            r.melted.len(),
            r.residual.len()
        );
    }

    #[test]
    fn conservation_holds_for_an_empty_convention_too() {
        // The degenerate end of the ladder: nothing classifies, so everything
        // is residual — and the ledger still balances.
        let conv = Convention::default();
        let r = smelt(&ore(), &conv);
        assert_eq!(r.melted.len(), 0);
        assert!(r.conserved());
    }

    #[test]
    fn an_unclassified_kind_is_named_by_kind_in_the_slag() {
        let conv =
            Convention::from_config("classify Write\nrow depth=0 concern=State\n").expect("config");
        let r = smelt(&ore(), &conv);
        assert!(
            r.residual.iter().any(|x| x.reason
                == ResidualReason::EventKindNotInConvention {
                    kind: EventKind::Cast
                }),
            "the Cast event must be named as its own shape"
        );
    }

    #[test]
    fn an_undeclared_scope_is_residual_with_no_address() {
        // Event 3 sits in scope c9, which scopes.tsv never declares.
        let conv =
            Convention::from_config("classify Read\nrow depth=0 concern=State\n").expect("config");
        let r = smelt(&ore(), &conv);
        let s = r
            .residual
            .iter()
            .find(|x| x.reason == ResidualReason::ScopeNotDeclared)
            .expect("the undeclared scope must be named");
        assert!(s.at.is_none(), "no depth means no derivable address");
    }

    #[test]
    fn classifying_without_a_row_is_a_different_residual_than_not_classifying() {
        // The falsifier for collapsing gates 4 and 5: they name different fixes
        // (add a `classify` line vs add a `row` line) and must stay distinct.
        let conv = Convention::from_config("classify Write\n").expect("config");
        let r = smelt(&ore(), &conv);
        assert!(
            r.residual
                .iter()
                .any(|x| x.reason == ResidualReason::NoConventionRowAtAddress),
            "a classified kind with no row must say so"
        );
        assert!(
            r.residual
                .iter()
                .any(|x| matches!(x.reason, ResidualReason::EventKindNotInConvention { .. }))
        );
    }

    #[test]
    fn adding_a_convention_row_moves_facts_from_slag_to_melt() {
        // The repeat leg, measured: residual falls because the CONVENTION grew,
        // not because a match arm widened.
        let before = smelt(
            &ore(),
            &Convention::from_config("classify Write\nrow depth=0 concern=State\n")
                .expect("config"),
        );
        let after = smelt(
            &ore(),
            &Convention::from_config("classify Write\nclassify Cast\nrow depth=0 concern=State\n")
                .expect("config"),
        );
        assert!(
            after.residual.len() < before.residual.len(),
            "before {} after {}",
            before.residual.len(),
            after.residual.len()
        );
        assert!(after.melted.len() > before.melted.len());
        assert!(before.conserved() && after.conserved());
    }

    #[test]
    fn the_slag_shape_histogram_ranks_by_frequency() {
        let conv = Convention::default();
        let r = smelt(&ore(), &conv);
        let shapes = r.slag_shapes();
        assert!(!shapes.is_empty());
        // Sorted descending by count.
        for w in shapes.windows(2) {
            assert!(w[0].2 >= w[1].2, "shape histogram is not ranked");
        }
    }

    #[test]
    fn control_density_separates_a_table_from_an_algorithm() {
        // Anti-vacuity: the measure must actually discriminate, not return the
        // same number for both shapes.
        let table = FunctionCensus {
            function: 0,
            by_concern: [100, 2, 5, 0, 0],
            melted: 107,
            residual: 0,
        };
        let algo = FunctionCensus {
            function: 1,
            by_concern: [10, 30, 20, 5, 0],
            melted: 65,
            residual: 0,
        };
        assert!(table.control_density() < 0.05);
        assert!(algo.control_density() > 0.4);

        let mut c = Census::default();
        c.functions.insert(0, table);
        c.functions.insert(1, algo);
        // The threshold must BIND: it selects one and not the other, and moving
        // it changes the answer. A knob that selects everything is decoration.
        assert_eq!(c.data_shaped(0.05), vec![0]);
        assert_eq!(
            c.data_shaped(1.0),
            vec![0, 1],
            "a loose threshold admits both"
        );
        assert!(
            c.data_shaped(0.0).is_empty(),
            "a zero threshold admits neither"
        );
    }

    #[test]
    fn a_single_guard_is_enough_to_leave_the_zero_control_set() {
        // Pins the measured narrowness: PC_Define's shape — overwhelmingly
        // assignments, plus one error check — does NOT qualify as zero-control.
        // If a future edit made it qualify, the criterion would have been
        // widened, which is exactly what must not happen silently.
        let mut c = Census::default();
        c.functions.insert(
            0,
            FunctionCensus {
                function: 0,
                by_concern: [131, 2, 5, 5, 0],
                melted: 143,
                residual: 0,
            },
        );
        assert!(c.data_shaped(0.0).is_empty(), "one guard must disqualify");
        assert!(
            !c.data_shaped(0.05).is_empty(),
            "but it is still low-control"
        );
    }

    #[test]
    fn a_function_with_no_melted_rows_is_not_called_data_shaped() {
        // Otherwise every unmelted function would be reported as a table, which
        // is the classic guard-fires-on-everything defect.
        let mut c = Census::default();
        c.functions.insert(
            0,
            FunctionCensus {
                function: 0,
                by_concern: [0; 5],
                melted: 0,
                residual: 40,
            },
        );
        assert!(c.data_shaped(1.0).is_empty());
    }
}
