{
  description = "Rust-Nix";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, flake-utils, ... }: flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs {
        inherit system;
      };
    in
    with pkgs; rec {
      formatter = nixpkgs-fmt;
      devShells.default = mkShell {
        nativeBuildInputs = [ pkg-config ];
        buildInputs = [ openssl ];
      };
    });
}
