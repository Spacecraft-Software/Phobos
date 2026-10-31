<!--
SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
SPDX-License-Identifier: CC-BY-SA-4.0
-->

# AGENTS.md

Guidance for any coding agent working in this repository (Standard §5.7).
`CLAUDE.md` imports this file and adds only Claude-Code-specific notes.

## What this is

Phobos is a memory-safe Rust port of the id Tech 1 engine. It plays *Doom*,
*The Ultimate Doom*, *Doom II* and *Final Doom* from user-supplied IWADs.

- **Registered game** (Standard §18.5): §18 and §10 do not apply.
- **Assurance category D** (Standard §19.1).
- **Requirements:** [`PRD.md`](PRD.md). **Order of work:**
  [`PLAN.md`](PLAN.md). **Next actions:** [`TODO.md`](TODO.md). Tick items
  in those files when the work is merged and its checks pass, never earlier.

## Gates (run all before declaring a change done)

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked
cargo deny check            # from `nix develop`
cargo audit
reuse lint
tools/oracle-smoke.sh       # from `nix develop`; needs Chocolate Doom and Freedoom
```

## Invariants (a change that breaks one is rejected)

- **No `unsafe`.** `unsafe_code = "forbid"` is set for the whole workspace.
- **Determinism.** `phobos-fixed`, `phobos-sim` and `phobos-game`:
  - contain no floating point (`clippy::float_arithmetic` is denied);
  - read no clock (`clippy.toml` in each crate bans `std::time`);
  - run on one thread; the thinker update order decides the random sequence.
- **Vanilla arithmetic.** Overflow that vanilla relies on is written with
  `wrapping_*`. Overflow checks stay on in dev and test profiles.
- **Crate direction** (enforced by `deny.toml`):
  - `phobos-sim` depends only on `phobos-fixed` and `phobos-data`;
  - only `phobos-game`, `phobos-app` and `phobos-tools` open WADs;
  - only `phobos-app` and `phobos-gpu` touch windowing or GPU crates.
- **Presentation never feeds back.** Renderers take `&World`, never
  `&mut World`.
- **Untrusted input.** WAD, demo, save and DeHackEd bytes are parsed with
  checked bounds into typed errors. No `unwrap`, `expect` or `panic!` on
  input (all three are denied outside tests).
- **Errors.** Library crates return canonical error structs
  (microsoft-rust-guidelines M-ERRORS-CANONICAL-STRUCTS); `phobos-app` uses
  `anyhow` with context. Error types are `Send + Sync + 'static`.
- **Transliterated code.** A file ported from the id source names its C
  source file in its module doc, keeps the original function order, and keeps
  `Copyright (C) 1993-1996 id Software, Inc.` in its SPDX header. **No such
  file may be committed until `NOTICE.md` records the maintainer's ruling on
  the licence grant (P-013).**
- **No network access** in any Phobos build (Standard §9).

## Test data

- CI uses only Freedoom, from the flake (`nix develop` exports
  `PHOBOS_FREEDOOM_DIR`).
- Commercial IWADs and the shareware `doom1.wad` are local only: point
  `PHOBOS_IWAD_DIR` at a directory that holds them. Tests that need them skip
  when the variable is unset.
- Never commit an IWAD, a PWAD, or a demo that is not free to redistribute.

## Oracle

See [`doc/oracle.md`](doc/oracle.md). Chocolate Doom runs out of tree only.

## Workflow

- Feature branch, then pull request, then squash-merge. Never push to `main`.
- Commits are signed (Standard §6.3) and dated in UTC.
- Push over HTTPS (Standard §6.4).
- Dates in documents are ISO 8601 (Standard §14).
