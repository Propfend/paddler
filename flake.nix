{
  description = "Paddler - Open-source LLMOps platform for hosting and scaling AI";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    crane,
  }: let
    supportedSystems = ["x86_64-linux" "aarch64-linux"];
    forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
  in {
    packages = forAllSystems (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [rust-overlay.overlays.default];
      };

      version = "3.0.1";

      rustToolchain = pkgs.rust-bin.stable."1.93.0".default;
      craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

      frontendAssets = pkgs.buildNpmPackage {
        pname = "paddler-frontend";
        inherit version;
        src = pkgs.lib.cleanSourceWith {
          src = ./.;
          filter = path: type: let
            relativePath = pkgs.lib.removePrefix (toString ./. + "/") (toString path);
          in
            type
            == "directory"
            || relativePath == "package.json"
            || relativePath == "package-lock.json"
            || relativePath == "tsconfig.json"
            || relativePath == "eslint.config.js"
            || relativePath == "jarmuz-static.mjs"
            || pkgs.lib.hasPrefix "resources/" relativePath
            || pkgs.lib.hasPrefix "jarmuz/" relativePath;
        };
        npmDepsHash = "sha256-VyOuokidiLG0PU5EkjKHU2kGooesQZuRjxngin0lC4Y=";
        dontNpmBuild = true;
        buildPhase = ''
          runHook preBuild
          node jarmuz-static.mjs
          runHook postBuild
        '';
        installPhase = ''
          runHook preInstall
          mkdir -p $out
          cp -r static $out/
          cp esbuild-meta.json $out/
          runHook postInstall
        '';
      };

      src = let
        rustSrc = pkgs.lib.cleanSourceWith {
          src = ./.;
          filter = path: type: let
            relativePath = pkgs.lib.removePrefix (toString ./. + "/") (toString path);
          in
            (craneLib.filterCargoSources path type)
            || pkgs.lib.hasSuffix ".html" path
            || pkgs.lib.hasPrefix "resources/images/" relativePath
            || pkgs.lib.hasPrefix "fixtures/" relativePath
            || relativePath == "resources"
            || relativePath == "resources/images"
            || relativePath == "fixtures";
        };
      in
        pkgs.runCommand "paddler-src" {} ''
          cp -r ${rustSrc} $out
          chmod -R u+w $out
          cp -r ${frontendAssets}/static $out/static
          cp ${frontendAssets}/esbuild-meta.json $out/esbuild-meta.json
        '';

      commonArgs = {
        inherit src;
        pname = "paddler";
        inherit version;
        cargoExtraArgs = "--package paddler --features web_admin_panel";

        nativeBuildInputs = with pkgs; [
          cmake
          ninja
          pkg-config
          libclang
        ];

        buildInputs = with pkgs; [
          openssl
        ];

        LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
        BINDGEN_EXTRA_CLANG_ARGS = "-isystem ${pkgs.llvmPackages.libclang.lib}/lib/clang/${pkgs.lib.getVersion pkgs.llvmPackages.libclang}/include -isystem ${pkgs.glibc.dev}/include";
      };

      cargoArtifacts = craneLib.buildDepsOnly commonArgs;

      buildPaddler = features:
        craneLib.buildPackage (commonArgs
          // {
            inherit cargoArtifacts;
            cargoExtraArgs = "--package paddler --features ${features}";
          });

      paddler = buildPaddler "web_admin_panel";
      paddler-cuda = buildPaddler "web_admin_panel,cuda";
      paddler-vulkan = buildPaddler "web_admin_panel,vulkan";
    in {
      default = paddler;
      inherit paddler paddler-cuda paddler-vulkan;
    });

    devShells = forAllSystems (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [rust-overlay.overlays.default];
      };

      rustToolchain = pkgs.rust-bin.stable."1.93.0".default;
    in {
      default = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [
          rustToolchain
          cmake
          ninja
          pkg-config
          libclang
          nodejs
        ];

        buildInputs = with pkgs; [
          openssl
        ];

        LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
      };
    });
  };
}
