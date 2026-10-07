{
  pkgs ? import <nixpkgs> { },
}:
pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    llvmPackages.clang
    pkg-config
  ];
  buildInputs = with pkgs; [
    chafa
    ffmpeg
    glib
    openssl
    sqls
    sqlx-cli
    cargo-nextest
  ];

  LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
  DATABASE_URL = "sqlite:///home/strigoi/.local/share/silver/library.db";
}
