# Stereo SPO x R2IL — accumulated integration plan v1

> **Status:** PROPOSAL. No wave is authorised to write code before its gate below is green.
> **Scope:** the C++ -> Rust transcode arc (OpenGGS as the corpus), the SPO x R2IL
> agreement mechanism, and the CausalEdge64 epistemic wiring it lands on.
> **Grading:** `[G]` = coded or measured in-tree, with the file that proves it.
> `[H]` = grounded hypothesis, mechanism named, not yet measured.
> `[S]` = speculative, no mechanism yet. **Nothing in this plan is unmarked.**
> **Rule:** a `[H]` may not be cited as a reason to build; only its probe may be built.

---

## 0. The thesis

> **⊘ AMENDED 2026-09-11 (A-1).** The original opening read *"A source harvest
> alone cannot grade behaviour … `ruff_cpp_spo` enumerates structure and stops
> at the body."* That was defensible when this arc was viewed through the
> signature plane alone. It is **too strong and is corrected below**, on
> evidence `ruff` PR #118 produced after this plan was written. Struck, not
> deleted: the passive-inventory framing is wrong, and a future session should
> see that it was held and why it failed.

**The source eye can understand and classify behaviour. What it cannot do is
independently warrant or calibrate its own interpretation.**

`ruff` is not a passive inventory. It already carries ordered behavioural ore
(`MethodOre` / `OreEvent` / `OreScope`, present in **both** the cpp and ruby
frontends), scopes and control, reads / writes / raises / calls, symbol roles,
and an actual behavioural classifier (`recipe::classify(&impl BodyFacts) ->
RecipeCentroid`). That is active interpretation, not enumeration.

`ruff_r2il` / `r2sleigh` are therefore **not what makes the arc behavioural**.
They are the second witness that can corroborate, contradict or measure an
interpretation the source eye has already made.

```
   Ruff ACTIVE source understanding
                 x
   R2IL machine-behaviour eye
                 v
          stereo disparity
                 v
       epistemic calibration
```

and NOT:

```
   passive source inventory  x  R2IL  ->  behaviour magically appears
```

The consequence for everything below is a scoping one, not a demolition:
stereo does not create behavioural understanding, it prices it. Every `(f, c)`
in this plan is a **confidence in an existing interpretation**, never the
interpretation itself.

> **⊘ SUPERSEDED by C-4 (2026-09-11).** This paragraph said `(f, c)` is
> "tokenised into" `CausalEdge64` — a one-way promotion, causality finally
> earned. **That is the wrong direction.** See §2.-1: CE64 is the compact wire
> image of the SPOFC relation, encode/decode, not a later causal-only result.
> Struck in place.

`CausalEdge64` is the **compact wire/ABI image of the SPOFC relation itself**:

```
   semantic relation
        v
   S : P : O : f : c
        v  encode
   CausalEdge64
        v  decode
   S : P : O : f : c
```

Its further fields are **lenses over that packed relation** — projection/mask,
inference grade, direction, witness state — not a separate causal payload that
appears only once Pearl causality is established. The epistemic layer that
reasons about the not-knowing is MUL.

**What is new here is the wiring and the probe, not the substrate.** Every
carrier named below already exists. The single most common failure this plan
exists to prevent is building a ninth mechanism next to eight shipped ones.

---

## 1. The ledger — everything this arc discovered, nothing dropped

### 1.1 OpenGGS / OpenGGS-rs (this repo)

| id | finding | grade | where |
|---|---|---|---|
| L-1 | `.lvl` record is **36,732 bytes**, derived twice from opposite ends (`offsetof` on the compiled struct; boundary detection in shipped level data). Independent double-derivation is the data-shape oracle, and it is the one oracle this corpus already has. | `[G]` | `crates/openggs-stagefile/src/lib.rs` |
| L-2 | Corrected a wrong first instinct that the shipped data predated the struct. The struct is authoritative. | `[G]` | `docs/TRANSCODE-LEDGER.md` F-series |
| L-3 | **Coverage is 13/181.** The furnace emits **no Rust**. It places concerns; it does not transcode. | `[G]` | this repo |
| L-4 | **Residual reaching 0 in one pass is a WEAK gate.** With no external witness the furnace only checks itself. | `[G]` | `crates/openggs-furnace/` |
| L-5 | **Control density is unimodal and does NOT split data from behaviour.** min 0.000 / p25 0.045 / median 0.087 / p75 0.134 / max 0.255; buckets 58/49/39/30/4/1. | `[G]` | `ore/pass2-census.md` |
| L-6 | Zero-control selects **17/181** and misses `PC_Define` — so "no control flow" is not a behaviour-free predicate. | `[G]` | same |
| L-7 | The round-trip oracle caught a namespace defect on its **first** run (527 lost / 527 invented) — `ModelGraph::default()` left the namespace empty. Fixed to `ModelGraph::new(NAMESPACE)`. | `[G]` | `ruff` `examples/harvest_openggs.rs` |
| L-8 | Ledger arithmetic was published wrong (State 29,181 vs true 22,207; 22,207+6,974 conflated then double-counted). Corrected by dated storno, not deletion. | `[G]` | `docs/TRANSCODE-LEDGER.md` |
| L-9 | `openggs-furnace`'s hand-rolled facet is **rail-byte-order incompatible** with the canonical `Facet` wire order. It must be deleted, not adapted. | `[G]` | see L-14 |
| L-10 | OpenGGS-rs has **no CI**. Every gate in this plan is currently local-only. | `[G]` | absence of `.github/workflows` |

### 1.2 ruff (the harvest frontends)

| id | finding | grade | where |
|---|---|---|---|
L-11 | `CppFunction` gained `return_type` / `param_types` / `is_static`, plus `walk_free_functions_with_diagnostics` and an error-severity diagnostics gate. | `[G]` | PR #118, head `b13168b4` |
| L-12 | **C file-scope `static` is INTERNAL LINKAGE and must never map to `CppMethod::is_static`.** Two different meanings, one keyword. | `[G]` | `ruff_spo_triplet/src/expand.rs:878` emits only when true |
| L-13 | `recipe::classify(&impl BodyFacts) -> RecipeCentroid` **already classifies bodies** over (writes, reads, raises, calls). My arc re-derived placement without it. | `[G]` | `ruff_spo_triplet/src/recipe.rs` |
| L-14 | `ruff_spo_address::Facet` **already is** the V3 4+12 content-blind facet: `from_parts(facet_classid, part_of, is_a)`, wire order `byte[4+2t] = lo`, `byte[5+2t] = hi`. | `[G]` | `ruff_spo_address/src/lib.rs:79` |
| L-15 | `ruff_python_dto_check::preflight` is a **route-dedup + separation-of-concerns proposer** — the existing pattern for *proposing functions from evidence*, which my arc hand-wrote as `.ggs` config instead. | `[G]` | `preflight/{scanner,mod}.rs` |
| L-16 | SPOFC is `Triple { s, p, o, f, c }` with provenance tiers: Structural (1.0, 1.0), Authoritative (0.95, 0.90), Inferred (0.85, 0.75), OpenProjectExtracted (0.95, 0.88), **CppExtracted (0.95, 0.82)**. The tiers are **constants**, not measurements. | `[G]` | `ruff_spo_triplet/src/triple.rs` |
| L-17 | `openproject-nexgen-rs` contributes `extract_graph_with_schema` / `extract_triples_with_schema` + `SchemaReport` — the SQL-schema-as-config oracle, already generalised past one app. | `[G]` | `crates/ruff_openproject/src/lib.rs` |
| L-18 | `consumer-transcode-furnace-playbook.md` and `fuzzy-recipe-codebook.md` are **MANDATORY** reads per `AGENTS.md`. Both were skipped on first pass. | `[G]` | `ruff/AGENTS.md` |
| L-19 | Two red CI aggregates on #118 were **self-inflicted cancellation cascades** from rapid pushes, not failures. On `5fb47858` every job but `cargo-test-linux` succeeded. | `[G]` | run history |
| L-20 | `prek` cannot run in this sandbox, so Markdown edits ship unchecked and CI's `prek` job is the first to see them. Pipe tables get reformatted. | `[G]` | `ruff/AGENTS.md` |

