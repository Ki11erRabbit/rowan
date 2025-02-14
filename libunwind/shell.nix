{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
    nativeBuildInputs = with pkgs; [ libclang libunwind clang ];
    packages = with pkgs; [ clang ];
    #propagateBuildInputs = with pkgs; [ libunwind ];
    shellHook = ''
        export LIBCLANG_PATH="${pkgs.libclang.lib}/lib"
        export LIBUNWIND_PATH="${pkgs.libunwind}/lib"
        #export LIBC_INCLUDES_PATH="${pkgs.glibc.dev}/include"
        #export GCC_INCLUDES_PATH="${pkgs.libclang.out}/include"
    '';

}
