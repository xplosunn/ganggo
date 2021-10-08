{
  description = "ganggo";

  inputs = {
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    naersk = {
      url = "github:nmattia/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixpkgs.url = "nixpkgs/nixos-unstable";
  };

  outputs = { self, fenix, flake-utils, naersk, nixpkgs }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      rec {

        packages = rec {
          ganggo = (naersk.lib.${system}.override {
            inherit (fenix.packages.${system}.minimal) cargo rustc;
          }).buildPackage { src = ./.; };
          ggBashLib = pkgs.writeTextFile {
            name = "gg_bash_lib.sh";
            text = ''
              #This is a launcher script that takes care of re-redirecting to stdout so ganggo can be used in bash sensibly
              #Because ganggo draws on the terminal, we have to print the selection to stderr
              #To then use it we sadly can't just do something like selection=$(printf "a\nb" | gango 2>&1) because that still breaks terminal raw mode
              #So we gotta be uhhhh.... creative
              gg_dmenu_launch() {
                entries=$1
                outVarName=$2
                hint=$3
                { selection=$(printf "$entries" | ${ganggo}/bin/ganggo dmenu --hint "$hint" 2>&1 1>&$out); } {out}>&1
                declare -g "$outVarName"="$selection"
              }
            '';
          };
        };

        defaultPackage = packages.ganggo;

        devShell = pkgs.mkShell {
          name = "gango-dev-shell";
          buildInputs = [
            fenix.packages.${system}.minimal.cargo
            fenix.packages.${system}.minimal.rustc
            pkgs.rustfmt
            fenix.packages.${system}.rust-analyzer
            pkgs.nixpkgs-fmt
          ];

          # Certain Rust tools won't work without this
          # This can also be fixed by using oxalica/rust-overlay and specifying the rust-src extension
          # See https://discourse.nixos.org/t/rust-src-not-found-and-other-misadventures-of-developing-rust-on-nixos/11570/3?u=samuela. for more details.
          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
        };
      });
}