### 1.3 r2sleigh (the second eye)

| id | finding | grade | where |
|---|---|---|---|
| L-21 | r2il is **strongly-typed P-code**: sized varnodes, 5 address spaces (`Ram`/`Register`/`Unique`/`Const`/`Custom(u32)`), serde-serialisable, SSA-ready, generated from Sleigh specs rather than hand-written per architecture. | `[G]` | `doc/r2il.md` |
| L-22 | **V4 = V3 storage format + behavioural IR properties**, and the "very simple representation" is the **sized varnode** — `(space, size, offset)` fits the V3 rail grammar without widening. | `[H]` | operator statement; the fit is arithmetic, the wiring is unbuilt |
| L-23 | A win32 census probe harness already exists. | `[G]` | `probes/win32-census/EXECUTION-NOTES.md` |
| L-24 | **R2IL's function-discovery rate against SPO's 181 is UNMEASURED.** Every `(f, c)` in this plan is a ratio whose denominator this number is. | `[S]` | — |

### 1.4 lance-graph (the epistemic carriers)

| id | finding | grade | where |
|---|---|---|---|
| L-25 | **Bits 59-60 carry TWO lenses, ordinal-identical on the wire** — `Crystalline ≡ Direct` … `Murky ≡ Unknown` — "which is precisely why the bits cannot disambiguate themselves". `TruthLens::Trust` is the **canonical default**. | `[G]` | `lance-graph-contract/src/band_reading.rs` |
| L-26 | `TRUTH_STATES = 4` is a **compile-time assert**. The planner's 5-variant `TrustTexture` (with `Dissonant`) is **unrepresentable** and "must never be routed through these bits". | `[G]` | same |
| L-27 | There are **four** `TrustTexture` homonyms. The wire one is `causal_edge::layout::TrustTexture` (`Crystalline`/`Solid`/`Fuzzy`/`Murky`) — **not** `contract::mul::TrustTexture` (`Calibrated`/`Overconfident`/`Uncertain`/`Underconfident`), which is the one the phrase "wire into MUL" naturally reaches for. | `[G]` | `band_reading.rs`; `docs/TYPE_DUPLICATION_MAP.md` |
| L-28 | `ReasoningBand` (bits 61-63) has **all 8 ordinals assigned** and `from_bits_3` is total: `Surface`(0) / `Association` / **`Relation`(2)** / **`Causal`(3)** / `Counterfactual` / `Perspective` / `Meta` / `Transcendent`. `BAND_STATES == 8` is asserted. **The mapping is stable; nothing is to be minted there.** | `[G]` | `causal-edge/src/layout.rs:353` |
| L-29 | "relates > causes detection" is the **2 -> 3 promotion**, and the ladder is monotone in epistemic strength. | `[G]` | same |
| L-30 | `with_reasoning_band` is the **only writer**; nothing derives a band. Rewriting the band to `Counterfactual`, or rewriting the topology, breaks the two fields' orthogonality. | `[G]` | `recipe_vocab.rs:63`; `probe_four_plane_causal_medium.rs:17` |
| L-31 | Frozen decision **F5: the band GRADES; the witness reference DISCRIMINATES.** A weak episodic witness and a weak epistemic claim are the same band and different things. | `[G]` | `band_reading.rs` |
| L-32 | `DismechTopology` counts, measured on 2,100 corpus files **before** the module existed: `DIRECT` 9,073 / `INDIRECT_KNOWN_INTERMEDIATES` **3,978** / `INDIRECT_UNKNOWN_INTERMEDIATES` **4,539** / `UNKNOWN` 408. Ordinal 1 is "the hidden-mediator oracle population … so they can be hidden and recovery measured"; ordinal 2 is "the genuine known-unknown population and the epistemic-restraint control". | `[G]` | `lance-graph-contract/src/dismech_evidence.rs` |
| L-33 | Every parse **fails closed**: `UNKNOWN` is a value the corpus asserts (408 rows), so minting it from a parse failure would forge an assertion. | `[G]` | same |
| L-34 | `dismech_evidence` is **source-side only** and deliberately never imports `CausalEdge64` — "the durable causal overlay must not become a pile of hot reasoning registers" (operator ruling). The mapping happens at hydration, in the consumer. | `[G]` | same |
| L-35 | `InferenceType::to_mantissa()` uses **5 of 16** i4 codes: Deduction +1, Induction +2, Abduction −1, Revision +4, Synthesis +5. | `[G]` | `lance-graph-contract/src/nars.rs:64` |
| L-36 | **LATENT DEFECT: `from_mantissa(-8)` silently returns `Deduction`.** `m.unsigned_abs() & 0x7` maps −8 -> 8 & 7 -> 0, indistinguishable from code 0. A producer writing −8 gets a positive assertion with no diagnostic. | `[G]` | `nars.rs:79` |
| L-37 | `Surface = 0` is `#[default]`, so an unstamped band is indistinguishable from an asserted lowest rung. Solved **out-of-band**: `BandPresence::Absent` is the zero-fallback and projection **refuses** — "even ordinal 0 must refuse — a refusal is not Surface(0)". | `[G]` | `band_reading.rs:201,574` |
| L-38 | Declaration lookup is **TOTAL** (hot-path safe, zero-fallback); projection is **FALLIBLE** (a lens mismatch, absent band, or untrusted provenance must fail, never return a plausible value). `BandDeclarations::get` returns `Option` so "explicitly band-free" and "never declared" stay distinguishable to an audit. | `[G]` | same |
| L-39 | The v1 `temporal`-bits trap reaches V3 **transitively** through `CausalEdgeV3::from_v1`, which has **no provenance parameter**. A tainted register is indistinguishable from a clean one; `EdgeProvenance::V3Register` is an **assertion, never an inference**, and `Unknown` refuses. | `[G]` | same |
| L-40 | **`I-NOISE-FLOOR-JIRAK`: classical IID Berry-Esseen is WRONG here.** Bits are weakly dependent by construction (overlapping role-key slices, shared 4096-centroid codebook, bundle accumulation). Any σ-threshold claim must cite Jirak 2016, or state that it is hand-tuned. | `[G]` | `lance-graph/CLAUDE.md` |
| L-41 | **`I-LEGACY-API-FEATURE-GATED`**: a layout reclaim needs a version gate + a field-isolation matrix. Caught **5 times** in Sprint-11; the same function name must never mean two things under two flags. | `[G]` | same |
| L-42 | The **falsifiability rule**: a filter needs an anti-vacuity test; a guard needs **both** a can-fire and a can-stay-silent test on non-trivial inputs; a threshold needs an inertness test. "A guard that fires on everything carries exactly as much information as one that never fires." | `[G]` | same |

### 1.5 The consumer arc — where the ergonomics came from (method only)

The private clinical consumer is cited here for **method only**: no table names,
no column names, no domain concepts, no corpus, no source, no licence terms.

