# Phobos — TODO

The next concrete actions, in order. This list covers the open decisions, M0,
and the start of M1 from [`PLAN.md`](PLAN.md). It is refreshed when M1 closes:
finished items stay ticked, and new items take new identifiers. Updated
2026-10-27.

- [x] T-001 Decide whether the working directory `doom-rs/` is renamed to `phobos/` (renamed 2026-10-27)
- [ ] T-002 Create the GitHub repository `Spacecraft-Software/Phobos` and protect `main`
- [ ] T-003 Run `git init`, confirm SSH commit signing works, push an initial signed commit
- [ ] T-004 Move the source documents (`RustDoom.html`, `doomrustport.*`) to `doc/history/`
- [ ] T-005 Move `phobos-prd.texi` and `phobos-prd.html` to `doc/`
- [ ] T-006 Copy `LICENSE`, `LICENSES/`, `NOTICE.md`, `CONTRIBUTING.md` from `/spacecraft-software/license/`
- [ ] T-007 Write `README.md` with the posture section, game declaration, Category D, and conformance claim
- [ ] T-008 Write `CREDITS.md` and `SECURITY.md`
- [ ] T-009 Write `AGENTS.md` and a `CLAUDE.md` that imports it
- [ ] T-010 Add `.gitattributes`, `.editorconfig`, `REUSE.toml`; make `reuse lint` pass
- [ ] T-011 Create the Cargo workspace with the eleven empty crates
- [ ] T-012 Add `rust-toolchain.toml` and `fuzz/rust-toolchain.toml`
- [ ] T-013 Add `[workspace.lints]`, per-crate `forbid(unsafe_code)`, and the `clippy.toml` bans
- [ ] T-014 Write `flake.nix` with the dev shell
- [ ] T-015 Add `chocolate-doom` to the flake and find the flags for a headless statdump run
- [ ] T-016 Add fixed-output derivations for Freedoom and `doom1.wad` with SHA-256 pins
- [ ] T-017 Write `deny.toml` with the licence allow-list and crate dependency bans
- [ ] T-018 Write the CI workflow with every gate and the CR-byte check
- [ ] T-019 Add the CI job that runs the oracle on `doom1.wad` DEMO1
- [ ] T-020 Check the 1999 GPL release headers for the "or later" grant and record it in `NOTICE.md`
- [ ] T-021 Open the `PROJECTS.md` pull request registering Phobos as a Category D game
- [ ] T-022 Open the Standard pull request adding Phobos to the §2.1 and §18.5 registries
- [ ] T-023 Open the matching Construct pull request for the `spacecraft-steelbore-standard` skill
- [ ] T-024 List a free-IWAD demo corpus and check that its demos may be fetched in CI
- [ ] T-025 Decide OQ-2: whether macOS and Windows are required for v1.0
- [ ] T-026 Decide OQ-3 and record the reference machine's CPU, GPU, and RAM
- [ ] T-027 Decide OQ-4: whether Chocolate Doom code may be adapted
- [ ] T-028 Decide OQ-5 and OQ-6: default music backend and vanilla save loading
- [ ] T-029 Write `WadError` and the WAD header and directory parser with unit tests
- [ ] T-030 Write the lump namespace with last-wins lookup and marker ranges
- [ ] T-031 Collect lump signatures for the eight target IWADs and write the identification table
- [ ] T-032 Add the first fuzz target for the WAD directory parser
- [ ] T-033 Add `phobos-tools wad ls --json`
- [ ] T-034 Tick the finished `P-` and `R-` items and refresh this list at the M1 exit
