{
  description = "A terminal UI icon picker for emoji, kaomoji, Unicode, and Nerd Font glyphs";

  inputs = {
    naersk = {
      url = "github:nix-community/naersk";
      flake = false;
    };
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
  };

  outputs =
    {
      self,
      naersk,
      nixpkgs,
    }:
    let
      supportedSystems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-darwin"
        "x86_64-linux"
      ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
      overlay = final: _previous: {
        latuicon = final.callPackage ./nix/package.nix {
          naersk = final.callPackage naersk { };
        };
      };
      pkgsFor =
        system:
        import nixpkgs {
          inherit system;
          overlays = [ overlay ];
        };
    in
    {
      overlays.default = overlay;

      packages = forAllSystems (
        system:
        let
          pkgs = pkgsFor system;
        in
        {
          inherit (pkgs) latuicon;
          default = pkgs.latuicon;
        }
      );

      apps = forAllSystems (
        system:
        let
          app = {
            type = "app";
            program = "${self.packages.${system}.latuicon}/bin/latuicon";
            meta.description = "Launch the latuicon terminal UI icon picker";
          };
        in
        {
          default = app;
          latuicon = app;
        }
      );

      checks = forAllSystems (
        system:
        let
          pkgs = pkgsFor system;
          package = self.packages.${system}.latuicon;
          formattingSource = pkgs.lib.fileset.toSource {
            root = ./.;
            fileset = pkgs.lib.fileset.unions [
              ./Cargo.toml
              ./build.rs
              ./flake.nix
              ./nix
              ./src
            ];
          };
        in
        {
          inherit package;

          clippy = (pkgs.callPackage naersk { }).buildPackage {
            name = "latuicon-clippy";
            inherit (package) src version;

            mode = "clippy";
            cargoBuildOptions =
              previous:
              previous
              ++ [
                "--all-targets"
                "--all-features"
              ];
            cargoClippyOptions = _previous: [ ];
          };

          formatting = pkgs.stdenvNoCC.mkDerivation {
            pname = "latuicon-formatting";
            inherit (package) version;
            src = formattingSource;

            nativeBuildInputs = [
              pkgs.cargo
              pkgs.nixfmt
              pkgs.rustfmt
            ];

            dontConfigure = true;

            buildPhase = ''
              runHook preBuild
              cargo fmt --all -- --check
              nixfmt --check flake.nix nix/package.nix
              runHook postBuild
            '';

            installPhase = ''
              runHook preInstall
              touch $out
              runHook postInstall
            '';
          };
        }
      );

      devShells = forAllSystems (
        system:
        let
          pkgs = pkgsFor system;
        in
        {
          default = pkgs.mkShell {
            inputsFrom = [ self.packages.${system}.latuicon ];
            packages = [
              pkgs.cargo
              pkgs.clippy
              pkgs.rust-analyzer
              pkgs.rustc
              pkgs.rustfmt
            ];
          };
        }
      );

      formatter = forAllSystems (system: (pkgsFor system).nixfmt);
    };
}
