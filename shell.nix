{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  buildInputs = with pkgs; [
    rustc
    cargo
    libunwind
    pkg-config
  ];

  
    shellHook = ''
      export LD_LIBRARY_PATH=${pkgs.libunwind}/lib:$LD_LIBRARY_PATH
        export RUSTFLAGS="-C link-arg=-Wl,--export-dynamic -C link-arg=-Wl,--whole-archive -C link-arg=-lunwind -C link-arg=-Wl,--no-whole-archive"
    '';
}
