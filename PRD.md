# Phobos — PRD

MVP: M0–M5 — Phobos v0.1

| Field | Value |
|-------|-------|
| Document | Product Requirements Document, version 1.0 |
| Date | 2026-10-27 |
| Status | Draft, awaiting maintainer review |
| Codename | **Phobos** (approved 2026-10-27). Directory `phobos/`; repository `Spacecraft-Software/Phobos` |
| Product | Memory-safe Rust port of the id Tech 1 engine, playing *Doom*, *The Ultimate Doom*, *Doom II*, and *Final Doom* (*TNT: Evilution*, *The Plutonia Experiment*) from user-supplied IWADs |
| Class | **Game** (Standard §18.5); exempt from §18 and §10 |
| Assurance category | **D — Incidental** (Standard §19.1: games) |
| Posture | Personal / Hobby (Standard §5.1) |
| Engine licence | `GPL-3.0-or-later`; inherited id Software code `GPL-2.0-or-later` (§4.2) |
| Document licence | `CC-BY-SA-4.0` (Standard §4.1.1) |
| Governing standard | The Steelbore Standard v2.14 |
| Maintainer | Mohamed Hammad — <Mohamed.Hammad@SpacecraftSoftware.org> |

> **Trademark.** *Doom* is a trademark of id Software / ZeniMax Media. Phobos is
> an independent engine that reads Doom data files. It ships no id Software
> assets and no id Software logos, and claims no endorsement.

## Summary

Phobos is one Rust codebase, with no C in the tree. It plays every
commercial id Tech 1 IWAD from *Doom* to *Final Doom* and the free Freedoom
IWADs. The engine picks its game mode from the IWAD it loads. Fidelity is the
product. Vanilla demos shall play back in sync and level statistics shall match
Chocolate Doom, the reference vanilla port. This keeps the port honest: a
behavioural difference shows up as a failed test, not as a forum report.

The game logic is transliterated by hand from the GPL source release. The
platform, I/O, audio, and presentation layers are written as idiomatic Rust from
the start. A software renderer, ported from the original `r_*.c`, is the
reference for how the game looks. A wgpu renderer with optional enhancements
comes later and is checked against that reference. It never replaces it.

## Sources and what this PRD changes

This PRD merges two earlier documents and settles the questions they left open.

| Source | What it gave this PRD |
|--------|----------------------|
| `RustDoom.html` — *RustDoom* PRD v1.0, 2026-07-23 | Functional requirement areas, the crate split, the determinism principles, the two renderer modes, test and fuzz strategy, risks, CLI surface |
| `doomrustport.texi` / `.html` — *Doom-to-Rust Porting Reference* v1.0, 2026-07-23 | Source history and licence, prior art (room4doom, iron-doom, rust-doom), the three porting strategies, the determinism contract (16.16 fixed point, `P_Random` table, no `f32` in game logic), Freedoom-based CI, crate stack |

| Topic | RustDoom PRD said | This PRD decides | Why |
|-------|-------------------|------------------|-----|
| Scope | Doom 1 only; Doom II a non-goal | Doom, Ultimate Doom, Doom II, TNT, Plutonia | The engine is shared. The extra work is game-mode logic, Doom II actors, finales, and the cast call |
| Name | "RustDoom" (working title) | Codename **Phobos** (approved 2026-10-27) | §2 needs an aerospace or astronomy codename. A trademark should not appear in the product name |
| Licence | Open question: GPL-2.0+ or Apache/MIT | `GPL-3.0-or-later`, inheriting `GPL-2.0-or-later` | The port derives from the GPL source (§4.1, §4.2). "Or later" makes GPL-3.0 compatible. R-008 checks the grant |
| Porting route | Not chosen | Manual transliteration (texi strategy 3) | Keeps game-logic structure for demo sync. Shows memory-safe design instead of copying C's shape |
| Reference renderer | GPU "Classic Accuracy Mode" | Ported software renderer; GPU renderer after MVP | Bit-faithful output and headless golden frames are cheap in software. GPU rasterisation cannot reproduce vanilla column and visplane behaviour exactly |
| Shaders | Rust GPU preferred, WGSL fallback | WGSL primary; Rust GPU optional | Rust GPU needs a nightly toolchain, which conflicts with a pinned stable toolchain (§24, §3.1) |
| Telemetry | Opt-in | None at all | §9 PFA: no tracking, no network |
| Platforms | Windows, Linux, macOS required | Linux required; macOS and Windows optional (OQ-2) | Personal posture; Steelbore OS is the home platform |
| Music | MUS→MIDI with SoundFont | OPL emulation from `GENMIDI` by default; SoundFont optional | This is how vanilla sounds; room4doom showed it is practical |
| Fidelity oracle | Internal golden demos | Chocolate Doom `-statdump` differential runs plus internal checksums | An outside reference catches mistakes that self-recorded goldens copy forward |

