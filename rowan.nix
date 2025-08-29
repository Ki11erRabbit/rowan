{
    lib,
    stdenv,
    rustPlatform,
    libunwind,
    pkg-config,
    cargo,


}:
let
    triple = "${stdenv.hostPlatform.config}";
    rowan-runtime = rustPlatform.buildRustPackage {
        pname = "rowan-runtime";
        version = "0.0.0";

        src = builtins.path { name = "rowan-runtime"; path = ./.; };

        nativeBuildInputs = [
            cargo
            pkg-config
        ];

        buildInputs = [
            libunwind
        ];

        doCheck = false;

        cargoBuildFlags = ["-p" "rowan-runtime"];

        cargoLock = {
            lockFile = ./Cargo.lock;
            outputHashes = {
                "unwind-sys-0.1.4" = "sha256-kpONieYR+Nex/2K3fYGw4+QVlX5TiZ21tyNgd6gDc6c=";
            };
        };

        outputs = [ "out" "dev" "lib" ];

        installPhase = ''
            runHook preInstall
        mkdir -p $lib/lib $dev $out/lib $dev/include
        cp target/${triple}/release/librowan_runtime.so $lib/lib/
        cp headers/* $dev/include/
        ln -s $lib/lib $out/lib
        runHook postInstall
        '';
        meta = {
            description = "The Runtime for the Rowan Programming Language";
            homepage = "https://github.com/Ki11erRabbit/rowan";
            license = lib.licenses.mit;
            maintainers = [];
            platforms = [
                "x86_64-linux"
            ];
        };
    };
in
{
    rowan = rustPlatform.buildRustPackage  {
        pname = "rowan";
        version = "0.0.0";

        src = builtins.path { name = "rowan"; path = ./.; };
        doCheck = false;

        nativeBuildInputs = [
        ];
        buildInputs = [
            rowan-runtime
            libunwind
        ];

        LIBRARY_PATH = "${rowan-runtime.lib}/lib";
        LD_LIBRARY_PATH = "${rowan-runtime.lib}/lib";

        outputs = [ "out" ];

        cargoBuildFlags = ["-p" "rowan"];

        cargoLock = {
            lockFile = ./Cargo.lock;
            outputHashes = {
                "unwind-sys-0.1.4" = "sha256-kpONieYR+Nex/2K3fYGw4+QVlX5TiZ21tyNgd6gDc6c=";
            };
        };
        installPhase = ''
            runHook preInstall

            mkdir -p $out/bin
            cp target/${triple}/release/rowan $out/bin/

            runHook postInstall
        '';


        meta = {
            description = "The Runtime for the Rowan Programming Language";
            homepage = "https://github.com/Ki11erRabbit/rowan";
            licenses = lib.licenses.mit;
            maintainers = [];
            mainProgram = "rowan";
            platforms = [
                "x86_64-linux"
            ];
        };
    };
    rowan-runtime = rowan-runtime;
    rowanc = rustPlatform.buildRustPackage  {
         pname = "rowanc";
         version = "0.0.0";

         src = builtins.path { name = "rowanc"; path = ./.; };
         doCheck = false;

         cargoBuildFlags = ["-p" "rowanc"];

         nativeBuildInputs = [
             pkg-config
         ];
         buildInputs = [
         ];

         outputs = [ "out" ];

         cargoLock = {
             lockFile = ./Cargo.lock;
             outputHashes = {
                 "unwind-sys-0.1.4" = "sha256-kpONieYR+Nex/2K3fYGw4+QVlX5TiZ21tyNgd6gDc6c=";
             };
         };
         installPhase = ''
             runHook preInstall

             mkdir -p $out/bin
             cp target/${triple}/release/rowanc $out/bin/

             runHook postInstall
         '';


         meta = {
             description = "The Compiler for the Rowan Programming Language";
             homepage = "https://github.com/Ki11erRabbit/rowan";
             licenses = lib.licenses.mit;
             maintainers = [];
             mainProgram = "rowanc";
             platforms = [
                 "x86_64-linux"
             ];
         };
     };
}