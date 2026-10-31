<!--
SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
SPDX-License-Identifier: CC-BY-SA-4.0
-->

# The fidelity oracle

Phobos checks its simulation against Chocolate Doom, the reference vanilla
port. Chocolate Doom runs out of tree, from the flake, in CI and in the dev
shell only. No Chocolate Doom code is linked into Phobos (PRD NG-7).

## What M0 established (2026-10-31)

Chocolate Doom 3.1.1 runs headless with SDL's dummy drivers:

```sh
SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy \
  chocolate-doom -iwad freedoom1.wad -timedemo demo1 \
    -nosound -nomusic -nogui -statdump stats.txt
```

It prints `timed <gametics> gametics in <realtics> realtics` and exits with
status 255. That status is its normal end of a timedemo: it ends through
`I_Error`. `tools/oracle-smoke.sh` runs this in CI.

Two facts limit what this run can prove:

- `-statdump` records **completed levels only**. The built-in demos end
  mid-level, so their statdump is empty.
- The gametic count is the demo's length, so it is the same whether or not
  playback stays in sync.

So the statdump comparison of PRD R-072 applies to demos that finish levels,
and the per-tic comparison below is the main oracle.

## The per-tic trace (planned for M2, P-038 and P-039)

A test-only patch to Chocolate Doom, applied through a Nix overlay, adds
`-tictrace <file>`. After each `G_Ticker`, it writes one line per tic:

```
<gametic> <prndindex> <hash>
```

- `gametic`: the tic number, decimal.
- `prndindex`: `P_Random`'s index after the tic, decimal.
- `hash`: FNV-1a 64-bit, as 16 lower-case hex digits, over every map object
  in thinker-list order. Each object contributes these fields as
  little-endian 32-bit integers: `type`, `x`, `y`, `z`, `angle`, `health`,
  the state's index in `states[]`, and `flags`.

The checksum of Phobos's `World` (PRD R-038) uses exactly this definition, so
`phobos-tools demo diff` can name the first tic where the two disagree.
