# SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Nix flake for Phobos (PLAN.md P-004, P-011).
#
# Usage:
#   nix develop            dev shell: audit tools, the oracle, Freedoom
#   nix build .#oracle     Chocolate Doom, the test oracle (doc/oracle.md)
#   nix build .#freedoom   the Freedoom IWADs CI uses
#
# The Rust toolchain is not taken from nixpkgs: rust-toolchain.toml pins it,
# and rustup installs it. nixpkgs' rustc can trail that pin.
#
# The Freedoom IWADs are pinned by flake.lock; tools/testdata.sha256 records
# their checksums so a silent nixpkgs change fails CI instead of moving the
# test data.
{
  description = "Phobos: a memory-safe Rust port of the id Tech 1 engine";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        freedoomDir = "${pkgs.freedoom}/share/games/doom";
      in
      {
        packages = {
          oracle = pkgs.chocolate-doom;
          inherit (pkgs) freedoom;
        };

        devShells.default = pkgs.mkShell {
          name = "phobos-dev";
          packages = with pkgs; [
            rustup
            cargo-deny
            cargo-audit
            cargo-fuzz
            reuse
            texinfo
            chocolate-doom
            freedoom
          ];
          PHOBOS_FREEDOOM_DIR = freedoomDir;
          shellHook = ''
            echo "phobos dev shell"
            echo "  PHOBOS_FREEDOOM_DIR=$PHOBOS_FREEDOOM_DIR"
            echo "  gates: see AGENTS.md"
          '';
        };

        formatter = pkgs.nixfmt-rfc-style;
      }
    );
}