## Needs

*Players and preservationists* want to play the classic id Tech 1 games on
current Linux systems with vanilla behaviour: the same physics, the same
monster behaviour, the same random sequence, so that historic demos play back
and speedrun records stay checkable. They supply their own IWADs and expect the
engine to find them, play every map, and never crash on a malformed PWAD.

*The maintainer* wants a complete id Tech 1 port in safe Rust, as a showcase.
The simulation shall have no `unsafe`, and every claim of fidelity shall be
backed by a test anyone can rerun. No existing Rust effort has shipped all of
*Doom* through *Final Doom*. room4doom comes closest and still lacks the cast
call and the automap.

## Target games

| IWAD | Game mode | Mission | Maps | Game version emulated | Available in CI |
|------|-----------|---------|------|----------------------|-----------------|
| `doom1.wad` | shareware | doom | E1M1–E1M9 | 1.9 | Yes (redistributable shareware) |
| `doom.wad` (registered) | registered | doom | E1–E3 | 1.9 | No — local only |
| `doom.wad` (Ultimate) | retail | doom | E1–E4 | ultimate | No — local only |
| `doom2.wad` | commercial | doom2 | MAP01–MAP32 | 1.9 | No — local only |
| `tnt.wad` | commercial | pack_tnt | MAP01–MAP32 | final | No — local only |
| `plutonia.wad` | commercial | pack_plut | MAP01–MAP32 | final | No — local only |
| `freedoom1.wad` | retail | doom | E1–E4 | ultimate | Yes |
| `freedoom2.wad` | commercial | doom2 | MAP01–MAP32 | 1.9 | Yes |

The game version can be changed with `-gameversion` (values `1.9`, `ultimate`,
`final`, `final2`, as in Chocolate Doom). This covers behaviour that changed
between executables. One example is how teleports set a thing's height in the
*Final Doom* executable. Another is the *Ultimate Doom* change to Lost Soul
bounce.

### Per-game content beyond the shared engine

- *Doom / Ultimate Doom*: episode select, animated intermission maps
  (`wi_stuff`), episode end texts, E3 bunny scroller, E4 ending, sky per
  episode, boss triggers on E1M8 (tag 666), E2M8, E3M8, E4M6, E4M8.
- *Doom II*: super shotgun; eight extra monsters, including the Arch-vile,
  Pain Elemental, and Icon of Sin; MAP07 tags 666/667; MAP30 boss brain
  spawner; secret exits to MAP31 and MAP32; between-map text screens; cast
  call; sky per map range.
- *Final Doom*: Doom II engine with mission-specific level names (HUD,
  automap), finale texts, and the `final` / `final2` executable behaviour.

## Goals

| ID | Goal |
|----|------|
| G1 | Play every map of the eight target IWADs from start to finish |
| G2 | Vanilla demos play back in sync; statistics match Chocolate Doom |
| G3 | Safe Rust throughout; `unsafe` forbidden in simulation crates |
| G4 | Untrusted WAD, demo, and save input can never crash or corrupt the engine |
| G5 | Simulation runs headless with no window, GPU, or audio dependency |
| G6 | Ships through Guix, Nix, and Arch packaging with a Texinfo manual |
| G7 | Optional modern GPU renderer that never changes gameplay |

## Non-goals

| ID | Non-goal | Reason |
|----|----------|--------|
| NG-1 | Boom, MBF, MBF21, ZDoom, DECORATE, ZScript | Scope; vanilla fidelity first. DeHackEd (M7) is the only extension format planned |
| NG-2 | Heretic, Hexen, Strife, Chex Quest | Different games |
| NG-3 | Network multiplayer | A network listener changes the licence (AGPL, §4.1) and the category (§19.2). It needs its own PRD |
| NG-4 | Distributing commercial IWADs | Legal |
| NG-5 | Console, mobile, VR | Platform and certification cost |
| NG-6 | Telemetry of any kind | §9 |
| NG-7 | Linking C code into the shipped binary | Product identity; the C oracle is test-only and out of tree |

