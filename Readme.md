# Проверка
```bash
[ilya@nixos:~]$ zgrep CONFIG_IO_URING /proc/config.gz
CONFIG_IO_URING=y
CONFIG_IO_URING_MOCK_FILE=m
CONFIG_IO_URING_ZCRX=y
```
подтверждает, что ваше ядро в NixOS полностью поддерживает io_uring на уровне компиляции.

# Запуск
```bash
nix develop
```
