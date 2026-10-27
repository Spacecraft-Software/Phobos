# Phobos — Plan

MVP: M0–M5 — Phobos v0.1

This plan sets the order of the work in [`PRD.md`](PRD.md). It does not restate
the requirements. Every work package (`P-`) names the requirement items (`R-`)
it delivers. A package is ticked when its pull request is merged and its checks
pass. The `R-` items in `PRD.md` are ticked when every package delivering them
is done. Updated 2026-10-27.

## How the work runs

- **One package, one branch, one pull request.** The branch is named
  `p-NNN-short-name`. It is squash-merged with a signed commit (§6.3) and the
  branch is then deleted. Nothing is pushed to `main` directly.
- **Every PR passes the M0 gates:** `cargo fmt --check`;
  `cargo clippy --all-targets --all-features --locked -- -D warnings`;
  `cargo test`; doc tests; `cargo deny check`; `cargo audit`; `reuse lint`.
- **Rust rules.** Load `microsoft-rust-guidelines` before touching any `.rs`
  file. Library crates return canonical error structs
  (M-ERRORS-CANONICAL-STRUCTS); `thiserror` may derive `Display`. `phobos-app`
  uses `anyhow` with `.context()` (M-APP-ERROR). No `unwrap` or `expect` on
  input, only where M-PANIC-IS-STOP allows a panic. Error types are
  `Send + Sync + 'static`, pinned by a compile-time assertion next to the type.
- **Simulation invariants, enforced by tooling:**
  - `#![forbid(unsafe_code)]` in `phobos-fixed`, `phobos-wad`, `phobos-data`,
    `phobos-sim`, and `phobos-game`.
  - `clippy::float_arithmetic` denied in `phobos-fixed`, `phobos-sim`, and
    `phobos-game`.
  - `std::time` and `rand` banned there through `disallowed-types` and
    `disallowed-methods`.
  - `overflow-checks = true` in the test and dev profiles.
- **Benchmarks before and after.** No optimisation lands without a criterion
  or `-timedemo` measurement on both sides of the change, recorded in the PR
  (§3.2). Flamegraphs are used to find where to look.
- **Transliteration discipline.** Each ported C file names its source path in
  its module doc comment and keeps the original function order, so a reviewer
  can read the two side by side. Files carrying id Software code keep the
  id copyright line and `GPL-2.0-or-later` in their SPDX header (§4.2).

## Sequencing

```
M0 ─► M1 ─┬─► M2 (simulation) ──────────────┬─► M5 ─► M6 ─► M7
          │      └─ demo harness (P-038) ───┘          └──► M8
          └─► M3 renderer core (P-042–P-047) ─► M3 front end ─┘
                                M4 audio (after P-029) ──────┘
```

- **Critical path:** M0 → M1 → M2 → M5. Demo sync is the product, so the
  headless demo harness (P-038) is built inside M2, long before the
  user-facing demo features of M5. Every simulation package after P-029 is
  checked against the oracle as it lands.
- **Parallel track:** the renderer core (M3, P-042–P-047) needs only M1's
  decoded data and a read-only `World`. It can proceed beside M2. The front
  end (status bar, menus, intermission, finales) needs M2's game flow.
- **M4** needs sound origins from `p_mobj` (P-029). Music can start any time
  after M1.
- **M7 and M8** follow the release. The one exception is P-026, which keeps
  the info tables mutable so DeHackEd needs no redesign later.

### Fidelity ladder

Each rung is a statdump match against Chocolate Doom (R-011) before the next
rung starts.

| Rung | Target | Proves |
|------|--------|--------|
| 1 | `doom1.wad` DEMO1 | Movement, collision, hitscan, the E1 monsters |
| 2 | `doom1.wad` DEMO2, DEMO3 | Doors, lifts, specials, more of E1 |
| 3 | `freedoom1.wad` built-in demos, `-gameversion ultimate` | E2–E4 actors, Ultimate behaviour |
| 4 | `freedoom2.wad` built-in demos | Doom II monsters, super shotgun, MAP specials |
| 5 | Free-IWAD demo corpus (T-024) | Breadth |
| 6 | Commercial IWAD built-in demos, local (P-075) | Doom, Ultimate, Doom II, TNT, Plutonia, `final` behaviour |

## Toolchain and dependency candidates

Category D owes audit tooling only (§24). Each crate is added in the package
that first needs it, after `cargo deny` accepts its licence and advisories.
Versions are pinned in `Cargo.lock`.

