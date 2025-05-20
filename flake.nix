{
  description = "Open Source Union Token Lists";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        rust = pkgs.rust-bin.stable."1.82.0".default;
      in
      {
        devShell = pkgs.mkShell {
          buildInputs = with pkgs; [
            rust
            openssl            
            pkg-config         
            jq
            bun
            just
            biome
            direnv
            nixfmt-rfc-style
            nodePackages_latest.nodejs
            python3
          ];
        };
      }
    );
}
