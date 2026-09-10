# OpenGGS C++ → Rust transcode ledger

Append-only. New entries at the top; existing entries are corrected in place
only by adding a dated `⊘` note, never by deletion.

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
