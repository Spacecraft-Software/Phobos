<!--
SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
SPDX-License-Identifier: CC-BY-SA-4.0
-->

# Phobos

A memory-safe Rust port of the id Tech 1 engine. Phobos plays *Doom*,
*The Ultimate Doom*, *Doom II* and *Final Doom* (*TNT: Evilution*, *The
Plutonia Experiment*) from IWADs you supply, and the free Freedoom IWADs.

> **Status: Planning, then M0.** The workspace, its gates and the test oracle
> exist; nothing is playable yet. See [`PRD.md`](PRD.md) for the requirements,
> [`PLAN.md`](PLAN.md) for the order of the work and [`TODO.md`](TODO.md) for
> the next actions.

*Doom* is a trademark of id Software / ZeniMax Media. Phobos is an independent
engine that reads Doom data files; it ships no id Software assets and claims
no endorsement. See [`NOTICE.md`](NOTICE.md).

## Layout

| Path | Contents |
|------|----------|
| `crates/phobos-fixed` | 16.16 fixed point, binary angles, trig and random tables |
| `crates/phobos-wad` | WAD reader, IWAD identification, lump namespace |
| `crates/phobos-data` | Map, graphics and sound lump decoders |
| `crates/phobos-sim` | The deterministic simulation |
| `crates/phobos-game` | Game flow: modes, missions, demos, saves, cheats |
| `crates/phobos-render` | The software renderer |
| `crates/phobos-ui` | Status bar, HUD, automap, menus, intermission, finales |
| `crates/phobos-audio` | Mixer, MUS sequencer, OPL backend |
| `crates/phobos-gpu` | wgpu presenter, later the enhanced renderer |
| `crates/phobos-app` | The `phobos` executable |
| `crates/phobos-tools` | The `phobos-tools` executable |
| `doc/` | Manuals and design notes, including [`doc/oracle.md`](doc/oracle.md) |
| `tools/` | Scripts used by CI |

## Building

The toolchain is pinned in `rust-toolchain.toml`; rustup installs it on first
use. The Nix dev shell adds the audit tools, the Chocolate Doom oracle and the
Freedoom IWADs:

```sh
nix develop
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo deny check
tools/oracle-smoke.sh
```

## Project Posture

Phobos is a **personal hobby project** (Standard §5.1): no warranty, no
support commitment, contributions welcome at the maintainer's discretion. See
[`NOTICE.md`](NOTICE.md) and [`CONTRIBUTING.md`](CONTRIBUTING.md).

**Game declaration (Standard §18.5.1).** Phobos is a **game**. It is exempt
from §18 (accessibility mode) and §10 (key bindings). It adopts two §18.5.2
recommendations anyway: controls are remappable, and it never captures the
screen-reader keys `Insert` and `CapsLock`.

**Assurance category (Standard §19): D — Incidental.**

Conforms to The Steelbore Standard v2.15 — Category D, tailored (§13; see
"Compliance" below).

## Compliance

The tailoring register (Standard §19.5). A clause not listed is applied in
full.

| Clause | Status | Justification | Date |
|--------|--------|---------------|------|
| §13 | not-applicable | The game draws its menus and HUD from IWAD graphics; there is no widget component system to declare | 2026-10-31 |
| §8 | tailored | The Texinfo manual `doc/phobos.texi` lands with M6 (P-080); before then the PRD's Texinfo rendering is the only manual | 2026-10-31 |
| §5.5 | tailored | Packaging definitions land with M6 (P-081); there is no release to package before then | 2026-10-31 |

## License

Code: `GPL-3.0-or-later` ([`LICENSE`](LICENSE)). Documents: `CC-BY-SA-4.0`.
Code transliterated from the id Software source keeps its upstream notice; its
licence status is recorded in [`NOTICE.md`](NOTICE.md).

## Maintainer

Mohamed Hammad &lt;Mohamed.Hammad@SpacecraftSoftware.org&gt;
<https://Phobos.SpacecraftSoftware.org/>
