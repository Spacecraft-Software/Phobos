<!--
SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
SPDX-License-Identifier: CC-BY-SA-4.0
-->

# CLAUDE.md

@AGENTS.md

> Record project knowledge in `AGENTS.md`, not here. This file holds only
> Claude-Code-only context (Standard §5.7).

## Skills to load

1. `spacecraft-steelbore-standard` before any artifact.
2. `microsoft-rust-guidelines` before any `.rs` file; `spacecraft-rust-guidelines`
   on top for the audio thread, the frame handoff and Rayon work.
3. `spacecraft-nix-guidelines` for `flake.nix` and `packaging/`.
4. `spacecraft-texinfo-document` for `doc/*.texi`.
5. `spacecraft-cli-shell` and `spacecraft-cli-preference` before any shell
   command; `spacecraft-missing-pkg` when a tool is absent.