## Product principles

1. **Correctness before cleverness.** Where gameplay depends on a behaviour,
   vanilla behaviour wins, including documented bugs.
2. **Determinism is the contract.** The simulation has a fixed 35 Hz tic,
   16.16 fixed-point maths with explicit wrapping, the original random tables,
   a stable iteration order, and no wall-clock time, floating point, or
   renderer state.
3. **Presentation never feeds back.** Renderers, audio, and UI read a snapshot
   of the simulation and never change it.
4. **Evidence over assertion.** Each fidelity claim names the test that proves
   it.
5. **Untrusted by default.** Every byte of a WAD, demo, or save is parsed with
   checked bounds into typed errors.

## Architecture

### Porting strategy

Phobos uses **manual transliteration** (porting reference, strategy 3). Game-logic
files (`p_*`, `g_game`, `info`, `tables`, `m_random`, `m_fixed`) keep their
function boundaries and call order, so a diff against `linuxdoom-1.10` stays
readable and the random-number call sequence is preserved. Rendering
(`r_*`), video (`v_video`), and the front end (`st_*`, `hu_*`, `am_map`,
`m_menu`, `wi_stuff`, `f_finale`) keep their algorithms but use idiomatic data
structures. Platform code (`i_*`) is rewritten from scratch.

The other two strategies were rejected:

- A `doomgeneric` shim plays on day one but keeps C in the tree until the
  end, which NG-7 forbids.
- `c2rust` output carries C's global-pointer shape into thousands of `unsafe`
  blocks. Goal G3 would then mean refactoring the whole tree a second time.

Kreimeier's pre-release cleanup of the source added a few bugs. Chocolate Doom
documents the corrections. The port follows Chocolate Doom's corrected
behaviour and credits it in `CREDITS.md` (§15.3, OQ-4).

### Crate layout

```
phobos/
├── crates/
│   ├── phobos-fixed/    # Fixed (16.16), Angle (BAM), trig tables, P_Random
│   ├── phobos-wad/      # WAD reader, IWAD identification, lump namespace
│   ├── phobos-data/     # Map, texture, patch, flat, sprite, sound decoding
│   ├── phobos-sim/      # Deterministic game simulation (#![forbid(unsafe_code)])
│   ├── phobos-game/     # Game flow: modes, missions, demos, saves, cheats
│   ├── phobos-render/   # Software renderer to an 8-bit framebuffer
│   ├── phobos-ui/       # Status bar, HUD, automap, menus, intermission, finales
│   ├── phobos-audio/    # Mixer, SFX, MUS sequencer, OPL backend
│   ├── phobos-gpu/      # wgpu presenter (M3) and enhanced renderer (M8)
│   ├── phobos-app/      # winit event loop, CLI, config, input mapping
│   └── phobos-tools/    # `phobos-tools wad|demo|map` inspection and CI checks
└── fuzz/                # cargo-fuzz targets
```

Dependency rules, enforced by `cargo deny` bans and checked at review:

- `phobos-sim` depends only on `phobos-fixed` and `phobos-data`.
- No renderer depends on `phobos-wad`. Renderers receive decoded resources.
- Only `phobos-app` and `phobos-gpu` depend on windowing or GPU crates.

### State ownership

The C engine keeps its state in mutable globals. Phobos keeps it in owned
structs passed down the call tree:

- `Level` owns the map geometry, sectors, lines, blockmap, and reject table.
- `World` owns the `Level`, the thinker arena, the players, and the random
  indices. A tic is `fn tick(&mut World, &[TicCmd; MAXPLAYERS])`.
- **Thinkers** live in an arena addressed by generational handles. An
  index-linked list inside the arena keeps the vanilla insertion and iteration
  order. That order decides the random-call sequence, so demo sync depends on
  it. Pointer fields such as `mobj_t.target` and `tracer` become
  `Option<MobjHandle>`. A handle to a removed thing resolves to `None`. In C,
  the same case reads freed memory that the zone allocator had not yet reused.
  The port reproduces that observable result explicitly.
- Sector thing lists and blockmap links keep their vanilla link order, for the
  same reason.
- The arithmetic overflows that vanilla relies on use `wrapping_*`
  operations explicitly. `overflow-checks = true` stays on in test profiles so
  that any accidental overflow fails.

