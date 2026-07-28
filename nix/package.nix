{
  lib,
  naersk,
}:

let
  manifest = builtins.fromTOML (builtins.readFile ../Cargo.toml);
in
naersk.buildPackage {
  inherit (manifest.package) version;
  pname = manifest.package.name;

  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.unions [
      ../Cargo.lock
      ../Cargo.toml
      ../LICENSE
      ../README.md
      ../THIRD_PARTY_LICENSES.md
      ../build.rs
      ../data
      ../src
    ];
  };

  doCheck = true;

  meta = {
    inherit (manifest.package) description;
    homepage = manifest.package.repository;
    license = lib.licenses.mit;
    mainProgram = manifest.package.name;
    platforms = lib.platforms.unix;
  };
}