| Need | Candidate | First package | Note |
|------|-----------|---------------|------|
| Toolchain | Latest stable at M0, pinned in `rust-toolchain.toml`; edition 2024 | P-002 | `fuzz/` carries its own nightly toolchain file for `cargo-fuzz` only |
| Library errors | `thiserror` | P-014 | Derives `Display` only |
| App errors | `anyhow` | P-048 | `phobos-app` only |
| Logging | `tracing`, `tracing-subscriber` | P-014 | Structured logs; no `println!` in threaded code |
| Data parallelism | `rayon` | P-043 | Behind a benchmark (R-056, level load) |
| Window and input | `winit` | P-048 | Main-thread event loop |
| GPU | `wgpu` | P-049 | Presenter first, enhanced renderer in M8 |
| CPU presenter | `softbuffer` | P-050 | Fallback |
| Audio output | `cpal` | P-064 | Callback thread |
| Audio command queue | `rtrb` (SPSC ring) | P-064 | Wait-free, bounded; alternative `crossbeam::queue::ArrayQueue` |
| OPL synthesis | `nuked-opl3` | P-067 | 0.1.0; fallback is an in-tree port |
| Configuration | `serde`, `toml` | P-052 | Configuration only; saves use the hand-written format (P-077) |
| Golden frames | `png` | P-060 | Test and tool code |
| Gamepad | `gilrs` | P-063 | Optional |
| Tests | `proptest`, `criterion` | P-023, P-022 | Dev-dependencies |

Choices that keep dependencies out:

- **Command-line parsing is hand-written.** Vanilla's `M_CheckParm` scheme
  (`-iwad`, `-warp 1 1`) does not fit clap's model. A small scanner gives
  exact vanilla behaviour and also accepts `--iwad`.
- **XDG paths are read from the environment directly.** The rules are short
  and need no crate.
- **The per-tic checksum is our own FNV-1a fold.** It is deterministic and
  stable across versions.
- **WADs are read into a `Vec<u8>`, not memory-mapped.** Mapping is `unsafe`
  and a file changed underneath it is undefined behaviour; IWADs are at most
  about 18 MiB.

## M0 — Foundation and compliance

Exit when a fresh clone passes CI with every gate active on an empty
workspace, and the oracle runs a demo headless in CI.

- [ ] P-001 `git init`; Cargo workspace (resolver 3) with all eleven crates as empty members (R-001)
- [ ] P-002 `rust-toolchain.toml` pinned to stable with rustfmt and clippy; `fuzz/rust-toolchain.toml` on nightly (R-001)
- [ ] P-003 `[workspace.lints]` plus per-crate `forbid(unsafe_code)`, float and clock bans in simulation crates (R-026)
- [ ] P-004 `flake.nix` dev shell with toolchain, `cargo-deny`, `cargo-audit`, `reuse`, `chocolate-doom` (R-011)
- [ ] P-005 Posture files copied from `/spacecraft-software/license/`: `LICENSE`, `LICENSES/`, `NOTICE.md`, `CONTRIBUTING.md`, `CREDITS.md`, `SECURITY.md` (R-002, R-009, R-010)
- [ ] P-006 `README.md` with posture, game declaration, Category D, conformance claim (R-002)
- [ ] P-007 `AGENTS.md` with build commands, simulation invariants, crate dependency rules; `CLAUDE.md` importing it (R-003)
- [ ] P-008 SPDX headers, `REUSE.toml` for generated HTML, `.gitattributes`, `.editorconfig` (R-004, R-005)
- [ ] P-009 `deny.toml`: GPL-3.0-compatible licence allow-list and bans that enforce the crate dependency rules (R-006)
- [ ] P-010 CI workflow running every gate, plus a CR-byte check (R-005, R-006)
- [ ] P-011 Fixed-output Nix derivations for Freedoom (pinned release) and `doom1.wad`, by SHA-256 (R-012)
- [ ] P-012 CI job proving the oracle runs a `doom1.wad` demo headless and writes a statdump (R-011)
- [ ] P-013 GPL "or later" grant checked against the 1999 release headers; result in `NOTICE.md` (R-008)
- [ ] P-014 Registry pull requests: `PROJECTS.md`, Standard §2.1 and §18.5 (Standard and Construct, same version) (R-007)

## M1 — WAD and resource layer

Exit when all eight IWADs are identified, every lump of the free IWADs
decodes, and fuzzing has run clean for its time box.

