let
pkgs = import (fetchTarball "https://github.com/NixOS/nixpkgs/archive/refs/tags/23.05.tar.gz") { };
unstable_pkgs = import (fetchTarball "https://nixos.org/channels/nixos-unstable/nixexprs.tar.xz") { };
in
pkgs.mkShell {
  packages = [
    pkgs.texlive.combined.scheme-medium
    unstable_pkgs.rustc
    unstable_pkgs.cargo
  ];
}
