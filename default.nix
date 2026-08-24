{ pkgs ? import <nixpkgs> {} }:

let
  manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
in
pkgs.rustPlatform.buildRustPackage rec {
  pname = manifest.name;
  version = manifest.version;

  src = pkgs.lib.cleanSource ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
# These are the output hashes of nix packaging for vendoring git dependencies.
# They cannot be calculated during build time.
# They make sure that packages/dependencies are resolved reproducibly.
# The hash needs to be adjusted with a new version of wayrs-client.
# for debugging/ mismatches run "nix flake check --all-systems" or "nix build"
# See https://nix.dev/manual/nix/2.34/language/advanced-attributes.html#adv-attr-outputHash

    outputHashes = {
      "wayrs-client-1.3.1" = "sha256-9LnJnuUkE3M+7dqrfX4L7HEgBNrB4sbYXN0pVhCnNl4=";
    };
  };
#  WARNING: This is a hack to make the build work.
#  It is required so the tests are working. They need a valid User home filesystem
#  and they will fail if executed from /nix/store
#  It creates a build sandbox instead
  preCheck = ''
    export HOME="$TMPDIR/home"
    mkdir -p "$HOME"
  '';

}