{ pkgs ? import <nixpkgs> { } }:

pkgs.mkShell rec {
  buildInputs = with pkgs; [
    expat
    fontconfig
    freetype
    freetype.dev
    libGL
    pkgconfig
    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr
    vulkan-loader
  ];

  LD_LIBRARY_PATH =
    builtins.foldl' (a: b: "${a}:${b}/lib") "${pkgs.vulkan-loader}/lib" buildInputs;
}
