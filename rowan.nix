{
    lib,
    stdenv,
    libunwind,
    pkg-config,
    cargo,


}:

stdenv.mkDerivation ({
    pname = "rowan";
    version = "0.0.0";

    src = builtins.path { name = "rowan"; path = ./.; };

    nativeBuildInputs = [
        cargo
    ];

    buildInputs = [
        libunwind
    ];

    outputs = [ "out" "dev" ];

    buildPhase = ''
    cargo build --release -p rowan-runtime
    cargo build --release -p rowan
    cp target/release/librowan_runtime.so $(out)
    cp target/release/librowan_runtime.so $(dev)
    cp target/release/rowan $(out)
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
})