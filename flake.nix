{
  inputs = {
    devenv = {
      url = "github:cachix/devenv";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-parts.url = "github:hercules-ci/flake-parts";
    git-hooks = {
      url = "github:cachix/git-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixpkgs.url = "github:cachix/devenv-nixpkgs/rolling";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    systems.url = "github:nix-systems/default";
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { flake-parts, ... }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        inputs.git-hooks.flakeModule
        inputs.devenv.flakeModule
        inputs.treefmt-nix.flakeModule
      ];
      systems = import inputs.systems;

      perSystem =
        {
          config,
          lib,
          pkgs,
          system,
          ...
        }:
        {
          packages = {
            default = pkgs.callPackage ./nix/git-ombl.nix { };

            # Cross-compilation packages for releases
            git-ombl-linux-x86_64 = pkgs.callPackage ./nix/git-ombl.nix { };
            git-ombl-linux-aarch64 = pkgs.pkgsCross.aarch64-multiplatform.callPackage ./nix/git-ombl.nix { };
            git-ombl-macos-x86_64 = pkgs.pkgsCross.x86_64-darwin.callPackage ./nix/git-ombl.nix { };
            git-ombl-macos-aarch64 = pkgs.pkgsCross.aarch64-darwin.callPackage ./nix/git-ombl.nix { };
            git-ombl-windows-x86_64 = pkgs.pkgsCross.mingwW64.callPackage ./nix/git-ombl.nix { };
          };

          devenv.shells.default = {
            packages = with pkgs; [
              git
              nil
              openssl
              pkg-config
            ];
            containers = lib.mkForce { };
            languages.rust = {
              enable = true;
              channel = "nightly";
            };
            enterShell = ''
              ${config.pre-commit.installationScript}
            '';
          };

          pre-commit = import ./nix/pre-commit { inherit config; };
          treefmt = import ./nix/treefmt;
        };
    };
}
