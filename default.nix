let
  nixpkgs = builtins.fetchTarball "https://github.com/NixOS/nixpkgs/archive/nixos-24.05.tar.gz";
  pkgs = import nixpkgs { config = {}; overlays = []; };
  
  fenix = import (builtins.fetchTarball "https://github.com/nix-community/fenix/archive/main.tar.gz") {};

  static = pkgs.buildNpmPackage {
    pname = "static";
    version = "2.1.0";
    src = ./.;

    npmDepsHash = "sha256-R0scqexthcgnw4/DIPBwN/edKMWJiXr2pbW49bIgxcw=";

    npmFlags = [ "--ignore-scripts" ];

    dontNpmBuild = true;

    buildPhase = ''
      make node_modules
    '';

    installPhase = ''
      mkdir $out
      mv node_modules $out
    '';

    dontNpmInstall = true;
  };
in
pkgs.rustPlatform.buildRustPackage {
  pname = "paddler";
  version = "2.1.0";

  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  nativeBuildInputs = with pkgs; [
    fenix.minimal.toolchain
    pkg-config
    cmake
    nodejs
    rustPlatform.bindgenHook
  ];

  buildInputs = with pkgs; [
    openssl
    static
  ];
  
  buildFeatures = [ "web_admin_panel" ];

  buildPhase = ''
    ln -s result/node_modules ./node_modules
    make
    ls
  '';

  installPhase = ''
    mkdir -p $out/bin
    ls
    mv target/release/paddler $out/bin
  '';

  # checkFlags = [
  #   # 
  #   "--skip=example::tests:example_test"
  # ];

  doCheck = false;
}