- [ ] P-015 `phobos-wad`: header and directory parsing with checked arithmetic and a canonical `WadError` (R-013)
- [ ] P-016 Lump namespace: ordered WAD stack, last-wins lookup, marker ranges (R-016)
- [ ] P-017 IWAD identification table and search order (R-014, R-015)
- [ ] P-018 `phobos-data` map decoding into typed arrays, with every cross-index validated (R-017)
- [ ] P-019 Graphics decoding: palette, colour maps, patch posts, flats, PNAMES and TEXTURE definitions, ENDOOM (R-018)
- [ ] P-020 Sound decoding: DMX sample headers, MUS headers, GENMIDI banks (R-019)
- [ ] P-021 Fuzz targets for WAD directory, map, patch, and texture decoding, seeded from Freedoom (R-020)
- [ ] P-022 Criterion benchmark for full IWAD load and decode as the first baseline (performance targets)
- [ ] P-023 `phobos-tools wad ls` and `wad identify` with `--json` (R-021)
- [ ] P-024 PWAD sprite and flat merging (R-022) (optional)

## M2 — Deterministic simulation core

Exit at fidelity ladder rung 4: `freedoom2.wad` built-in demos match the oracle.

- [ ] P-025 `phobos-fixed`: `Fixed` and `Angle` newtypes, wrapping operators, `FixedMul`, `FixedDiv` with the overflow branch, proptest suite (R-023)
- [ ] P-026 Trig and random tables carried verbatim; info tables converted once by a committed script in `tools/` into a runtime `GameInfo` value that DeHackEd can later patch (R-024, R-025, R-028)
- [ ] P-027 Thinker arena: generational `MobjHandle`, index-linked list in insertion order, single-thread contract documented (R-027)
- [ ] P-028 `p_setup`: build `Level` from map data, blockmap, reject, sector line lists (R-029)
- [ ] P-029 `p_mobj` and `p_tick`: spawn, remove, state machine, 35 Hz think loop over `TicCmd` (R-026)
- [ ] P-030 `p_maputl` and `p_map`: blockmap iteration, path traversal, collision, sliding, use lines (R-029)
- [ ] P-031 `p_user` and `p_pspr`: player think, all nine weapons (R-030)
- [ ] P-032 `p_enemy`: Doom monsters first, then the Doom II monsters (R-031)
- [ ] P-033 `p_spec`, `p_doors`, `p_plats`, `p_floor`, `p_ceilng`, `p_lights`, `p_switch`, `p_telept` (R-032)
- [ ] P-034 Level triggers and the MAP30 boss brain (R-033)
- [ ] P-035 `g_game`: game modes, missions, level flow, secret exits, `-gameversion`, skills and switches (R-034, R-035, R-036)
- [ ] P-036 Vanilla cheat codes (R-037)
- [ ] P-037 Per-tic FNV-1a `World` checksum (R-038)
- [ ] P-038 Headless demo harness in `phobos-tools`: reads `.lmp`, writes statdump and checksum trace (R-039)
- [ ] P-039 Fidelity ladder rungs 1–4 passing in CI (R-039; groundwork for R-072)
- [ ] P-040 Criterion tic benchmark on `freedoom2.wad` MAP30 as baseline (performance targets)
- [ ] P-041 spechit and intercepts overflow emulation (R-040) (optional)

## M3 — Classic renderer and front end

Exit when every free-IWAD map renders with golden frames in CI, and a full
game runs from title to finale with keyboard and mouse.

- [ ] P-042 `v_video`: indexed framebuffer type and patch drawing (R-041)
- [ ] P-043 `r_data`: texture compositing at level load, Rayon only if a benchmark shows a gain (R-041)
- [ ] P-044 `r_main`, `r_bsp`, `r_segs`: view setup, BSP walk, wall columns (R-041)
- [ ] P-045 `r_plane`, `r_things`, `r_draw`: visplanes, sprites, masked mid-textures, fuzz, sky, light diminishing (R-042)
- [ ] P-046 Vanilla render limits with logged, non-fatal fallback (R-043)
- [ ] P-047 Renderer takes `&World` only, so the type system prevents writes to game state (R-041)
- [ ] P-048 `phobos-app`: winit event loop on the main thread, game thread, triple-buffer frame handoff (R-044)
- [ ] P-049 wgpu presenter: indexed texture, `PLAYPAL` lookup in WGSL, palette flashes, 4:3 scaling (R-044)
- [ ] P-050 `softbuffer` presenter selected when no adapter is found (R-045)
- [ ] P-051 Input mapping from winit events to `TicCmd` with remappable bindings (R-052)
- [ ] P-052 TOML configuration with warn-and-default handling and XDG paths (R-053)
- [ ] P-053 `st_*` and `hu_*`: status bar, face, messages, level names per mission (R-046)
- [ ] P-054 `am_map` automap (R-047)
- [ ] P-055 `m_menu` menus with ENDOOM on quit (R-048)
- [ ] P-056 `wi_stuff` intermission for Doom and Doom II (R-049)
- [ ] P-057 `f_finale`: text screens per mission, cast call, bunny scroller, E4 ending (R-050)
- [ ] P-058 `d_main` loop: title, attract demos per game mode (R-051)
- [ ] P-059 No-IWAD screen with the `steelbore` theme and bundled OFL font (R-054)
- [ ] P-060 Headless render-to-PNG golden frames in CI (R-055)
- [ ] P-061 2× and 4× scale with banded parallel drawing (R-056) (optional)
- [ ] P-062 Uncapped frame rate with interpolation between tics (R-057) (optional)
- [ ] P-063 Gamepad input through gilrs (R-058) (optional)

