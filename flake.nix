{
  description = "Rust dev env";

  inputs.flake-utils.url = "github:numtide/flake-utils";
  inputs.rust-overlay.url = "github:oxalica/rust-overlay";

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
      in rec {
        packages.transgression = pkgs.callPackage (with pkgs; rustPlatform.buildRustPackage rec {
  pname = "transgression";
  version = "0.0.1";
  src = ./.;

  /*src = fetchFromGitHub {
    owner = "BurntSushi";
    repo = pname;
    rev = version;
    sha256 = "1iga3320mgi7m853la55xip514a3chqsdi1a1rwv25lr9b1p7vd3";
  };*/

  cargoSha256 = "17ldqr3asrdcsh4l29m3b5r37r5d0b3npq1lrgjmxb6vlx6a36qh";

  meta = with lib; {
    description = "A transgressive way to manage your transmission torrents";
    homepage = "https://github.com/BurntSushi/ripgrep";
    license = licenses.unlicense;
    maintainers = [];
  };
}) {};
        defaultPackage = packages.transgression;
        devShell = with pkgs;
          mkShell {
            name = "rust-env";
            nativeBuildInputs = [
    wrapGAppsHook
    ];
            buildInputs = [
              (rust-bin.stable.latest.default.override {
                extensions = ["rust-src"];
              })
              rustfmt
              clippy
              rust-analyzer
              pkg-config
              cargo-generate
              wayland
              gtk4
              openssl
              rust-bindgen
              curl
              libtorrent-rasterbar # needed for libtorrent-sys
              pkgs.boost.dev
             ];
            XDG_DATA_DIRS="/home/vitalii/.nix-profile/share:/run/current-system/sw/share:${pkgs.gnome.adwaita-icon-theme}/share:${pkgs.gtk4}/share/gsettings-schemas/gtk4-${pkgs.gtk4.version}:${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/gsettings-desktop-schemas-${pkgs.gsettings-desktop-schemas.version}";
            LIBCLANG_PATH= pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.lib ];
            PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig:${pkgs.libxml2.dev}/lib/pkgconfig";
             BINDGEN_EXTRA_CLANG_ARGS = 
    # Includes with normal include path
    (builtins.map (a: ''-I"${a}/include"'') [
      pkgs.glibc.dev 
      pkgs.libtorrent-rasterbar.dev
    ])
    # Includes with special directory paths
    ++ [
      ''-I"${pkgs.llvmPackages_latest.libclang.lib}/lib/clang/${pkgs.llvmPackages_latest.libclang.version}/include"''
      ''-I ${pkgs.llvmPackages_latest.clang.libc_dev}/include''
    ];
#            RUST_SRC_PATH = "${pkgs.rust.packages.nightly.rustPlatform.rustLibSrc}";
            shellHook = "exec fish";
          };
      });
}
