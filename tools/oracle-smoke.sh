#!/bin/sh
# SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Proves the Chocolate Doom oracle runs headless (PLAN.md P-012): it plays
# freedoom1.wad's DEMO1 as a timedemo and must report a nonzero tic count and
# write its statdump file. See doc/oracle.md for why the statdump is empty.
#
# Needs chocolate-doom on PATH and PHOBOS_FREEDOOM_DIR set; `nix develop`
# provides both.
set -eu

: "${PHOBOS_FREEDOOM_DIR:?run inside nix develop, or set PHOBOS_FREEDOOM_DIR}"
iwad="$PHOBOS_FREEDOOM_DIR/freedoom1.wad"
[ -f "$iwad" ] || { echo "oracle-smoke: missing $iwad" >&2; exit 1; }

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

# Chocolate Doom ends a timedemo through I_Error, so its exit status is 255
# on success; the log line and the statdump file are what is checked.
set +e
HOME="$work" SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy \
  chocolate-doom -iwad "$iwad" -timedemo demo1 -nosound -nomusic -nogui \
    -statdump "$work/stats.txt" >"$work/run.log" 2>&1
status=$?
set -e

line=$(grep -E '^timed [0-9]+ gametics' "$work/run.log" || true)
if [ -z "$line" ]; then
  echo "oracle-smoke: no timedemo result (exit status $status); log follows" >&2
  cat "$work/run.log" >&2
  exit 1
fi
tics=$(printf '%s\n' "$line" | sed 's/^timed \([0-9]*\) gametics.*/\1/')
if [ "$tics" -le 0 ]; then
  echo "oracle-smoke: timedemo ran 0 tics" >&2
  exit 1
fi
if [ ! -f "$work/stats.txt" ]; then
  echo "oracle-smoke: no statdump file written" >&2
  exit 1
fi
echo "oracle-smoke: ok, $line"
