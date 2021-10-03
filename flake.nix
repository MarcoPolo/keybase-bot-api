# Notes:
# You'll want the direnv plugin in VSCode.
# If rust-analyzer fails to load in VSCode, try restarting the rust-analyzer server.
# the VSCode direnv plugin is a little racey. And it can load after the rust-analyzer starts.
{
  description = "A Rust Environment flake";
  inputs.flake-utils.url = "github:numtide/flake-utils";
  inputs.rust-overlay.url = "github:oxalica/rust-overlay";

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (
      system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs { inherit system overlays; };
          rustStable = pkgs.rust-bin.stable.latest.default.override {
            extensions = [ "rust-src" ];
          };
        in
          {
            devShell = pkgs.mkShell {
              buildInputs = [
                rustStable
                # If the project requires openssl, uncomment these
                # pkgs.pkg-config
                # pkgs.openssl
              ];
              # If the project requires openssl, uncomment this
              # PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
            };
          }
    );
}
