//! `openggs-furnace` — ore → furnace → slag → repeat, over the OpenGGS C++
//! behavioural harvest.
//!
//! ```text
//!   harvest (ruff_cpp_spo, libclang)   the intake arm — lossless, untouched
//!     -> ore        deterministic typed fact enumeration      (`ore`)
//!     -> furnace    melt into flat, facet-addressed rows      (`furnace`)
//!     -> slag       the addressed residual, named and ranked  (`slag`)
//!     -> repeat     the top slag shape names the next convention row
//! ```
//!
//! # Why this exists
//!
//! Hand-porting C++ function by function got the transcode to 13 of 181 and
//! would have carried on at that rate. It also produced a coverage number no
//! one could check. This crate replaces the hand-curated claim with a
//! **measurement**: the corpus is melted under a convention that lives in a
//! config FILE, and whatever resists is reported at the address where it
//! resisted, grouped by shape, ranked by how often it recurs.
//!
//! The residual is not waste. It is the empirical boundary of the current
//! convention, and it is the work queue.
//!
//! # The rules that keep it honest
//!
//! - **Data as config.** [`convention`] hardcodes no C++ vocabulary. Concerns
//!   are assigned by rows in a file, never by a match arm in Rust. A test reads
//!   that module's own source to enforce it.
//! - **Flat rows.** A [`furnace::FlatFact`] holds no `Vec`, `Box`, map or
//!   `String`. Cardinality means more rows.
//! - **No catch-all residual.** [`slag::ResidualReason`] has no `Other`. A shape
//!   that fits no variant means a variant is added, with before/after counts.
//! - **Conservation.** Every ore fact yields exactly one melted row or exactly
//!   one residual, by construction of the smelt loop's control flow.
//! - **Residual falls only when the convention grows** — never by widening a
//!   match arm to reclassify a shape as handled.

pub mod convention;
pub mod facet;
pub mod furnace;
pub mod ore;
pub mod slag;