### Concurrency model (Standard §3.2)

| Work | Model | Decision and trade-off |
|------|-------|------------------------|
| Simulation tic | **Serial**, one thread | Determinism needs one exact order of random calls and thinker updates. A tic costs well under 1 ms (budget below), so by Amdahl's law parallelism could save almost nothing and would risk desync. Serial is chosen and documented here |
| Event loop and presentation | winit main thread | winit requires the main thread on several platforms. Frames go from the game thread to the presenter through a triple buffer, so neither side waits for the other |
| Software renderer | Serial at 320×200. Optional banded parallel drawing with Rayon at 2× and 4× scale (R-056) | BSP traversal is serial by nature. Column and span drawing for screen bands can run in parallel once clipping is done. Adopted only if a benchmark shows a gain over serial at that scale |
| Audio mixing and OPL synthesis | cpal callback thread | The simulation sends sound commands over a bounded lock-free SPSC queue and never blocks. If the queue is full, the oldest command is dropped and counted |
| Level load (texture compositing, patch decoding) | Rayon over textures | Embarrassingly parallel. Kept only if it beats serial on the reference machine |
| Demo corpus in CI | One process per demo | Shared-nothing parallelism over the corpus |

### Rendering

- **Software renderer (reference, M3).** A port of `r_*.c` that writes
  palette indices to a 320×200 framebuffer (or 2× / 4× scale, optional). Colour
  maps, light diminishing, fuzz, sky, masked mid-textures, and sprite clipping
  follow vanilla. Vanilla render limits (visplanes, drawsegs, vissprites) are
  enforced by default and overflow into a logged, non-fatal fallback. Vanilla
  crashes in this case; Phobos does not. A limit-removing mode is optional
  (R-085).
- **Presenter (M3).** The indexed frame is uploaded to a wgpu texture. A small
  WGSL shader expands it through `PLAYPAL` and applies palette flashes. The
  output is scaled to 4:3 aspect, with integer scaling as an option. If no GPU
  adapter is available, a `softbuffer` CPU path is used.
- **Enhanced renderer (M8).** A wgpu renderer reads the same
  `RenderSnapshot`. It stores static level geometry in GPU buffers, draws
  sprites with instancing, and applies colour-map lighting in the shader.
  Dynamic lights, post-processing, and ray-traced effects are optional. Its
  output is checked against the software renderer on golden scenes.

### Audio

- **Sound effects.** DMX `DS*` lumps are decoded and resampled to the output
  rate. There are 8 channels, using vanilla's priority and channel-stealing
  rules, distance attenuation, and stereo separation (`s_sound.c`).
- **Music.** The `MUS` sequencer drives an OPL3 emulator voiced from the IWAD's
  `GENMIDI` bank, the vanilla sound. Candidate crate: `nuked-opl3`, a pure-Rust
  port. It is at 0.1.0 and needs the §24 audit before adoption. SoundFont
  MIDI is optional.
- **Degraded mode.** If no audio device is available, the game runs silent
  and logs one warning.

### Platform and files

- Windowing and input use winit. Gamepads use gilrs (optional).
- File locations follow XDG. Configuration goes in
  `$XDG_CONFIG_HOME/phobos/phobos.toml`; saves and demos in
  `$XDG_DATA_HOME/phobos/`.
- IWAD search order: `-iwad`, `DOOMWADDIR`, `DOOMWADPATH`, the XDG data
  directories plus `doom/`, then the current directory.
- No network access in any build (§9).
- The engine uses no fonts of its own except on the no-IWAD screen and the
  debug overlay. Those use bundled Share Tech Mono / Inconsolata (OFL, §12) and
  the `steelbore` theme (§11, §11.6). In-game menus and HUD use IWAD graphics.

## Interfaces

### Command line

Vanilla single-dash arguments are accepted as written. GNU-style long forms are
accepted alongside them. `--help` and `--version` follow
`spacecraft-cli-standard`, and `--version` carries the §15 attribution.