| id | finding | grade |
|---|---|---|
| L-43 | **Two oracles, diverse redundancy: a value-parity witness and a STRUCTURE-parity witness.** The structure witness is a deterministic click-path digest of the legacy app, gated in CI for internal consistency. | `[G]` |
| L-44 | **A write-derived schema beats a declared one.** The committed dump was stale, so the schema was reconstructed from the set of columns the code actually writes — plus every column it reads or soft-deletes but never inserts, because a schema must carry every column the app *touches*. The third arm asserts the stale dump does **not** carry them: that is the anti-vacuity proof the oracle was load-bearing. | `[G]` |
| L-45 | Structure-oracle resolution climbs by adding codebook rows to a config file and re-digesting — **zero code change, data-as-config**. Unresolved tokens are named slag by category, never a catch-all. | `[G]` |
| L-46 | **The askama landing is genuinely trivial once the mask is the currency:** a template that loops columns and cells "never mentions a field name, so it renders ANY `(ClassView, FieldMask)` pair unchanged — adding a class is a new basis + cell accessor, never a new template". | `[G]` |
| L-47 | The dependency arrow was **inverted, not duplicated**: the addressed row (key + positioned fields) is canonical and the string-cell row is its *lowering*. Dropping the mask position at the last moment is what forces behaviour into the cell (an `href`, an `onclick`) — "the T2 trap in HTML clothing". | `[G]` |
| L-48 | **`FieldMask` is a u64 and discards every position >= 64 by documented contract.** No concept in the manifest has >64 fields, so *no test on real data could see it*; the falsifier needs position **133**, plus the second half (`!FieldMask::from_positions(&[133]).has(133)`) that makes "wide" load-bearing rather than decorative. | `[G]` |
| L-49 | `len as u8` wraps at exactly **256** to 0, so a class at the boundary renders **NOTHING** — the worst possible failure for a presence contract, and invisible. Capped by an `ADDRESSABLE_POSITIONS` constant and pinned with a real 256-field fixture. | `[G]` |
| L-50 | **An ordinal is an address in ONE set, never in its own.** An action ordinal indexes the class's *harvested* action set (ordinal 0 was already `Dispose`); a view's own navigation got a separate vocabulary rather than a 35th entry. **A view is not a subtype of what it views.** | `[G]` |
| L-51 | **Direction is `raw -> semantics` and `raw -> display`, never `display -> semantics`.** Recovering a machine identity by string-splitting a localised label inverts the dependency; the signature that only offered display text was the cause, not the implementation. | `[G]` |
| L-52 | **A test that builds the data structure itself does not test its producer.** Five hand-built falsifiers all stayed green while the producer could return an empty field. | `[G]` |
| L-53 | **The SHAPE of a fixture is part of its coverage.** Every real config had an empty mask, so mask and `0..n` coincided and a renumbering was indistinguishable from a mask read — the disable run stayed green until a synthetic non-contiguous mask was parsed through the same path. | `[G]` |
| L-54 | **Three-axis mint gate: `CONCEPT <=> METHOD ∧ STORAGE ∧ STRUCTURE`.** Requiring all three shrank a 5-candidate draft to one clean mint. It fires on **conjunction** and yields a boolean — it is not yet an `(f, c)`. | `[G]` |
| L-55 | Process traps, each measured: **commit before you disable, then checkout** (a restore ate a fix and its test); **a disable that does not apply is indistinguishable from a guard that is not load-bearing** (assert the replacement happened); **`git status` says CLEAN, not CURRENT** (a stale branch manufactures findings that do not exist on main, and one such finding outlived its own rebase); **`git diff A B` is not what a merge does — `A...B` is**. | `[G]` |

### 1.6 Sibling doctrine that constrains this plan

| id | constraint | grade |
|---|---|---|
| L-56 | **a2ui-rs T1/T2/T3:** no second widget vocabulary; **behaviour travels by ADDRESS, never on the surface**; no serialisation in the hot path. RBAC is by **projection** — an unauthorised field is absent from the wire, and a missing mask never falls back to "emit everything". | `[G]` |
| L-57 | **lance-graph-java's `G11` fence was PROSE until a test enforced it** — and the list it asserted was already false, having silently grown by one import. **A named invariant with no test is not an invariant.** | `[G]` |
| L-58 | **The missing-capability STOP rule:** a consumer that needs something the substrate lacks does **not** hand-roll it one layer up. The capability lands substrate-first, or the work stops. | `[G]` |
| L-59 | **OGAR: the Core is shaped first, deliberately** — a generated layer is only as clean as the Core it targets. Never a parallel object model; never an adapter carrying its own state. A Core gap means *extend the Core*, filed and reviewed. | `[G]` |
| L-60 | **The 5+3 council is STRICTLY ordered**: the 5 streamline into a hardened draft FIRST; only then do the 3 attack the hardened object. Attacking unhardened material invites reactive coding. | `[G]` |
| L-61 | **Board hygiene:** a PR that adds a type, plan, deliverable or finding updates the board in the SAME commit. `SUPERSESSION-INDEX.md` is generated and regenerated **LAST**, after the board writes, because the board is one of its inputs. | `[G]` |

### 1.6b Evidence added after this plan was written (2026-09-11, `ruff` #118)

Append-only. These rows are the basis of amendments A-1 through A-5.

| id | finding | grade |
|---|---|---|
| L-64 | **The source eye is ACTIVE, not a passive inventory.** `ruff` already carries ordered behavioural ore (`MethodOre` / `OreEvent` / `OreScope` — present independently in `ruff_cpp_spo::events` AND `ruff_ruby_spo::events`), scopes and control, reads/writes/raises/calls, symbol roles, and a behavioural classifier (`recipe::classify(&impl BodyFacts) -> RecipeCentroid`). This **corrects L-3's framing and this plan's original thesis** (see A-1): what the source eye lacks is not behavioural understanding but the ability to warrant its own interpretation. | `[G]` |
| L-65 | **ACTIVE INTAKE EVIDENCE != SHARED SPO/IR PROJECTION.** `CppFunction::calls` (every callee) and `CppMethod::calls` (the closed mutator set) are semantically incompatible, and the guard forbidding the mapping is architectural evidence, not defensive programming: **ruff knows more about behaviour than one downstream carrier can honestly express.** The two wrong resolutions are overloading the field or minting `dispatches_to` to make types line up. The right one is the open question in §2.0. | `[G]` |
| L-66 | **The corpus cannot witness its own machinery.** Four latent defects landed on #118 in one morning — diagnostics gap, `is_static` overload, overload dedup, namespace collapse — each invisible because OpenGGS has no unresolved includes in the measured run, no relevant file-scope statics, no overloads and no namespaces. **Measured: the dedup drops 0 of 181.** A corpus that cannot exercise a mechanism cannot falsify it. | `[G]` |
| L-67 | **The reviewers kept finding facts ruff KNOWS but its shared projection cannot carry.** Read as a nuisance this is four bug reports; read correctly it is repeated, independent evidence for why an intake layer distinct from the projection exists at all. This is the strongest result of the #118 arc and it was accidental. | `[H]` |

### 1.6c Operator corrections, 2026-09-11 (basis of amendments C-1..C-7)

