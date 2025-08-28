{
    lib,
    stdenv,
    rustPlatform,
    libunwind,
    pkg-config,
    cargo,


}:
let
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
        cargoLock = {
            lockFile = ./Cargo.lock;
            outputHashes = {
                "unwind-sys-0.1.4" = "sha256-kpONieYR+Nex/2K3fYGw4+QVlX5TiZ21tyNgd6gDc6c=";
            };
        };

        outputs = [ "out" "dev" ];

        installPhase = ''
        cp target/release/librowan_runtime.so $(out)
        cp target/release/librowan_runtime.so $(dev)
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
rustPlatform.buildRustPackage  {
    pname = "rowan";
    version = "0.0.0";

    src = builtins.path { name = "rowan"; path = ./.; };

    nativeBuildInputs = [
    ];
    buildInputs = [
        rowan-runtime
        libunwind
    ];

    cargoLock = {
        lockFile = ./Cargo.lock;
        outputHashes = {
            "unwind-sys-0.1.4" = "sha256-kpONieYR+Nex/2K3fYGw4+QVlX5TiZ21tyNgd6gDc6c=";
        };
    };



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
}