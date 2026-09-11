# OpenGGS C++ → Rust transcode ledger

Append-only. New entries at the top; existing entries are corrected in place
only by adding a dated `⊘` note, never by deletion.

## Furnace pass (2026-09-10) — ore → furnace → slag → repeat

Applied the `ruff_r2il` method (`.claude/knowledge/consumer-transcode-furnace-playbook.md`,
`ruff_r2il::{ore,furnace,slag,convention}`) to the OpenGGS harvest. New crate:
`openggs-furnace`, 36 tests.

| stage | module | what it does here |
|---|---|---|
| intake arm | the `ruff_cpp_spo` harvest | lossless, untouched, not re-extracted |
| ore | `ore` | 34,326 typed facts over the 4 TSVs, axes interned in SORTED order so an address is stable across runs |
| convention | `convention` | radix tree over facet prefixes, longest-prefix-wins. **A config FILE, not match arms** |
| furnace | `furnace` | melt → flat `FlatFact { id, facet, concern, a, b }`, concern-separated |
| slag | `slag` | named residuals, addressed, ranked by shape. No `Other`, ever |
| repeat | `HarvestReport::slag_shapes` | the top shape names the next convention row |

Facet is the **V3 content-blind 4+12** shape: `classid u32 = 0` (zero-fallback
ladder: default class, prefix routing dormant — the same sanctioned reading
`ruff_r2il::mgra` uses for a 6502 image) plus six `(u8:u8)` rails —
`unit · function · scope · depth:kind · seq · symbol`. Never widened to a u16
or a u24. `const _: () = assert!(size_of::<Facet>() == 16)`.

### Measured, two passes, no Rust changed between them

| | pass 1 | pass 2 |
|---|---|---|
| convention | `ore/convention.pass1.ggs` (3 kinds, 1 row) | `ore/convention.pass2.ggs` (13 kinds, 13 rows) |
| enumerated | 34,326 | 34,326 |
| melted | 21,854 | 34,326 |
| residual | 12,472 | 0 |
| conserved | true | true |

Pass 1's slag was ten shapes, every one `EventKindNotInConvention`, ranked
`ScopeExit`/`ScopeEnter` 3,487 each · `Condition` 1,688 · `Branch` 1,617 ·
`Call` 1,260 · `Decl` 353 · `Cast` 346 · `Param` 139 · `Break` 69 ·
`Return` 26. Each shape named exactly one row. Pass 2 is that list as config.
Artifacts: `ore/pass{1,2}-census.md`, `ore/pass{1,2}-slag-shapes.tsv`.

### F-3 — residual hit 0 in ONE pass, and that is a WEAKNESS, not a win

The playbook says repeat until the slag pile stops shrinking. It stopped
immediately, which means the pass-1 melt gate is too easy: *"does the
convention name this event kind"* is satisfied by any complete list of the
13 kinds the harvest emits. A gate that a 13-line file fully discharges
measures the file, not the corpus.

What the pass genuinely bought is the **concern-separated, addressed
decomposition** — 34,326 rows at stable 16-byte coordinates, split
State 22,207 / Structure 6,974 / Control 3,400 / Interface 1,399 /
Conversion 346 (sums to 34,326) — which is the God-object-to-SoC move the
playbook describes.