## M4 — Sound and music

Exit when sound effects and OPL music play in all game modes, and a run with
no audio device completes silently with one warning.

- [ ] P-064 Audio thread on cpal fed by a bounded SPSC ring; drop-oldest with a counter when full (R-059)
- [ ] P-065 `s_sound` channel logic on the game side and a mixer on the audio side (R-060)
- [ ] P-066 MUS sequencer driven by tic time (R-061)
- [ ] P-067 OPL backend: audit `nuked-opl3`, else port Nuked OPL3 in tree; GENMIDI voice mapping (R-062)
- [ ] P-068 Degraded silent mode when no device opens (R-063)
- [ ] P-069 Sound-effect and music volume in menu and configuration (R-064)
- [ ] P-070 SoundFont MIDI backend (R-065) (optional)
- [ ] P-071 PC speaker sound effects (R-066) (optional)

## M5 — Demos, saves and fidelity gate

Exit at fidelity ladder rung 6 on the maintainer's machine. This milestone
completes Phobos v0.1.

- [ ] P-072 `-playdemo`, `-timedemo`, and `-statdump` in `phobos-app`, sharing the M2 harness code (R-067, R-069, R-070)
- [ ] P-073 `-record` in vanilla format (R-068)
- [ ] P-074 CI differential job over the free-IWAD corpus, one process per demo (R-072)
- [ ] P-075 Local acceptance script for the six commercial IWADs, run before each release (R-071)
- [ ] P-076 `phobos-tools demo diff` reporting the first divergent tic (R-073)
- [ ] P-077 Native save: versioned header, hand-written field order, handles stored as arena indices (R-074)
- [ ] P-078 Save refusal path: message shown, game continues, round-trip checksum test (R-075)
- [ ] P-079 Vanilla `.dsg` loading (R-076) (optional)

## M6 — Release engineering v1.0

Exit at the §19.4 G3 gate with a signed tag.

- [ ] P-080 `doc/phobos.texi` manual building to Info, HTML, and PDF (R-078)
- [ ] P-081 `packaging/guix.scm`, `packaging/default.nix`, `packaging/PKGBUILD` with `install-info` (R-079)
- [ ] P-082 `--help` and `--version` with §15 attribution (R-080)
- [ ] P-083 README compliance section and tailoring register (R-081)
- [ ] P-084 Release profile (LTO, `codegen-units = 1`) noted in `Cargo.toml`, with before and after benchmarks (R-082)
- [ ] P-085 SBOM from `Cargo.lock` in the release workflow, with checksum (R-077)
- [ ] P-086 Clean-environment install from each of the three package definitions (R-083)
- [ ] P-087 G3 checklist run and signed release tag (R-084)
- [ ] P-088 Limit-removing render mode (R-085) (optional)
- [ ] P-089 macOS and Windows builds (R-086) (optional)

## M7 — Vanilla modding extensions

Exit when DeHackEd patches from `-deh` and from a lump both apply and fuzzing
has run clean.

- [ ] P-090 DeHackEd parser applying patches to `GameInfo` (R-087)
- [ ] P-091 `DEHACKED` lump loading in WAD order (R-088)
- [ ] P-092 DeHackEd fuzz target (R-089)
- [ ] P-093 UMAPINFO (R-090) (optional)

## M8 — Enhanced GPU renderer

Exit when the parity check passes on every golden scene and device loss falls
back without ending the game.

- [ ] P-094 `RenderSnapshot` extracted from `World` each tic (R-091)
- [ ] P-095 Level geometry buffers and instanced sprites (R-092)
- [ ] P-096 Colour-map lighting in WGSL (R-093)
- [ ] P-097 Golden-scene parity check with a declared tolerance (R-094)
- [ ] P-098 Device-loss fallback to the software renderer (R-095)
- [ ] P-099 Dynamic lights and post-processing (R-096) (optional)
- [ ] P-100 Rust GPU shader path (R-097) (optional)
- [ ] P-101 Ray-traced ambient occlusion and shadows (R-098) (optional)
- [ ] P-102 WebGPU / WASM build (R-099) (optional)
