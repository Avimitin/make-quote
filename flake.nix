{
  description = "Dev shell for setup build environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        toolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" ];
        };
      in with pkgs; {
        devShells.default = mkShell {
          buildInputs = [
            # Including cargo,clippy,cargo-fmt
            toolchain
            # rust-analyzer comes from nixpkgs toolchain, I want the unwrapped version
            rust-analyzer-unwrapped
          ];

          # Some environment to make rust-analyzer work correctly (Still the path prefix issue)
          RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";
        };
      });
}
