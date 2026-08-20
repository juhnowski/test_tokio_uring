# /home/ilya/test_tokio_uring/flake.nix
{
  description = "Среда разработки для тестирования tokio-uring на NixOS";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    # Исправлено: правильный репозиторий называется flake-utils
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rustc
            rust-analyzer
            clippy
            rustfmt
            pkg-config
            libiconv
          ];

          shellHook = ''
            echo "--- Среда разработки tokio-uring активирована ---"
            echo "Версия ядра: $(uname -r)"
            cargo --version
          '';
        };
      });
}
