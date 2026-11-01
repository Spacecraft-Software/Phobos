<!--
SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
SPDX-License-Identifier: CC-BY-SA-4.0
-->

# Benchmark baselines

Figures recorded before any optimisation (Standard §3.2). Each row names the
machine, the input and the command, so a later figure can be compared with it.

| Date | Benchmark | Input | Machine | Median |
|------|-----------|-------|---------|--------|
| 2026-11-01 | `phobos-wad` `parse_directory` | `freedoom2.wad` (Freedoom 0.13.0, 28 MiB, 3163 lumps) | Intel Core i7-8665U, 8 threads | 2.90 ms |
| 2026-11-01 | `phobos-wad` `decode_all` (every map, texture, patch, flat; serial) | same | same | 26.7 ms |

Command: `PHOBOS_FREEDOOM_DIR=… cargo bench -p phobos-wad --bench load -- --measurement-time 5`.

The PRD target for a map load is 250 ms; decoding the whole IWAD serially is
well inside it, so P-043 keeps level loading serial unless the renderer's
texture compositing changes the picture.
