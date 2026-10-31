<!--
SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
SPDX-License-Identifier: CC-BY-SA-4.0
-->

# NOTICE

## Project Posture

Spacecraft Software is a **personal hobby project** maintained by Mohamed Hammad. Its
primary purpose is to serve the maintainer's own use case — a Rust-first
systems software stack, tooling, and rewrites that the maintainer wants to
run on his own machines. Most subprojects are developed at hobby pace and
shaped around that workflow.

A handful of subprojects (e.g., **Anvil-SSH**) are intentionally designed and
maintained with general use in mind. Where this applies, it is stated
explicitly in that subproject's `README.md`. The umbrella as a whole is not.

## No Warranty

Spacecraft Software and every subproject under it are provided **AS IS**, without
warranty of any kind, express or implied. This includes — but is not limited
to — implied warranties of merchantability, fitness for a particular purpose,
non-infringement, and the absence of latent defects. The entire risk as to
quality and performance lies with the user.

## No Liability

In no event shall the maintainer be liable for any claim, damages, or other
liability — whether in an action of contract, tort, or otherwise — arising
from, out of, or in connection with this software, its use, or its inability
to be used. This includes, without limitation, data loss, hardware damage,
business interruption, security incidents, and any direct, indirect,
incidental, special, exemplary, or consequential damages of any kind.

## Authoritative Terms

The statements above describe the maintainer's posture in plain language. The
formal, legally binding terms are those of the **GNU General Public License,
version 3 or later** (`GPL-3.0-or-later`). In any conflict between this
notice and the license text, the license text governs.

The full license is available at <https://www.gnu.org/licenses/gpl-3.0.html>
and in the project's `LICENSE` file.

## Phobos-specific notices

### Trademark

*Doom* is a trademark of id Software / ZeniMax Media. Phobos is an independent
engine that reads Doom data files. It ships no id Software assets and no id
Software logos, and claims no endorsement.

### Game data

Phobos distributes no IWAD. The commercial IWADs (`doom.wad`, `doom2.wad`,
`tnt.wad`, `plutonia.wad`) and the shareware `doom1.wad` must be supplied by
the user. Continuous integration uses only the free Freedoom IWADs, fetched
through the flake (BSD-3-Clause, by the Freedoom project).

### Code derived from the id Software source release

Parts of Phobos are transliterated from the id Software Doom source
(`linuxdoom-1.10`). Files that carry such code keep the line
`Copyright (C) 1993-1996 id Software, Inc.` and are marked
`GPL-2.0-or-later` in their SPDX header (Standard §4.2).

**Status of the "or later" grant (PLAN.md P-013): open, checked 2026-10-31.**

| Source | What it states |
|--------|----------------|
| `id-Software/DOOM`, commit `a77dfb96` (2024-01-16) | `README.TXT`: "Licensed under the GNU General Public License 2.0." Adds the GPL v2 text; the source files keep the 1997 "DOOM Source Code License" header |
| The 1997 archive, `idstuff/source/doomsrc.zip` (SHA-256 `29348bc1…a337`) | "DOOM Source Code License" headers; not a GPL release |
| Wikipedia, *Doom engine*, citing the idgames entry "Doom source code, under the GNU GPL" | Re-released 1999-10-03 under "GNU General Public License v2.0 or later" |
| Chocolate Doom (`src/doom/*.c`) | id-derived files carry "either version 2 of the License, or (at your option) any later version" |

The 1999 GPL archive itself was not reachable for a primary check. Until the
maintainer rules on this, no id-derived code is committed (the first such code
is planned for M2).

## Contributions, Suggestions, and Forks

- Contributions and suggestions are welcome — see [`CONTRIBUTING.md`](./CONTRIBUTING.md).
- PR acceptance is at the maintainer's sole discretion.
- Forking under GPL-3.0-or-later is always available and encouraged when
  goals diverge.

## Maintainer

Mohamed Hammad &lt;Mohamed.Hammad@SpacecraftSoftware.org&gt;
Copyright (c) 2026 Mohamed Hammad
License: GPL-3.0-or-later
Website: <https://Phobos.SpacecraftSoftware.org/>

---

*--- Forged in Spacecraft Software ---*
