{
  description = "Development environment for RustiCal";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-26.05";
    nixpkgs-unstable.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      nixpkgs-unstable,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        pkgs-unstable = import nixpkgs-unstable { inherit system; };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs =
            (with pkgs; [
              # The task runner, similar to make
              just
              # For frontend modules
              deno

              cargo-vet
              cargo-about
              cargo-deny
              cargo-insta
            ])
            ++ (with pkgs-unstable; [
              # For documentation
              zensical
              cargo-release
            ]);
        };
      }
    );
}