| id | finding | grade |
|---|---|---|
| L-68 | **V3 is storage; V4 is IR. The code-graph is a PHASE, not a layer:** during discovery the IR IS a code-graph; after discovery IR/AR belongs to compile. **OGAR is dumb AR + adapters** — it mints and adapts, never reasons. **Reasoning is always in lance-graph; consumers reuse.** | `[G]` operator |
| L-69 | **`CausalEdge64` is the compact wire image of the SPOFC relation**, encode/decode — not a causal-only result that appears once Pearl causality is earned. Its further fields are lenses over the packed relation. **This supersedes the plan's original "tokenised into" framing.** | `[G]` operator |
| L-70 | **Four axes, not five bits of causality.** Semantic `P` (`CAUSES`/`SUPPORTS`) = what the relation MEANS; bits 59–60 = how it is mediated; bits 61–63 = what reasoning it entitles; plus the SPO projection. *`P = CAUSES` asserts meaning; `band = Causal` grants permission to reason causally.* `P = CAUSES` ∧ `topology = IndirectUnknown` is legitimate and rich. | `[G]` operator |
| L-71 | **"Tarski" is already taken, with a different meaning.** `Belief::rung` (`lance-graph-planner/src/nars/belief.rs:96`) is a **derivation depth** — 0 observed, derived = `max(premise rungs)+1`, fixed at creation, revision does not change it. Metalanguage level, NOT admissibility. Calling bits 61–63 "Tarski" would collide; call them the **reasoning-grade / admissibility lattice**. The Tarski rung is a fifth, independent axis. Separately, `dismech_evidence::Supports` (`SUPPORT`/`PARTIAL`/`REFUTE`/`NO_EVIDENCE`, ~89,800 occurrences) is an **evidence stance**, not a predicate — do not alias it to a semantic `P = SUPPORTS`. | `[G]` verified in-tree |
| L-72 | **R2IL is EXECUTED, never pre-converted** (operator ruling 2026-08-26, `ogar-r2il/src/lib.rs`), and `ogar-r2il` deliberately carries **no `r2sleigh` dependency**. Its `project(&slab, shape, &mask)` already provides the same-slab two-readings mechanism: one body's 360 content bytes hold 180/120/90 calls depending on `LaneShape`, masked and lazy, *"a shape is a lens, not a migration"*. **The stereo comparison's mechanism is already shipped.** | `[G]` |

### 1.7 The process finding about this arc itself

| id | finding | grade |
|---|---|---|
| L-62 | **Five instances of one pattern in my own arc:** a working, already-measured mechanism sat one or two files from where the arc stopped reading, and new machinery got built to answer a question that mechanism had already priced. The five: (1) `ruff_spo_address::Facet` re-derived incompatibly (L-9/L-14); (2) `recipe::classify` unused (L-13); (3) `fuzzy-recipe-codebook.md` unread (L-18); (4) `ConceptConvention` re-derived; (5) `preflight` already proposes config from evidence where I hand-wrote it (L-15). | `[G]` |
| L-63 | **The mitigation is mechanical, not remembered:** every wave below names the *existing* symbol it must consume, and a wave that introduces a new type must say which existing symbol it rejected and why. A wave with no "rejected alternative" line is unreviewed. | `[H]` |

---

## 2. The architecture

### 2.-1 The model (C-1..C-5, operator-corrected 2026-09-11)

**One line:** *lance-graph **is** the CodeGraph; SPOFC is its relational algebra;
`CausalEdge64` is the compact wire image of that algebra; R2IL is another native
behavioural plane, not something that must first be converted into SPO.*

| pin | statement |
|---|---|
| **C-1** | **V3 is storage. V4 is IR.** |
| **C-2** | The code-graph is a **PHASE, not a layer.** *During* discovery the IR **is** a code-graph. *After* discovery, IR/AR belongs to **compile**. |
| **C-3** | **OGAR is dumb AR + adapters.** It mints and adapts; it never reasons. **Reasoning is always in lance-graph, and consumers reuse it.** |
| **C-4** | **CE64 ⇄ SPOFC**, encode/decode — never "reasoning → causality finally earned → CE64". |
| **C-5** | Bits **59–60 say how the relation is mediated**; bits **61–63 say how strongly the system is entitled to reason from it**. **`CAUSES`/`SUPPORTS` say what the relation MEANS** — they are semantic `P`, never positional bit meanings. |

```
   Ruff ore            R2IL
      │                  │
      └──────┬─────────┘
             v
        lance-graph
         CodeGraph
             │
       reason in SoA
             │
             v
      SPOFC relations
             ⇅
       CausalEdge64
         wire form
```

**Why this corrects an earlier hedge of mine.** §2.0 below and my session
reasoning said *"lance-graph does not become the code-graph; the code-graph is a
tenant layout lance-graph queries"* — hedging toward the assembler-vs-storage
fence. That fence is about **minting**, not **reasoning**. Reasoning is
lance-graph's by rule (C-3), so the code-graph **is** there. OGAR still mints and
adapts. The two were never in tension and I treated them as if they were.

**Consequence for SPOFC:** without lance-graph, SPOFC is a mostly mythical tuple
— the interesting `f`/`c`, witness accumulation, masks, history and cross-plane
reasoning have no machinery behind them. Inside lance-graph, the R2IL plane, the
SPO plane, the `f`/`c` evidence, the CE64 wire and the temporal/witness planes
are all operable on **one SoA substrate**.

### 2.-1a The four axes (C-5) — and the Tarski collision to avoid

Five bits are **not** "five bits of causality". They are two orthogonal
epistemic controls, and neither is the relation's meaning:

| axis | carrier | question |
|---|---|---|
| **semantic `P`** | the predicate | `CAUSES` / `SUPPORTS` / `ENABLES` — **what does the relation MEAN?** |
| SPO projection | `S`/`P`/`O` subset | which Pearl vertices |
| **mediation witness** | **bits 59–60** | `Direct` / `IndirectKnown` / `IndirectUnknown` / `Unknown` — **how is it mediated?** |
| **reasoning entitlement** | **bits 61–63** | `Surface`→`Association`→`Relation`→`Causal`→`Counterfactual`→`Perspective`→`Meta`→`Transcendent` — **what inference regime may treat it as admissible?** |

**The separation that kills a whole bug class:**

> `P = CAUSES` is an **assertion about meaning**.
> `band = Causal` is **permission to reason causally from it**.

So `Relation → Causal` is never "the predicate changed from `SUPPORTS` to
`CAUSES`" — it is "the accumulated evidence crossed the threshold at which
causal reasoning is permitted". This is what forbids the old failure of
reinterpreting a semantic `CAUSES` bit as a `CausalMask` bit.

**`P = CAUSES` with `topology = IndirectUnknown` is a legitimate, rich state:**
*enough evidence for a causal relation, and enough to know it is mediated, but
not enough to name the mediator.* Far richer than a boolean causal flag — and
it is why B-4's correction (unknown is not false) was necessary rather than
merely careful.