> ⊘ Corrected 2026-09-10 (codex P2 on PR #1). An earlier revision of this
> line published State as 29,181, which made the breakdown sum to 41,300
> against a stated 34,326 and contradicted `ore/pass2-census.md` committed
> beside it. The cause: 22,207 + 6,974 = 29,181 — State was conflated with
> Structure and Structure then listed again, double-counting it. The census
> file was always right; the prose was not.
What it did NOT buy is any claim about Rust.

The next gate has to be the playbook's **three-axis mint gate**
(`CONCEPT ⟺ METHOD ∧ STORAGE ∧ STRUCTURE`), which a kind list cannot
discharge. Unbuilt.

### F-4 — control density does NOT split data from behaviour (negative result)

The hypothesis was a bimodal corpus: tables near 0, algorithms far from it,
a threshold in the valley between. It was built on two functions —
`PC_Define` (131 writes, 2 branches) against `PC_Run` (10 branches in 21
scopes) — and those two do differ.

Over all 181 functions there is **no valley**. Unimodal and smooth:
min 0.000 · p25 0.045 · median 0.087 · p75 0.134 · max 0.255, bucket counts
decaying monotonically (58 / 49 / 39 / 30 / 4 / 1 in 0.05-wide bins). So any
threshold is an arbitrary cut, and `Census::data_shaped` must not be read as
a classifier. The synthetic unit test passed because it asserted the two
constructed extremes — exactly the vacuous-assertion shape: it could not
have failed on this corpus either way.

The one non-arbitrary criterion is `threshold = 0.0`: functions with no
`Control` row at all. That selects **17 of 181** (`defineAngles` with 64
state rows and zero control is the clean positive; also `AUDIO_Define`,
`loadTextures`, the `*_Draw` routines). It is also NARROW — **`PC_Define`,
the corpus's most table-shaped function, is not among them**, because its
single file-open error check contributes two Control rows. A criterion that
misses the canonical positive case is weak, and `a_single_guard_is_enough_to_leave_the_zero_control_set`
pins that so it cannot be widened silently.

### F-5 — the two oracles are identified but only half-available

The playbook requires two independent witnesses that fail differently.
Mapped onto a game:

| oracle | witness here | state |
|---|---|---|
| **value parity** | the `.lvl` stage file — the legacy data, byte-exact | PARTIAL: `openggs-stagefile` decodes the shipped `classic.lvl` at a twice-measured stride, but there is no encode side, so no round-trip byte-parity test |
| **structure parity** | the Klickwege equivalent: the screen/loop graph (`LOOP_Menu` → `LOOP_Gameloop` → `LOOP_Options` …), harvested from the ore's 1,260 `Call` events, never authored | NOT BUILT |

Neither is the frame-by-frame physics parity F-1's successor still needs.
Three distinct unmeasured things, named separately.

### Coverage is STILL 13 / 181

The furnace emits no Rust. It produced the map, the addressed decomposition
and the measurements above; it did not transcode a single additional
function, and `openggs-spo`'s ledger is unchanged. Anyone reading "34,326
facts melted" as progress on the port would be reading it wrong.

______________________________________________________________________

## What exists today (2026-09-10)

| Crate | Role | State |
|---|---|---|
| `openggs-core` | Renderer-free simulation: world constants, player state, physics, tile collision | 11 tests green |
| `openggs-stagefile` | Reader for the `.lvl` stage-file format | 5 tests green, incl. the shipped `classic.lvl` |
| `openggs-spo` | The libclang harvest inventory + the coverage ledger | 5 tests green |

`cargo test` — 21 passing. `cargo clippy --all-targets --all-features` — clean.
`unsafe_code = "forbid"` at the workspace root.

## The harvest is the map, and it is machine-made

The C++ side of the ledger is not hand-written. All 55 translation units were
walked with `ruff_cpp_spo` (libclang 18) via the AdaWorldAPI `ruff` fork:

```sh
cmake -S OpenGGS -B build -DCMAKE_EXPORT_COMPILE_COMMANDS=ON
ORE_CC_JSON=build/compile_commands.json ORE_OUT=/tmp/ore \
  LIBCLANG_PATH=/usr/lib/llvm-18/lib \
  cargo run -p ruff_cpp_spo --features libclang --example harvest_events
```

Result: **55/55 TUs parsed, 0 error diagnostics, 0 parse failures; 181 function
definitions, 34,326 ordered behavioural events, 1,137 symbols.** The function
column is committed verbatim as `ore/functions.tsv` and embedded by
`openggs-spo`, which fails its suite if a `PORTED` entry names a function the
harvest does not contain.

**The class plane is empty, and that is the corpus, not a harvest failure.** The
run reported `A0 five-set facts: 0` — OpenGGS is plain C: 25 file-scope structs
and 181 free functions, no classes. So `inherits_from` / `virtually_overrides`
and the rest of the C++ machine-plane predicates have nothing to bind to here.
What carries the signal is the ordered event ore (`Read`/`Write`/`Branch`/
`Condition`/`Call`/`ScopeEnter`), which is exactly the arm designed for
imperative bodies.

## Coverage: 13 / 181 functions

Not a percentage worth celebrating — it is the physics spine and the file
format, which is what the rest hangs off. Ported:

`PC_Define` · `PC_PowerUp_Set` · `PC_Run` · `PC_Friction` · `PC_Jump` ·
`PC_Gravity` · `PC_Check_Tilecollision` · `PC_Collision_{Down,Up,Left,Right}` ·
`GAME_ENVIRONMENT_Define` · `Load_Stagefile`

## Not ported — named, not silently missing

Each of these is in `openggs-spo::unported()`, and three of them are pinned by a
test so that porting one forces this document to be updated:

- **Rendering** (`PC_Draw`, `STAGE_Draw`, `GRAPHICS_*_Draw`, `SYSTEM_SDL_*`,
  `INTERFACE_Draw`, `SYSTEM_Draw_Text`) — needs an SDL shell crate.
- **Enemies** (`ENEMIES_*`, ~15 functions) and **sprites/bullets/platforms/drop
  stones** (`GRAPHICS_Sprites_*`) — whole subsystems.
- **Block-contents collision** (`PC_Collision_{CoinBlock,PowerUpBlock,Breakable,
  DropStone,WarpStone,CoinBlockHelmet}`) — the C++ fires these from *inside*
  `PC_Collision_Up`. `openggs-core::collision` documents the omission at the
  call site rather than stubbing them to "no collision", which would read as a
  transcode defect later.
- **Non-solid tile collision** (`PC_Check_TilecollisionNonSolid`, coins, exits).
- **Death** (`PC_Killed`, `PC_Death_Routine`), **game loops / menus / editor**
  (`LOOP_*`, ~60 functions), **audio** (`AUDIO_*`).

## Parity is UNMEASURED

Nothing here has been run against the original binary. The tests assert the
transcode's *internal* invariants (a fall lands on the floor surface; a runner
settles flush against a wall face at 143 and stays there; `int(v/4)` truncates
toward zero on both signs). That is not the same as byte-for-byte behavioural
parity with libopengGS, and this document does not claim it.

