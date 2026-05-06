{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs =
    {
      self,
      nixpkgs,
    }:
    let
      pkgs = nixpkgs.legacyPackages."x86_64-linux";
      dlopenLibraries = with pkgs; [
        libxkbcommon

        # GPU backend
        vulkan-loader
        # libGL

        # Window system
        wayland
        # xorg.libX11
        # xorg.libXcursor
        # xorg.libXi
      ];

    in
    {
      packages."x86_64-linux".default = pkgs.mkShell {
        buildInputs = with pkgs; [
          cargo
          rustc
          rustfmt
          clippy
          rust-analyzer

          glib
          glibc

          gdk-pixbuf
          pkg-config

          gtk4
          libxkbcommon
          libGL

          libxcursor
          libxrandr
          libxi
          libx11
        ];

        env.RUSTFLAGS = "-C link-arg=-Wl,-rpath,${pkgs.lib.makeLibraryPath dlopenLibraries}";
        env.RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
      };
    };
}