> **⚠ Do NOT call bits 61–63 the "Tarski rung".** Verified in-tree
> 2026-09-11: `lance_graph_planner::nars::belief::Belief::rung` **already owns
> that name with a different meaning** — a derivation depth (*"0 observed;
> derived = `max(premise rungs)+1`, fixed at creation — revision does NOT
> change it"*), i.e. how many inference steps from observation. That is
> **metalanguage level**, not admissibility. Reusing the word would be the
> "same word, two contracts" trap that `ogar-r2il` already refuses between
> loco's core `ADD` and R2IL's `IntAdd`. Call 61–63 the **reasoning-grade /
> admissibility lattice**; the Tarski rung is a **fifth, independent axis**.

> **⚠ `SUPPORTS` is also already overloaded.**
> `dismech_evidence::Supports` is a 4-state **evidence stance**
> (`SUPPORT`/`PARTIAL`/`REFUTE`/`NO_EVIDENCE`, ~89,800 measured occurrences) —
> closer to `f`/`c` polarity than to a predicate. A semantic `P = SUPPORTS` and
> an evidence stance `Supports::Support` are different things; do not alias.

### 2.0 The Active Code-Graph — the DISCOVERY-PHASE reading (A-2, re-framed by C-2)

> **⊘ Re-framed 2026-09-11.** A-2 called this "a latent layer to NAME". It is
> **not a layer** — it is the **discovery-phase reading of the IR** (C-2). The
> `CppFunction::calls` evidence below stands unchanged; only its framing moves.

`ruff` PR #118 produced the architectural evidence for a layer this plan had
left unnamed. `CppFunction::calls` holds every-callee behavioural intake
evidence. `CppMethod::calls` means something strictly narrower — the closed
mutator/dispatch vocabulary. The guard that now forbids mapping one onto the
other is **not merely defensive programming**; it is a boundary:

```
   ACTIVE INTAKE EVIDENCE   !=   SHARED SPO / IR PROJECTION
```

**Ruff may know more about behaviour than one downstream carrier can honestly
express.** The two wrong ways to resolve that are (a) overloading
`CppMethod::calls`, and (b) minting `dispatches_to` merely to make the types
line up. Both are the shape this arm has already rejected twice.

The right resolution is to record the proof obligation:

> **Where does full behavioural intake live, BEFORE lossy projections such as
> `ModelGraph` / SPOFC?**

So the pipeline gains one *conceptual* stage:

```
   source AST
        v
   ordered ore / calls / scopes / symbols / recipes
        v
   ACTIVE CODE-GRAPH
        v                    v
   SPOFC projection     R2IL stereo comparison
```

**Do NOT begin by creating `struct CodeGraph`.** The first task is to determine
whether the Code-Graph already IS the composition of pieces that exist today —
verified present in-tree, 2026-09-11:

| piece | where | `[G]` |
|---|---|---|
| `MethodOre` / `OreEvent` / `OreScope` | `ruff_cpp_spo::events`, and independently `ruff_ruby_spo::events` | ordered behavioural ore, already per-frontend plural |
| full `CppFunction::calls` | `ruff_cpp_spo::lib` | every-callee intake, currently reaching no triple |
| `BodyFacts` (trait) + `RecipeCentroid` | `ruff_spo_triplet::recipe` | the behavioural classifier |
| `BodyFacts` (struct) | `ruff_python_dto_check::extractors::body` | the python arm's own body evidence |
| structural `Facet` / address | `ruff_spo_address` | the V3 4+12 identity |

**A carrier is added ONLY if a behavioural relation the stereo comparison
requires cannot be expressed by those pieces.** That is a finding to reach, not
a design to start from — and this plan's own L-62 records five occasions where
this arc built a mechanism that already existed one file away.

### 2.1 The five stages

```
  C++ eye                                  binary eye
  ruff_cpp_spo (libclang)                  r2sleigh-lift (Sleigh -> r2il)
  structure: 181 fns, signatures           behaviour: typed varnode dataflow
        |                                        |
        +------------------ JOIN by symbol ------+
                           |
                  DISPARITY  ->  (f, c)          <- the measurement (S1)
                           |
              +------------+------------+
              |            |            |
       i4 mantissa    bits 59-60    bits 61-63
       (46-49)        2-bit truth   ReasoningBand
       rule +         mediator      Relation -> Causal
       direction      structure     promotion
       +-0 = NaN      (lens-        (STABLE — read
       -8 = zero       declared)     only, never mint)
              |            |            |
              +------------+------------+
                           |
                    MUL (DkPosition / TrustTexture /
                    compass / homeostasis / gate)
                    derives Dissonant; never a 5th ordinal
```

**Stage responsibilities, and the line each must not cross:**

| stage | carrier | must NOT |
|---|---|---|
| S1 evidence | stereo join | invent a third eye; claim a rate before the join is measured |
| S2 rule | i4 mantissa, bits 46-49 | reuse an assigned code; change `from_mantissa` without a version gate |
| S3 structure | 2-bit truth, bits 59-60 | route a 5-variant enum through 2 bits; read a lens that was not declared |
| S4 grade | `ReasoningBand`, bits 61-63 | mint an ordinal; derive a band anywhere but `with_reasoning_band`; rewrite the topology when promoting |
| S5 policy | MUL | store `Dissonant`; treat the wire `TrustTexture` and MUL's homonym as one type |

**The ordinal alias table (S3), from the code, not designed here:**

| ord | `Trust` lens (default) — disposition | `Topology` lens | corpus |
|---|---|---|---|
| 0 | `Crystalline` — proceed | `DIRECT` | 9,073 |
| 1 | `Solid` — proceed | `INDIRECT_KNOWN_INTERMEDIATES` | 3,978 |
| 2 | `Fuzzy` — **sandbox** | `INDIRECT_UNKNOWN_INTERMEDIATES` | 4,539 |
| 3 | `Murky` — **compass (veto)** | `UNKNOWN` (asserted) | 408 |

Ordinals 2 and 3 align in disposition **and** in epistemic status, which is
exactly why the misread is dangerous rather than obvious. **Ordinal 1 is the
break:** `Solid` (calibrated, proceed) and "a mediator exists and is nameable"
are not the same claim, and 3,978 rows hang on it. `[G]` for the table, `[H]`
for the claim that the alignment is load-bearing rather than coincidental — that
is what F-8 tests.

### 2.2 Endgame orientation (A-5, added 2026-09-11) — context, not scope

**The long-term objective is software-as-harvest: `ruff` and the other intake
arms reconstruct active behaviour from legacy software; `lance-graph` reasons
over that behaviour; sufficiently warranted reusable semantics may be canonised
as OGAR Active Records and lowered through `ogar-loco` as callable microcode.
The current stereo probe measures ONE PREREQUISITE of that path; it does not
claim the promotion path is already built.**

```
   stone-age software
        v  harvest
   Ruff ACTIVE Code-Graph
        v
   lance-graph  (active semantic / relational / epistemic reasoning)
        v
   stable reusable behaviour
        v
   OGAR  (Open Graph of Active Record — canonical active semantics)
        v
   ogar-loco  (microcode / orchestration)
```

Four fences on that diagram, so it is read as orientation and not as a mandate:

1. **`Lance` is not `lance-graph`.** Lance is the storage engine (upstream-
   authoritative, never forked); `lance-graph` is the spine. Conflating them is
   a category error this workspace has already ruled on.
2. **The arms stay PLURAL.** Ruff's Code-Graph, R2IL / `ogar-r2il`, SPOG, RO,
   DeepNSM / CAM-PQ, ARM and the rest are *eyes and dogfood* for `lance-graph`
   — never candidates for one universal physical encoding.
3. **The common waist is a HYPOTHESIS, not a carrier.** That `S : P : O` ~
   `P(S, O)` ~ `Fn_P(S, O)` is a thing to TEST. Nothing in this plan mints it.
4. **The OGAR/SurrealQL lesson is about provenance, not absence.** OGAR always
   had a behavioural IR — it is the Open Graph of *Active Record*. What that
   episode taught was **not** "behaviour should disappear" but "behaviour must
   not be interpreted out of storage / DDL". The modern reading to test is
   `harvested behaviour -> active Code-Graph understanding -> lance-graph
   warrant / recurrence / semantics -> canonical active P in OGAR -> Fn_P(...)
   -> compiled execution`. Historical OGAR docs are **evidence for continuity,
   not an implementation template.**

**This section is orientation. This PR does not implement any of it**, and a
session that treats §2.2 as a work list has misread it.

---

## 3. Waves

Each wave names the existing symbol it consumes and, where it adds anything,
the alternative it rejected (L-63).

### W0 — Make the ground measurable (no new mechanism)

| D-id | deliverable | consumes | gate |
|---|---|---|---|
| D-ST-0a | **CI on this repo.** `cargo test --workspace`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`. Pinned toolchain via `rust-toolchain.toml`. | — | every later gate is local-only without it (L-10) |
| D-ST-0b | **Delete `openggs-furnace`'s hand-rolled facet; consume `ruff_spo_address::Facet`.** Rejected alternative: adapting the local facet — its rail byte order is incompatible, so adapting would preserve a second wire grammar (L-9/L-14). | `ruff_spo_address::Facet::from_parts` | F-1 |
| D-ST-0c | **Replace hand-written placement with `recipe::classify`.** Rejected alternative: keeping the `.ggs` convention files as the placement authority — `classify` already does this over (writes, reads, raises, calls) (L-13). | `ruff_spo_triplet::recipe::classify` | F-2 |
| D-ST-0d | Re-run the census with `classify` and publish the **new** residual, with the L-4 caveat restated: a residual is not a gate without a second witness. | D-ST-0c | F-3 |

### W1 — The stereo join (THE gate for everything downstream)

| D-id | deliverable | consumes | gate |
|---|---|---|---|
| D-ST-1a | **`PROBE-STEREO-JOIN`**: lift the compiled OpenGGS binary through `r2sleigh-lift`, join to the 181 harvested `CppFunction`s by symbol, and report **four raw numbers**: functions r2il finds; of those, how many join to a harvested symbol; of those, how many agree on arity; of those, how many have a single-entry / single-exit varnode chain. | `r2sleigh-lift`, `ruff_cpp_spo` PR #118 signatures | J1 |
| D-ST-1b | Publish the **non-join residual by category** (inlined, static-folded, renamed by the linker, absent from the binary, present but unsymbolised). Named slag, no catch-all (L-45). | D-ST-1a | F-4 |
| D-ST-1c | ~~The 13 already-transcoded functions are the held-out calibration set~~ — **WITHDRAWN (B-1, 2026-09-11).** They are NOT ground truth: `docs/TRANSCODE-LEDGER.md` §"Parity is UNMEASURED" states *"Nothing here has been run against the original binary"* and that the tests assert only the transcode's INTERNAL invariants. Scoring a disparity metric against them would be circular, and they are additionally a biased sample — hand-picked for being mechanically tractable, which is the very property the metric is meant to predict. **Blocked on the ledger's own named falsifier** (instrument the C++ to dump per-frame state for a scripted input, run the same script through `Sim::step`, diff). Until that runs, the metric has NO calibration set and must say so. | `TRANSCODE-LEDGER` §Parity | F-5 |
| D-ST-1d | **`PROBE-SHAPE-FIRES` (A-3).** Run intake + join against a corpus or fixture that genuinely contains C++ namespace and overload structure, and assert the mechanism FIRES there. Does **not** replace OpenGGS as the measured population — it exists to prove the machinery is not inert outside the easiest case (measured: dedup drops 0 of 181 on OpenGGS). | `ruff_cpp_spo`, `ruff_spo_address::Facet` | F-19 |

> **Nothing in W2+ may be built before J1 is green.** Every `(f, c)` downstream
> is a ratio whose denominator D-ST-1a produces (L-24).

### W2 — The disparity metric

| D-id | deliverable | consumes | gate |
|---|---|---|---|
| D-ST-2a | **`f` = per-fact agreement rate** across the joined population. A fact is a `(subject, predicate, object)` the C++ eye asserts; agreement is the binary eye's dataflow confirming it — **computed against EXECUTED or lensed R2IL, never against R2IL lowered into triples** (C-6). | `ruff_spo_triplet::Triple` | F-6, F-23 |
| D-ST-2b | **`c` = accounted-for margin** — the fraction of the function's lifted operations the source-side facts explain. High `c` + low ops = mechanically recoverable; low `c` = the essential-15% hand port, **measured** rather than inferred from control density (L-5/L-6). | D-ST-2a | F-6, F-7 |
| D-ST-2c | A **new SPOFC provenance tier** whose `(f, c)` is *computed from D-ST-2a/2b* rather than pinned as a constant. Rejected alternative: reusing `CppExtracted (0.95, 0.82)` — those are constants and would silently launder a measurement into a literal (L-16). | `ruff_spo_triplet` provenance | F-6 |

**A-4 ordering constraint (2026-09-11) — raw statistics FIRST, `(f, c)` last.**
The existing SPOFC provenance tiers are constants, and a stereo similarity score
is not a NARS confidence. Relabelling one as the other would be the same class
of error as the `PARITY` marker and the `roundtrip_eq` claim already corrected
on `ruff` #118: an artifact asserting something about its own verification that
it has not earned.

So D-ST-2a/2b/2c do **not** emit an `(f, c)` on first landing. They emit and
keep **separate raw statistics**:

- joined population
- corroborated relations
- contradicted relations
- behavioural coverage
- non-join residual (categorised, per D-ST-1b)

`(f, c)` is derived only once the plan states, in writing, **which of those are
the sufficient statistics and what falsifies the derivation**. Until then the
raw five are the deliverable and the tier stays unminted.

F-7's pre-registered correlation threshold is load-bearing precisely because the
previous behavioural proxy already failed: control density was unimodal and did
not separate data from behaviour (L-5/L-6). A second proxy that is never tested
against that failure would be the same mistake with a new name.

### W3 — The epistemic wiring (substrate-side, gated)

| D-id | deliverable | consumes | gate |
|---|---|---|---|
| D-ST-3a | **i4 `0` = not-asserted / `-8` = exact zero**, behind a version gate with a field-isolation matrix, per `I-LEGACY-API-FEATURE-GATED` (L-41). Its red-then-green falsifier is L-36: `-8` currently aliases to `Deduction` silently. Justified as **consistency with D-ACR-7's refuse-on-unstated discipline** (L-37/L-38), not as free-code opportunism. | `nars.rs::{to_mantissa, from_mantissa}` | F-9, J2 |
| D-ST-3b | **Declare the truth lens per `(classid, rail)`** for every stereo producer. Rejected alternative: inferring the lens from the data — the contract states it is unrecoverable from the bits (L-25), and `Trust` is the default, so an undeclared topology producer is misread *plausibly*. | `BandDeclarations`, `TruthLens` | F-8 |
| D-ST-3c | **Label aliases live in the declaration layer**, naming the same ordinal per lens. The bits stay 2. Rejected alternative: a 5th ordinal for `Dissonant` — `TRUTH_STATES = 4` is a compile-time assert and the planner's 5-variant enum is explicitly forbidden on these bits (L-26/L-27). | same | F-8 |
| D-ST-3d | **MUL derives `Dissonant`** from (mantissa `0` vs non-zero) x (stereo sign agreement): mantissa `0` -> genuine unknown -> `Fuzzy`/sandbox; non-zero with the two eyes disagreeing in chain direction -> asserted contradiction -> `Murky`/veto. Nothing is stored. | `contract::mul`, D-ST-3a | F-10 |

### W4 — The promotion learner (`Relation` -> `Causal`)

| D-id | deliverable | consumes | gate |
|---|---|---|---|
| D-ST-4a | **Read-only band consumption first.** Score the existing `ReasoningBand` distribution over the joined corpus before writing a single band. Rejected alternative: minting a band ordinal — all 8 are assigned and the mapping is stable (L-28). | `reasoning_band()` | F-11 |
| D-ST-4b | **The 2 -> 3 promotion rule**, calibrated on the ordinal-1 labelled population (hide the mediator, score recovery) with the ordinal-2 population as the **hallucination control** (any recovery there is a false positive by construction) (L-32). | `DismechTopology`, D-ST-2a | F-11, F-12, J3 |
| D-ST-4c | Thresholds either cite a **Jirak-derived bound** or state in-source that they are hand-tuned. Rejected alternative: an IID Berry-Esseen significance claim — forbidden by `I-NOISE-FLOOR-JIRAK` (L-40). | — | F-13 |
| D-ST-4d | Writes go through **`with_reasoning_band` only**, and a promotion **never** rewrites the topology field (L-30). | `with_reasoning_band` | F-14 |

### W5 — The literature gate (blocking for W4's method, not for W4's plumbing)

| D-id | deliverable | gate |
|---|---|---|
| D-ST-5a | Ground the causality-learning proposal in primary literature before it shapes D-ST-4b's rule: report what is **proven** vs **suggested** vs **contradicted**, graded. Currently `[S]` — an unnamed proposal recalled from memory must not shape a promoter. | F-15 |

### W6 — Close the loop back to coverage

| D-id | deliverable | gate |
|---|---|---|
| D-ST-6a | Use `preflight`'s separation-of-concerns proposer to **propose Rust function boundaries** from the joined evidence, rather than hand-authored config. Rejected alternative: more `.ggs` files (L-15). | F-16 |
| D-ST-6b | Report coverage honestly against 181, separating **structurally placed** from **behaviourally transcoded**. The 85/15 split stands until D-ST-2b measures otherwise. | F-17 |

---

## 4. Falsification tests

Every test below states **what input makes it fail**. A test with no such input
is deleted, per the falsifiability rule (L-42).

| id | test | fails when | anti-vacuity arm |
|---|---|---|---|
| F-1 | A facet minted by this repo is byte-identical to `Facet::from_parts` for the same `(classid, part_of, is_a)`. | any rail byte is swapped | assert the two rails **differ** in the fixture, so a lo/hi swap is detectable at all |
| F-2 | **Dependency, not divergence (B-2).** Perturb or disable `classify` and assert the placement output CHANGES; restore and assert it returns. Divergences against the retired hand placement are still listed with their cause, but agreement is no longer treated as failure. | the output is identical with `classify` perturbed — then it is not wired in | the disable run must be red-then-green. **The original arm was wrong**: it demanded a semantic difference, so a correctly-wired classifier that legitimately agrees on all 181 would have failed it. Output equality never proved non-consultation |
| F-3 | The published residual equals a freshly recomputed one. | the ledger drifts from the census | the census is re-derived, not re-read from the ledger |
| F-4 | Every non-joining function lands in exactly one named category; the categories sum to the non-join count. | a function is uncategorised, or a catch-all appears | assert **no** category holds >90% of the residual (a single bucket is a catch-all with a name) |
| F-5 | **BLOCKED (B-1).** Cannot run until behavioural parity for the 13 is independently established, or an independently labelled calibration set exists. | — | the honest state is *no calibration set*; a metric reported without one carries no accuracy claim at all |
| F-6 | `f` and `c` change when a fact is removed from the C++ side, and change when an operation is removed from the lifted side. | either side is inert — the metric reads only one eye | both directions, separately |
| F-7 | **`c` is NOT a restatement of control density.** Correlate the two across 181; report the coefficient. | `c` correlates above a pre-registered threshold — then it is L-5's failed predicate wearing a new name | pre-register the threshold **before** computing |
| F-8 | Declaring `TruthLens::Topology` vs `Trust` on **ordinal 1** routes to a different MUL disposition, and both arms are non-trivially populated. | the declaration changes no decision — the alias table is decorative | both arms populated; the test cannot pass on an empty arm |
| F-9 | `from_mantissa(-8)` and `from_mantissa(0)` return **different** things under the gate, and the un-gated path is unchanged. Field-isolation matrix: writing the mantissa leaves every other field bit-identical. | `-8` still aliases to `0`, or another field moves | the pre-gate run **must** be red (disable-verified) |
| F-10 | `Dissonant` fires on a constructed sign-disagreement and does **not** fire on an agreement — both on non-trivial inputs. | it fires on everything, or never | both arms; an empty-input silence case does **not** count (L-42) |
| F-11 | The promoter's `Causal` set is non-trivial in both directions: non-empty, and `promoted * 3 < total`. | it promotes everything or nothing | both bounds asserted |
| F-12 | **Epistemic restraint, NOT unknown-as-false (B-4).** On the ordinal-2 population the promoter must MARK its output as unwarranted (abstain / carry `Fuzzy`-sandbox), rather than promote to `Causal` as though warranted. What is tested is whether it knows it lacks a warrant — never whether a recovered mediator is wrong. | it promotes ordinal-2 rows to `Causal` at the same rate as ordinal-1, i.e. the distinction has no effect on its behaviour | the ordinal-1 arm must show promotion, or the test proves only that the promoter is inert. **A genuine negative control needs independently labelled absent/incorrect-mediator cases and this corpus does not supply them** |
| F-13 | Raising the threshold silences something; lowering it admits something. | the knob is decoration | both directions, with the counts |
| F-14 | A promotion leaves bits 59-60 bit-identical. | the topology moved | assert the band **did** change, so the test is not passing on a no-op |
| F-15 | Every literature claim carries a grade and a citation; no `[S]` claim appears in a design decision. | an ungraded claim shapes the promoter | — |
| F-16 | A proposed function boundary is rejected when the evidence is removed. | the proposer emits the same boundaries from no evidence | run it against a stripped corpus and assert the output differs |
| F-17 | Coverage reports structural and behavioural counts **separately**, and the behavioural count matches the number of functions with an actual Rust body. | the two are summed into one number | a function counted behavioural with no body fails |
| F-18 | **The invariant-has-a-test check.** Every "must not" in §2's table has a named test in this table. | a prose invariant has no test — the `G11` failure mode (L-57) | this table is the test's index; a missing row is the failure. **F-18 failed its own check on first review (B-5, 2026-09-11)** — two §2 invariants had no row; F-21 and F-22 below are those rows |
| F-21 | **An UNDECLARED lens refuses (B-5).** Projecting the 2-bit field for a class with no `BandDeclarations` entry returns a `BandReadError`, never a plausible ordinal. | it silently yields the `Trust` default — the D-ACR-7 failure mode | pair with a declared class that projects successfully, so the test is not passing on universal refusal |
| F-22 | **`Dissonant` is derived, never stored; the `TrustTexture`s stay distinct (B-5).** No write path persists a 5th truth ordinal, and the wire enum (`Crystalline`/`Solid`/`Fuzzy`/`Murky`) is never substituted for `contract::mul`'s (`Calibrated`/`Overconfident`/`Uncertain`/`Underconfident`). | a `Dissonant` reaches a stored field, or the two 4-variant homonyms are used interchangeably | assert `TRUTH_STATES == 4` still holds and that a derived `Dissonant` is observable in MUL output while absent from every persisted row |
| F-19 | **The shape check fires (A-3).** On the namespace/overload corpus, the dedup discriminates non-trivially AND at least one identity is distinguished only by scope. | the mechanism is inert there too — then it is untested everywhere, not just on OpenGGS | pair with the OpenGGS run, where the same code drops 0 of 181; the two runs must DIFFER |
| F-23 | **No R2IL→SPO lowering exists on the comparison path (C-6).** Grep the stereo code for any construction of a `Triple` (or SPOFC row) from an r2il/varnode value. | such a construction exists — the forbidden conversion is on the path | assert the comparison DOES read r2il (executed or lensed), so the test is not passing because nothing reads it at all |
| F-20 | **`(f, c)` is not emitted before its statistics are stated (A-4).** No provenance tier carrying a computed `(f, c)` exists while the sufficient statistics and their falsifier are unwritten. | a stereo score ships relabelled as NARS confidence | the five raw statistics are present and separately reported |

### 4.1 Disable-run discipline (mandatory, from L-55)

1. **Commit first.** The restore step is `git checkout <file>`, which reverts to the last commit, not to the pre-disable working tree.
2. **Assert the disable applied** — `assert s.count(old) == 1, "anchor moved"`. A patch that no-ops looks exactly like a guard that is not load-bearing.
3. **Verify currency before the first measurement** — `git fetch origin main` and check the merge base. CLEAN is not CURRENT, and a finding drawn on a stale branch has already outlived a rebase once.
4. **Re-run the whole module** after a behaviour change, not only the new tests.

---

## 5. The kill joints

A joint is where the plan **stops** if the number comes back wrong. Each names
what dies, so a bad result is a result and not a re-scope.

### J1 — Does the second eye see the population? (gates all of W2+)

- **Green:** r2il joins a majority of the 181 with arity agreement.
- **Amber — a SCOPED PASS, and it does cross the gate (B-3, 2026-09-11).** It joins a minority. W2+ MAY proceed, restricted to the joined subset, provided every downstream claim names that subset and its size explicitly and coverage for the rest stays at zero. *(The original text said the metric "survives" while W1's gate admitted only a majority-green, so amber had no executable path — it described an outcome the plan then forbade. Amber is a pass with a stated denominator, not a stall.)*
- **KILL:** it joins almost nothing (inlining, no symbols). Then there is no second eye on this corpus, the stereo thesis is **false for OpenGGS**, and the honest outcome is: the data-shape oracle (L-1) stands, the structure oracle does not exist here, and the transcode is a hand port with SPO doing placement only. **Say that and stop** — do not substitute an execution-diff harness under the same D-ids, which would be a different plan wearing this one's name.

**A-3 scoping (2026-09-11) — J1 is not weakened, it is bounded.** Whatever it
returns, *"J1 proves or kills the stereo thesis for OpenGGS, not for arbitrary
C++."* OpenGGS stays the known population and its denominator (181) is not to be
changed or supplemented with an invented rate.

The reason for the bound is measured, not cautious. `ruff` PR #118 accumulated
**four** latent correctness defects in one morning — the diagnostics gap, the
`is_static` overload, the overload dedup, and the namespace collapse — and every
one was invisible on this corpus because its shape is unusually simple:

| shape the corpus lacks | what stayed untested |
|---|---|
| unresolved includes in the measured run | the error-diagnostic skip |
| file-scope `static` cases | the linkage/staticness distinction |
| overloads | the `(name, params)` key |
| namespaces | the qualified-name identity |

A corpus that cannot exercise a mechanism cannot witness its failure, so a green
J1 on OpenGGS alone would carry less information than its number suggests.

**Therefore J1 is PAIRED with an anti-vacuity SHAPE check (`D-ST-1d`):** run the
intake and join against a corpus or fixture that actually contains C++ namespace
and overload structure, and assert the mechanism *fires* there — a non-zero
dedup discrimination and at least one scope-distinguished identity. Measured
2026-09-11 on OpenGGS: the dedup drops **0 of 181**, so on this corpus that
machinery is inert and unwitnessed.

This does **not** replace OpenGGS. The second shape exists only to prove the
intake/join mechanism can fire outside the easiest case; the measured population
remains the first one.

### J2 — Is the i4 change safe? (gates W3d, W4)

- **Green:** the field-isolation matrix is clean and the pre-gate run is red.
- **KILL:** any other field moves, or a real producer is found relying on `-8`. Then the two codes stay unassigned, MUL cannot derive `Dissonant` this way, and W3d needs a different mechanism — **surfaced to the operator, not reimplemented locally** (L-58).

### J3 — Does the promoter discriminate? (gates W4 shipping)

- **Green:** F-11 and F-12 both pass — it promotes a non-trivial minority and recovers nothing on the control.
- **KILL:** it treats ordinal-2 rows as warranted — promoting them to `Causal` indistinguishably from ordinal-1 — so the mediator distinction changes nothing about its behaviour. The promoter does not ship at any threshold; the counts go in the ledger as a negative result.
- **NOT a KILL (B-4, corrected 2026-09-11):** the promoter recovering a mediator on an ordinal-2 row. `INDIRECT_UNKNOWN_INTERMEDIATES` records that *the source* does not know the mediator — it does NOT assert that none exists or that any proposed one is false, and there is no ground-truth label here to judge it against. Killing on recovery would kill a promoter that correctly found something the source lacked. `dismech_evidence`'s own wording (*"a reasoner that 'recovers' one is hallucinating closure"*) is a claim about epistemic restraint, and this plan previously mis-read it as a ground-truth label.

---

## 6. Explicitly out of scope

Named so a later session does not read silence as an opening.

- **No new widget or surface vocabulary**, and no behaviour on a rendered surface — a2ui-rs T1/T2 (L-56).
- **No serialisation in the hot path**; the LE bytes are the wire format (L-56).
- **No band ordinal minted**; `ReasoningBand` is read-only to this arc (L-28).
- **No 5th truth ordinal**; `Dissonant` is derived, never stored (L-26).
- **No parallel object model** for the transcoded C++; adapters target the Core (L-59).
- **No upstream issue, PR or comment filed** on any public repo for the substrate asks in W3 — those are **surfaced to the operator in session**. The i4 change (D-ST-3a) is exactly such an ask.
- **No execution-diff harness.** It is a legitimate *value*-parity oracle and a different plan; if J1 kills the stereo thesis it gets its own document.
- **No claim of "MIT proposed X"** until W5 grounds it.
- **NEVER convert R2IL into SPO (C-6).** Operator ruling 2026-08-26, verbatim in `OGAR/crates/ogar-r2il/src/lib.rs`: *"R2IL is EXECUTED, never pre-converted … converting live V4 R2IL down to a V3 SPO projection before running it is a lossy static shadow of semantics the interpreter already has first-class."* `ruff_r2il` (the ruff-side R2IL→SPO harvest) is explicitly **not on this path and never will be**. R2IL is a native plane (§2.-1); lifting a binary into R2IL is fine, lowering R2IL into triples is not.
- **Do NOT vendor `r2sleigh` as a submodule.** `ogar-r2il` exists *specifically* so r2sleigh is not a dependency, and calls that "the load-bearing design choice rather than an omission": r2il's opcodes are an **enumeration**, loco needs an **arity table** (82 slots, `0x90..=0xE1`), and taking the crate drags `Varnode`/`SpaceId`/`ArchSpec` into every consumer. Its `lance-graph-contract` edge is a **dev-dependency** to keep the lib surface `ogar-loco`-only.
- **No hand-rolled code-graph in frontend comments (C-7).** A fact about predicates — *"`CppFunction::calls` is every callee, `CppMethod::calls` is the closed mutator set, they must not alias"* — is a **code-graph relation**, i.e. DATA, queryable and reusable across the cpp / ruby / python / csharp arms. Writing it as a twenty-line prose block in one frontend's example is hand-rolling: it cannot be queried, cannot be reused, and drifts from the code it describes. **The executable guard is right; the prose beside it is the violation.** If the boundary cannot yet be expressed as a relation, that is the §2.0 proof obligation — not a licence to write the paragraph. Measured instances in this arc: the 11-line `is_static` comment and the 20-line `calls` comment, both landed on `ruff` #118 by me.
- **No internal digest pins** on artefacts this arc produces. Identity gates go on content that means something — row counts, structural invariants, a census.

## 7. Open questions

| id | question | blocks |
|---|---|---|
| Q-1 | Does r2il find the functions at all? | everything (D-ST-1a) |
| Q-2 | Is `c` independent of control density, or its restatement? | D-ST-2b (F-7) |
| Q-3 | Is the ordinal-1 disposition alignment load-bearing or coincidental? | D-ST-3b (F-8) |
| Q-4 | Which causality-learning proposal is meant? | D-ST-4b method (W5) |
| Q-5 | Does any live producer write `-8` today? | J2 |
| Q-6 | Is there a corpus outside DisMech with the ordinal-1/ordinal-2 split, so the promoter is not calibrated on one source? | D-ST-4b generality |
| Q-7 | Does the V4 varnode genuinely fit `6x(u8:u8)` without widening, or does `Custom(u32)` force an escape? | L-22 |

## 8. Provenance

Findings L-1..L-10 are this repo's; L-11..L-20 `ruff` (PR #118 open, head
`b13168b4`); L-21..L-24 `r2sleigh`; L-25..L-42 `lance-graph`; L-43..L-55 the
private consumer arc, method only; L-56..L-61 sibling doctrine;
L-62..L-63 this arc's own process finding.

Append-only. Regrade in place with a dated note; never delete a row.
