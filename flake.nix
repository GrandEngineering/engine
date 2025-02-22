{
  description = "Main Rust Project with Nix Flakes";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
  };

  outputs = { self, nixpkgs }:
  let
    system = "x86_64-linux";  # Adjust if needed
    pkgs = import nixpkgs { inherit system; };
  in {
    packages.${system}.default = pkgs.rustPlatform.buildRustPackage {
      pname = "main_project";
      version = "1.0";

      src = ./.;  # Your main project source
      useFetchCargoVendor=true;
      cargoLock = {
        lockFile = ./Cargo.lock;
      };
    };
  };
}
