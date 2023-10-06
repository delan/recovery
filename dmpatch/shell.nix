{ pkgs ? import <nixpkgs> {} }:
let
  md-kernel-module = pkgs.callPackage ./md.nix {
    # Make sure the module targets the same kernel as your system is using.
    # kernel = config.boot.kernelPackages.kernel;
    kernel = pkgs.linuxPackages.kernel;
  };
in
pkgs.mkShell {
  shellHook = ''
  '';

  buildInputs = [
    (md-kernel-module.overrideAttrs (_: {
      patches = [ ./md.patch ];
    }))
  ];
}
