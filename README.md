# OpenGGS-rs

A Rust transcode of [OpenGGS](https://github.com/bugix/OpenGGS) — Roman Högg's
free-software remake of "The Great Giana Sisters", written in C/C++ with SDL2.

**Status: the simulation spine, not the game.** 13 of the C++ corpus's 181
functions are ported: player physics, tile collision, and the `.lvl` stage-file
format. There is no renderer, no audio, no enemies and no game loop yet. What is
and is not ported is tracked mechanically, not by prose —
see [`docs/TRANSCODE-LEDGER.md`](docs/TRANSCODE-LEDGER.md).

```
crates/openggs-core       simulation: world, player, physics, tile collision (no SDL, no I/O)
crates/openggs-stagefile  reader for the .lvl stage-file format
crates/openggs-spo        the libclang harvest inventory + the coverage ledger
ore/functions.tsv         the harvest, verbatim: 181 function definitions from 55 TUs
```

## The approach

The C++ is one binary in which physics, rendering, audio and input all read and
write the same file-scope globals (`PC`, `World`, `GV`, `StageC64`, `TileType`).
The port takes the half that is a pure state transition and makes it a value:

```rust
let mut sim = Sim::default();
sim.stage = StageFile::open("base/stages/classic.lvl")?.stage(0)?.to_stage();

let events = sim.step(Input { right: true, jump: true, ..Input::default() }, 16);
if events.play_jump_sound { /* the shell plays it — this crate has no audio */ }
```

Routines that call `AUDIO_Sound_Play` inline upstream return a `bool` instead,
which is what keeps `openggs-core` free of an SDL dependency.

## The map is machine-made

The inventory of what the C++ contains comes from a libclang walk of all 55
translation units using `ruff_cpp_spo` from the AdaWorldAPI `ruff` fork — 55/55
parsed, zero error diagnostics, 181 functions, 34,326 ordered behavioural
events. `openggs-spo` embeds that inventory and **fails its test suite if the
ported list names a function the harvest does not contain**, so the coverage
number cannot drift away from the corpus.

Coverage means "a Rust routine was transcoded from this C++ function". It does
**not** mean behavioural parity with the original binary, which is unmeasured;
the ledger names the probe that would establish it.

## Build

```sh
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

Toolchain is pinned in `rust-toolchain.toml` (1.98.1, edition 2024).
`unsafe_code = "forbid"` workspace-wide.

To regenerate the harvest you need libclang and the SDL2 headers; the exact
invocation is in the ledger.

## Licence

GPL-2.0-only, inherited from upstream OpenGGS. `LICENSE` is the upstream file
verbatim. No C++ source and no game asset is vendored here — the harvest reads
the upstream corpus in place.
