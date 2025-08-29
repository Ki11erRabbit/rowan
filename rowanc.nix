{
    lib,
    stdenv,
    rustPlatform,
    libunwind,
    pkg-config,
    cargo,


}:

rustPlatform.buildRustPackage  {
    pname = "rowanc";
    version = "0.0.0";

    src = builtins.path { name = "rowanc"; path = ./.; };

    cargoBuildFlags = ["-p" "rowanc"];

    nativeBuildInputs = [
        pkg-config
    ];
    buildInputs = [
        libunwind
    ];

    outputs = [ "out" "dev" "lib" ];

    cargoLock = {
        lockFile = ./Cargo.lock;
        outputHashes = {
            "unwind-sys-0.1.4" = "sha256-kpONieYR+Nex/2K3fYGw4+QVlX5TiZ21tyNgd6gDc6c=";
        };
    };
    installPhase = ''
        cp target/release/rowanc $(out)
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
}