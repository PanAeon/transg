{
  description = "Rust dev env";

  inputs.flake-utils.url = "github:numtide/flake-utils";
  inputs.rust-overlay.url = "github:oxalica/rust-overlay";

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
      in {
        devShell = with pkgs;
          mkShell {
            name = "rust-env";
            buildInputs = [
              (rust-bin.nightly.latest.default.override {
                extensions = ["rust-src"];
              })
              rustfmt
              clippy
              rust-analyzer
              pkg-config
              cargo-generate
              wayland
              gtk4
              material-icons
              openssl
             ];
            PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig:${pkgs.libxml2.dev}/lib/pkgconfig";
#            RUST_SRC_PATH = "${pkgs.rust.packages.nightly.rustPlatform.rustLibSrc}";
            shellHook = "exec fish";
          };
      });
}
