<!--
SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
SPDX-License-Identifier: CC-BY-SA-4.0
-->

# Security Policy

Phobos reads untrusted files: WADs, demos, savegames and DeHackEd patches
downloaded from anywhere. This document is the Standard §26.1 statement of how
to report a finding and what to expect.

## Reporting channel

Report privately through **GitHub's private vulnerability reporting** on this
repository (Security → Report a vulnerability). If that is unavailable to you,
email **Mohamed.Hammad@SpacecraftSoftware.org**. **Never open a public issue**
for a security finding.

## Acknowledgement target

This is a hobby project (Standard §5.1). You will receive an acknowledgement
within **14 days**.

## Scope

In scope: every crate in this repository, the packaging definitions, and the
CI workflows. Of particular interest:

- a WAD, demo, savegame or DeHackEd file that makes Phobos panic, hang,
  allocate without bound, or read or write outside its buffers;
- any path by which loaded data causes a file to be read or written outside
  the configuration, data and IWAD directories;
- any network access at all (Phobos makes none).

Out of scope: Chocolate Doom, which CI runs only as a test oracle (report it
upstream), and the content of IWADs or PWADs themselves.

## Supported versions

Best effort on the **latest release only**. No backports. Before the first
release, fixes land on `main`.

## Coordinated disclosure

The maintainer honours a **90-day** embargo from acknowledgement, extendable by
mutual agreement. A fixed vulnerability is published as a GitHub Security
Advisory naming the affected versions, the fixed version, the impact, and any
workaround, and citing the release SBOM (Standard §26.2).

## Credit

Reporters are credited in the advisory unless they ask not to be.