| Argument | Purpose |
|----------|---------|
| `-iwad <file>` | Select IWAD |
| `-file <files…>` | Load PWADs with vanilla override rules |
| `-deh <files…>` | Load DeHackEd patches (M7) |
| `-warp <e> <m>` / `-warp <map>` | Start at a map |
| `-skill <1–5>` | Skill level |
| `-nomonsters`, `-fast`, `-respawn` | Gameplay switches |
| `-gameversion <v>` | `1.9`, `ultimate`, `final`, `final2` |
| `-record <file>`, `-playdemo <file>`, `-timedemo <file>` | Demo record, playback, benchmark |
| `-statdump <file>` | Write level statistics in Chocolate Doom format |
| `-width`, `-height`, `-fullscreen`, `-window` | Display |
| `-config <file>` | Alternate configuration |

`phobos-tools` is non-interactive and supports `--json`: `wad ls`,
`wad identify`, `demo check`, `demo diff`, `map stats`. CI and agents use it.

### Configuration

TOML with sections `[video]`, `[audio]`, `[input]`, `[gameplay]`,
`[compat]`, `[debug]`. An unknown key is warned about and ignored. A missing
or invalid value falls back to the default with a warning. A malformed file
never stops the game from starting.

### Data formats read

WAD (IWAD, PWAD), vanilla demo `.lmp` (version byte 109 and the earlier
versions the selected game version implies), DeHackEd `.deh` / `DEHACKED`
lump (M7), Phobos native saves (versioned), and optionally vanilla `.dsg`
saves. All are untrusted input.

## Verification

Category D owes a smoke test (§21.2). Fidelity is the product, so Phobos sets a
higher bar for itself. This is a choice, not a Standard obligation.

| Layer | Evidence | Runs in |
|-------|----------|---------|
| Fixed point, tables, random | Unit tests against values from the C source; property tests on wrapping | CI |
| WAD, map, patch, demo, save, DeHackEd parsers | Unit tests plus `cargo-fuzz` targets; no panic, no unbounded allocation | CI (time-boxed fuzz) |
| Simulation | Per-tic `World` checksum; golden traces for scripted sequences (movement, doors, weapons, monsters) | CI |
| Demo fidelity | Built-in demos and a demo corpus: `-statdump` output compared to Chocolate Doom built out of tree from a pinned commit (statcheck method) | CI on Freedoom and `doom1.wad`; locally on commercial IWADs |
| Renderer | Headless render-to-PNG golden frames at fixed tics | CI |
| GPU | wgpu validation on; headless adapter where available | CI where possible |
| Performance | Criterion benchmarks; `-timedemo` | CI sample, full run before releases |

### Performance targets (should)

Category D has no §22 budget obligation. These targets guide the work and gate
optimisation decisions. The reference machine is still to be chosen (OQ-3).

| Measure | Target |
|---------|--------|
| Tic simulation, 99th percentile, MAP30 (Doom II) | ≤ 1 ms |
| Software frame, 320×200 | ≤ 1 ms |
| Software frame, 4× scale (1280×800) | ≤ 6 ms |
| Map load, largest commercial map | ≤ 250 ms |
| Resident memory, any commercial IWAD | ≤ 256 MiB |
| Added input latency | No sleep beyond vsync and frame pacing |

## Standard compliance map

