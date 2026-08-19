# /home/ilya/test-uring/flake.nix
{
  description = "Среда разработки для тестирования tokio-uring на NixOS";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/utils";
  };

  outputs = { self, nixpkgs, utils }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Инструменты Rust
            cargo
            rustc
            rust-analyzer
            clippy
            rustfmt

            # Системные зависимости для сборки
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
