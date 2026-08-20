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
cargo run --release
```

# Результат
```bash
[ilya@nixos:~/test_tokio_uring]$ cargo run --release
    Finished `release` profile [optimized] target(s) in 0.02s
     Running `target/release/test_tokio_uring`
--- Тестирование nbdcache I/O на /mnt/nvme_final/nbdcache_test.bin ---
Результаты nbdcache (Enterprise Bench):
  Запись (WAL sync):  1548.25 MB/s (за 0.3307 сек)
  Чтение (Scrubbing): 2496.28 MB/s (за 0.2051 сек)
🗑️ Временный файл удален.
🏆 Триумф
```