| § | Applies | How Phobos satisfies it |
|---|---------|------------------------|
| §2 | Yes | Codename Phobos (Mars's moon; astronomy domain), approved 2026-10-27 |
| §3.1 | Yes | Rust; `#![forbid(unsafe_code)]` in `phobos-fixed`, `phobos-wad`, `phobos-data`, `phobos-sim`, `phobos-game`; typed errors; no `unwrap` on input |
| §3.2 | Yes | Concurrency table above; serial simulation documented; release flags noted in `Cargo.toml` |
| §3.3 | Partly | No network, no privilege, no crypto; PQC is N/A |
| §4 | Yes | GPL-3.0-or-later; inherited GPL-2.0-or-later notices kept verbatim; REUSE; `LICENSES/` |
| §5.2 | Yes | Posture files (R-002) |
| §5.5 | Yes | `packaging/guix.scm`, `default.nix`, `PKGBUILD` (R-079) |
| §5.7 | Yes | `AGENTS.md` + `CLAUDE.md` (R-003) |
| §6.3, §6.4 | Yes | Signed commits; no upstream contributions to Chocolate Doom or others without per-contribution approval |
| §6.5 | Yes | `.gitattributes`, `.editorconfig` (R-005) |
| §8 | Yes | `doc/phobos.texi` (R-078) |
| §9 | Yes | No network, no telemetry, local files only, no third-party subresources at run time |
| §10, §18 | No | Game carve-out §18.5. Adopted anyway from §18.5.2: remappable controls, and no capture of `Insert` or `CapsLock` |
| §11, §11.6 | Chrome only | No-IWAD screen, debug overlay, launcher messages. The game world uses `PLAYPAL` |
| §12 | Yes | Bundled OFL fonts for chrome text |
| §13 | Tailored | Menus are IWAD-native vanilla menus, not a component system; recorded in the README register |
| §14 | Yes | ISO 8601 UTC in save metadata, logs, statdump headers |
| §15 | Yes | `--version`, README, `CREDITS.md` (id Software, Chocolate Doom, room4doom if code is adapted, Freedoom) |
| §17 | Yes | This file follows §17.5 |
| §19 | Yes | Category D declared in README, AGENTS.md, PROJECTS.md |
| §20–§23, §25 | None owed | Category D |
| §24 | Audit tooling | `cargo audit`, `cargo deny` in CI; SBOM per release (R-077) |
| §26 | Yes | `SECURITY.md` (R-010) |

## M0 — Foundation and compliance

- [ ] R-001 Cargo workspace with `rust-toolchain.toml` and committed `Cargo.lock`
- [ ] R-002 Posture files: `README.md` (posture, game declaration, Category D, conformance claim), `NOTICE.md`, `CONTRIBUTING.md`, `LICENSE`, `LICENSES/`
- [ ] R-003 `AGENTS.md` and `CLAUDE.md` per §5.7
- [ ] R-004 REUSE headers on every file; `reuse lint` gates CI
- [ ] R-005 `.gitattributes` and `.editorconfig` per §6.5; CI rejects CR bytes
- [ ] R-006 CI runs `cargo fmt --check`, `cargo clippy --all-targets --all-features --locked -- -D warnings`, tests, doc tests, `cargo audit`, `cargo deny`
- [ ] R-007 Codename Phobos registered in `PROJECTS.md`, the §2.1 registry, and the §18.5 games registry
- [ ] R-008 "Or later" grant verified against the GPL source release headers and recorded in `NOTICE.md`
- [ ] R-009 `CREDITS.md` listing id Software, Chocolate Doom, Freedoom, and prior Rust art consulted
- [ ] R-010 `SECURITY.md` with reporting channel and support window
- [ ] R-011 Chocolate Doom oracle built reproducibly out of tree (Nix, pinned commit), used by CI only
- [ ] R-012 CI fetches Freedoom and shareware `doom1.wad` by pinned SHA-256

## M1 — WAD and resource layer

- [ ] R-013 WAD reader with checked bounds; malformed input yields a typed error and never panics
- [ ] R-014 IWAD identification for all eight target IWADs by lump signature
- [ ] R-015 IWAD search in the documented order
- [ ] R-016 PWAD loading with vanilla `-file` override semantics
- [ ] R-017 Map lumps decoded: THINGS, LINEDEFS, SIDEDEFS, VERTEXES, SEGS, SSECTORS, NODES, SECTORS, REJECT, BLOCKMAP
- [ ] R-018 Graphics decoded: PLAYPAL, COLORMAP, PNAMES, TEXTURE1/2, patches, flats, sprites, ENDOOM
- [ ] R-019 Sound lumps decoded: DS*, DP*, D_* (MUS), GENMIDI
- [ ] R-020 Fuzz targets for WAD, map, patch, and texture decoding
- [ ] R-021 `phobos-tools wad ls` and `wad identify` with `--json`
- [ ] R-022 PWAD sprite and flat merging (`-merge`) (optional)

## M2 — Deterministic simulation core

- [ ] R-023 16.16 `Fixed` and BAM `Angle` with bit-exact `FixedMul` and `FixedDiv`, including the overflow branch
- [ ] R-024 `finesine`, `finetangent`, `tantoangle` tables carried verbatim
- [ ] R-025 `P_Random` and `M_Random` from the 256-entry table with separate indices
- [ ] R-026 Fixed 35 Hz tic driven by `TicCmd`; simulation crates contain no floating point and no clock reads (enforced by lint)
- [ ] R-027 Thinker arena with generational handles that preserves vanilla iteration order
- [ ] R-028 `mobjinfo`, `states`, and sprite tables for the full Doom II set
- [ ] R-029 Player movement, collision, sliding, and use-line traversal
- [ ] R-030 All nine weapons and player sprite states, including the super shotgun
- [ ] R-031 Monster action functions for all Doom and Doom II monsters
- [ ] R-032 Line and sector specials: doors, plats, floors, ceilings, crushers, lights, switches, teleports, donuts, stairs
- [ ] R-033 Level triggers for E1M8, E2M8, E3M8, E4M6, E4M8, MAP07, and the MAP30 boss brain
- [ ] R-034 Game mode and mission selection with episode and map progression, including secret exits to MAP31 and MAP32
- [ ] R-035 `-gameversion` emulation of 1.9, ultimate, final, and final2 differences
- [ ] R-036 Skills 1–5 with `-nomonsters`, `-fast`, `-respawn`
- [ ] R-037 Vanilla cheat codes
- [ ] R-038 Per-tic `World` checksum exposed to tests and tools
- [ ] R-039 Headless simulation run with no window, GPU, or audio
- [ ] R-040 Emulation of the spechit and intercepts overflows as Chocolate Doom does (optional)

## M3 — Classic renderer and front end

- [ ] R-041 Software renderer port drawing palette indices to a 320×200 framebuffer
- [ ] R-042 Colour-map lighting, light diminishing, fuzz, sky per game mode, masked textures
- [ ] R-043 Vanilla render limits overflow into a logged, non-fatal fallback
- [ ] R-044 wgpu presenter with `PLAYPAL` expansion, palette flashes, and 4:3 scaling
- [ ] R-045 `softbuffer` presenter used when no GPU adapter is available
- [ ] R-046 Status bar and HUD, with level names for Doom, Doom II, TNT, and Plutonia
- [ ] R-047 Automap with follow, zoom, grid, and marks
- [ ] R-048 Menus: new game, episode (Doom only), skill, options, load, save, quit with ENDOOM
- [ ] R-049 Intermission: Doom animated maps and the Doom II background
- [ ] R-050 Finales: episode texts, Doom II, TNT, and Plutonia text screens, cast call, bunny scroller, E4 ending
- [ ] R-051 Title and attract loop per game mode
- [ ] R-052 Keyboard and mouse input with bindings remappable in the configuration
- [ ] R-053 TOML configuration and XDG file locations
- [ ] R-054 No-IWAD screen explaining how to supply an IWAD, using the `steelbore` theme
- [ ] R-055 Headless render-to-PNG golden frames in CI
- [ ] R-056 2× and 4× internal scale with banded parallel drawing (optional)
- [ ] R-057 Uncapped frame rate with interpolation between tics (optional)
- [ ] R-058 Gamepad input (optional)

## M4 — Sound and music

- [ ] R-059 Audio output on a dedicated thread fed by a bounded lock-free queue that never blocks the tic
- [ ] R-060 Sound effects with vanilla channel priority, attenuation, and stereo separation
- [ ] R-061 MUS parser and sequencer
- [ ] R-062 OPL music voiced from `GENMIDI`, as the default backend
- [ ] R-063 Silent degraded mode with a single warning when no audio device is present
- [ ] R-064 Separate sound-effect and music volume settings
- [ ] R-065 SoundFont MIDI backend (optional)
- [ ] R-066 PC speaker sound effects (optional)

## M5 — Demos, saves and fidelity gate

- [ ] R-067 Vanilla `.lmp` demo playback
- [ ] R-068 Vanilla `.lmp` demo recording
- [ ] R-069 `-timedemo` reporting tics, wall time, and frame rate
- [ ] R-070 `-statdump` output in Chocolate Doom format
- [ ] R-071 Built-in demos of all eight IWADs stay in sync (CI for free IWADs, local run for commercial)
- [ ] R-072 CI differential check: `-statdump` matches the Chocolate Doom oracle on the free-IWAD demo corpus
- [ ] R-073 `phobos-tools demo diff` reports the first divergent tic
- [ ] R-074 Native versioned savegame whose reload reproduces identical per-tic checksums
- [ ] R-075 A corrupt or incompatible save is refused with a message and the game continues
- [ ] R-076 Vanilla `.dsg` savegame loading (optional)

## M6 — Release engineering v1.0

- [ ] R-077 SBOM generated from `Cargo.lock` in the release run and shipped with a checksum
- [ ] R-078 Texinfo manual `doc/phobos.texi` building to Info, HTML, and PDF
- [ ] R-079 `packaging/guix.scm`, `packaging/default.nix`, `packaging/PKGBUILD` with `install-info`
- [ ] R-080 `--help` and `--version` per `spacecraft-cli-standard` with §15 attribution
- [ ] R-081 README compliance section with the tailoring register (§13)
- [ ] R-082 Release profile flags (LTO, `codegen-units = 1`) noted in `Cargo.toml`, benchmarked before and after
- [ ] R-083 Installation verified from each package definition in a clean environment
- [ ] R-084 Signed release tag
- [ ] R-085 Limit-removing render mode (optional)
- [ ] R-086 macOS and Windows builds (optional)

## M7 — Vanilla modding extensions

- [ ] R-087 DeHackEd patches loaded from `-deh` files
- [ ] R-088 DeHackEd patches loaded from a `DEHACKED` lump
- [ ] R-089 DeHackEd parser fuzz target
- [ ] R-090 UMAPINFO support (optional)

## M8 — Enhanced GPU renderer

- [ ] R-091 wgpu renderer consuming the read-only `RenderSnapshot`, selectable at run time
- [ ] R-092 Static level geometry in GPU buffers and instanced sprites
- [ ] R-093 Colour-map lighting reproduced in WGSL
- [ ] R-094 Golden-scene parity check against the software renderer within a declared tolerance
- [ ] R-095 Device loss returns to the software renderer without ending the game
- [ ] R-096 Dynamic lights and post-processing (optional)
- [ ] R-097 Rust GPU shader path (optional)
- [ ] R-098 Ray-traced ambient occlusion and shadows (optional)
- [ ] R-099 WebGPU / WASM build (optional)

## Risks

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Demo desync from subtle order or overflow differences | High | High | Per-tic checksums, Chocolate Doom oracle, `demo diff` first-divergence tool, transliteration that keeps call order |
| Thinker or link order lost when pointers become handles | High | Medium | Index-linked lists in the arena; golden traces on monster-heavy maps |
| Vanilla undefined behaviour (freed `target`, overflows) is relied on by demos | Medium | Medium | Emulate the observable result explicitly; R-040 optional overflow emulation |
| Commercial IWADs absent from CI | Medium | Certain | Freedoom and shareware in CI; a scripted local acceptance run before each release |
| Young OPL crate | Medium | Medium | §24 audit; fallback is porting Nuked OPL3 in tree, keeping its LGPL-2.1-or-later notice (§4.2) |
| wgpu API churn | Low | High | Presenter stays thin; `softbuffer` fallback; versions pinned |
| Scope creep toward Boom or ZDoom | High | High | NG-1; DeHackEd is the only extension format |
| Trademark complaint | Medium | Low | Codename, no assets, no logos, disclaimer |

## Open questions

| ID | Question | Recommendation |
|----|----------|----------------|
| OQ-1 | ~~Approve codename **Phobos**?~~ | **Resolved 2026-10-27:** approved by the maintainer |
| OQ-2 | Are macOS and Windows required for v1.0? | Optional: winit and wgpu make them cheap, but CI cost and a Personal posture argue against requiring them |
| OQ-3 | Which reference machine anchors the performance targets? | The maintainer's main Steelbore OS workstation, recorded with CPU, GPU, and RAM |
| OQ-4 | May the port adapt Chocolate Doom code (GPL-2.0-or-later), or only follow its documented behaviour? | Allow it with file-level credit; the licences are compatible |
| OQ-5 | Default music backend: OPL or SoundFont? | OPL (vanilla sound); SoundFont optional |
| OQ-6 | Is vanilla `.dsg` save loading worth the effort? | Keep it optional; the native format is required |

## References

- *RustDoom* PRD v1.0 (2026-07-23), `RustDoom.html`.
- *Doom-to-Rust Porting Reference* v1.0 (2026-07-23), `doomrustport.texi`.
- id Software, [id-Software/DOOM](https://github.com/id-Software/DOOM); [Doom source code history](https://doomwiki.org/wiki/Doom_source_code).
- [Chocolate Doom](https://github.com/chocolate-doom/chocolate-doom) and its [statcheck](https://github.com/chocolate-doom/statcheck) demo-regression method.
- Prior Rust art: [room4doom](https://github.com/flukejones/room4doom), [iron-doom](https://github.com/Henrique194/iron-doom), [rust-doom](https://github.com/cristicbz/rust-doom).
- [Freedoom](https://freedoom.github.io/).
- The Steelbore Standard v2.14, §2, §3, §4, §9, §11, §17.5, §18.5, §19.