The falsifier that would earn the claim: instrument the C++ to dump
`(pos_x, pos_y, run_velocity, jump_velocity)` per frame for a scripted input
sequence on `classic.lvl` stage 0, run the same script through `Sim::step`, and
diff. Until that runs, coverage means "a Rust routine was transcoded from this
C++ function", nothing more.

## Findings

### F-1 — the shipped `.lvl` files match the current struct exactly

`sizeof(ImportStage_Struct)` compiled against the real header is **36,732**
bytes. The shipped `classic.lvl` is 1,405,248 bytes, which is *not* a multiple
of 36,732 — 38 records plus a 9,432-byte tail. The first instinct, that the data
predates the struct, was **wrong**: locating the record boundaries in the data
itself (the default stage name recurs at a fixed stride) gives exactly 36,732.
The file simply ends mid-record, because the writer `fseek`s to a stage's slot
and writes one record — the tail is whatever the last write left.
`StageFile::stage_count` counts complete records only.

Both offsets and stride were therefore *measured* twice, from opposite ends
(`offsetof` on the compiled struct; boundary detection in the data), and agree.

### F-2 — the flush-against-wall test caught a wrong assertion, not a wrong port

The first version asserted `pc.hit_wall` at end of frame. It failed. The
simulation was right: wall contact zeroes `run_velocity`, and the direction
probes only fire while velocity is non-zero, so the flag alternates between
frames while leaning on a wall. The stable observable is the position. The test
now pins the right edge at 143 against a wall face at 144 — which also fails if
the player never moved at all, where `< 144` alone would have passed.

## Relation to the sibling repos

Read for reference, and deliberately **not** wired in:

- **OGAR** — the Core-First doctrine is why this is a hand-shaped Rust core
  rather than a generated object model. `ruff_cpp_codegen` renders *signature
  manifests* targeting `lance_graph_contract::MethodSig`; it does not emit
  working game logic, so the harvest here is used as a **map and a coverage
  gate**, not as a code generator. Claiming otherwise would be the exact
  "generated layer is only as clean as the Core it targets" failure.
- **lance-graph / lance-graph-java** — no dependency. A 16-byte classid facet
  and an SoA row store buy nothing for a single-player platformer with a
  256×30 tile grid; adding them would be architecture for its own sake.
- **a2ui-rs** — the render target, *when* the SDL shell lands. Its
  address-the-screen model (`NodeDelta` down, `ActionInvoke` up) is a plausible
  fit for the HUD and the level editor, neither of which is ported yet.

The one doctrine actually applied is the boundary discipline: the simulation
core takes no dependency on its shell. Routines that call `AUDIO_Sound_Play`
inline in the C++ return a `bool` instead.

## Upstream and licence

Upstream: Roman Högg's OpenGGS, <https://github.com/bugix/OpenGGS>, GPL-2.0.
This port is a derivative work and carries the same licence; `LICENSE` is the
upstream file verbatim. No C++ source or game asset is vendored into this repo —
the harvest reads the corpus in place.

`pstaender/giana-sisters-lost-levels` was **not** used: it is a separate
JavaScript project, not a source for this transcode.
