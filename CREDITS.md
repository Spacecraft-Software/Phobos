<!--
SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
SPDX-License-Identifier: CC-BY-SA-4.0
-->

# Credits

Phobos stands on other people's work (Standard §15.3).

| Work | Licence | How Phobos uses it |
|------|---------|--------------------|
| [id Software, *Doom* source (`linuxdoom-1.10`)](https://github.com/id-Software/DOOM) | GPL (see `NOTICE.md`) | The game logic, renderer and tables are transliterated from it |
| [Chocolate Doom](https://github.com/chocolate-doom/chocolate-doom), Simon Howard and contributors | GPL-2.0-or-later | The behavioural reference: corrections to the 1997 release, game-version differences, and the test oracle CI runs out of tree. No Chocolate Doom code is linked into Phobos |
| [Freedoom](https://freedoom.github.io/) | BSD-3-Clause | The free IWADs used in CI. Not distributed with Phobos |
| [room4doom](https://github.com/flukejones/room4doom), Luke Jones | MIT | Prior art consulted for structuring a Rust port. No code adapted so far |
| [rust-doom](https://github.com/cristicbz/rust-doom), Cristi Cobzarenco | Apache-2.0 / MIT | Prior art for WAD and BSP handling in Rust. No code adapted so far |

If code is adapted from any of these, the file carries the upstream notice and
this table names the files.
