#!/bin/sh
# SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Verifies the Freedoom IWADs against tools/testdata.sha256 (PLAN.md P-011),
# so a nixpkgs update cannot change the test data unnoticed.
set -eu
: "${PHOBOS_FREEDOOM_DIR:?run inside nix develop, or set PHOBOS_FREEDOOM_DIR}"
sums="$(cd "$(dirname "$0")" && pwd)/testdata.sha256"
cd "$PHOBOS_FREEDOOM_DIR"
sha256sum -c "$sums"
