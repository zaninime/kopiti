{ pkgs ? import ./nix/pkgs.nix }:
let inherit (pkgs) rustPlatform nix-gitignore;
in rustPlatform.buildRustPackage {
  pname = "kopiti";
  version = "0.1.0";

  src = nix-gitignore.gitignoreSource [ ".git/" ] ./.;

  cargoSha256 = "0hgnbwx72s67kka2m43snvc1qsmgxd6p14h0spradms6348xxbpc";
}
