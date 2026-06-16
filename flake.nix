{
  description = "Development environment for RustiCal";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-26.05";
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
        pkgs = import nixpkgs { inherit system; };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # The task runner, similar to make
            just
            # For frontend modules
            deno
            # For documentation
            mkdocs
            python314Packages.mkdocs-material
          ];
        };
      }
    );
}
