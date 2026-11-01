# Phobos — TODO

The next concrete actions, in order. This list covers the open decisions, M0,
and the start of M1 from [`PLAN.md`](PLAN.md). It is refreshed when M1 closes:
finished items stay ticked, and new items take new identifiers. Updated
2026-10-27.

- [x] T-001 Decide whether the working directory `doom-rs/` is renamed to `phobos/` (renamed 2026-10-27)
- [ ] T-002 Create the GitHub repository `Spacecraft-Software/Phobos` and protect `main`
- [x] T-003 Run `git init`, confirm SSH commit signing works, push an initial signed commit
- [x] T-004 Move the source documents (`RustDoom.html`, `doomrustport.*`) to `doc/history/`
- [x] T-005 Move `phobos-prd.texi` and `phobos-prd.html` to `doc/`
- [x] T-006 Copy `LICENSE`, `LICENSES/`, `NOTICE.md`, `CONTRIBUTING.md` from `/spacecraft-software/license/`
- [x] T-007 Write `README.md` with the posture section, game declaration, Category D, and conformance claim
- [x] T-008 Write `CREDITS.md` and `SECURITY.md`
- [x] T-009 Write `AGENTS.md` and a `CLAUDE.md` that imports it
- [x] T-010 Add `.gitattributes`, `.editorconfig`, `REUSE.toml`; make `reuse lint` pass
- [x] T-011 Create the Cargo workspace with the eleven empty crates
- [x] T-012 Add `rust-toolchain.toml` and `fuzz/rust-toolchain.toml`
- [x] T-013 Add `[workspace.lints]`, per-crate `forbid(unsafe_code)`, and the `clippy.toml` bans
- [x] T-014 Write `flake.nix` with the dev shell
- [x] T-015 Add `chocolate-doom` to the flake and find the flags for a headless statdump run
- [x] T-016 Pin Freedoom through the flake with SHA-256 checks (`doom1.wad`: no stable source, local only)
- [x] T-017 Write `deny.toml` with the licence allow-list and crate dependency bans
- [x] T-018 Write the CI workflow with every gate and the CR-byte check
- [x] T-019 Add the CI job that runs the oracle on `freedoom1.wad` DEMO1 (`doom1.wad` is local only)
- [x] T-020 Check the 1999 GPL release headers for the "or later" grant and record it in `NOTICE.md` (recorded; primary source not reachable, ruling open)
- [x] T-021 Open the `PROJECTS.md` pull request registering Phobos as a Category D game
- [x] T-022 Open the Standard pull request adding Phobos to the §15.1 and §18.5 registries
- [x] T-023 Open the matching Construct pull request for the `spacecraft-steelbore-standard` skill
- [ ] T-024 List a free-IWAD demo corpus and check that its demos may be fetched in CI
- [ ] T-025 Decide OQ-2: whether macOS and Windows are required for v1.0
- [ ] T-026 Decide OQ-3 and record the reference machine's CPU, GPU, and RAM
- [ ] T-027 Decide OQ-4: whether Chocolate Doom code may be adapted
- [ ] T-028 Decide OQ-5 and OQ-6: default music backend and vanilla save loading
- [x] T-029 Write `WadError` and the WAD header and directory parser with unit tests
- [x] T-030 Write the lump namespace with last-wins lookup and marker ranges
- [x] T-031 Collect lump signatures for the eight target IWADs and write the identification table
- [x] T-032 Add the first fuzz target for the WAD directory parser
- [x] T-033 Add `phobos-tools wad ls --json`
- [ ] T-034 Tick the finished `P-` and `R-` items and refresh this list at the M1 exit
