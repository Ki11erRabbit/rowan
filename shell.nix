{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
    nativeBuildInputs = with pkgs; [ libclang libunwind clang ];
    packages = with pkgs; [ clang ];
    #propagateBuildInputs = with pkgs; [ libunwind ];
    shellHook = ''
        export LIBCLANG_PATH="${pkgs.libclang.lib}/lib"
    '';

}
