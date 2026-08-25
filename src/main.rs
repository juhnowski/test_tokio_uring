// /home/ilya/test_tokio_uring/src/main.rs

use std::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;
use std::time::Instant;
use tokio_uring::fs::File;

// 512 МБ данных, кратно 4096
const DATA_SIZE: usize = 512 * 1024 * 1024;

/// Структура для сохранения результатов теста одного диска
struct BenchResult {
    name: &'static str,
    write_speed: f64,
    write_dur: f64,
    read_speed: f64,
    read_dur: f64,
}

async fn run_nbd_direct_bench(
    name: &'static str,
    path: &'static str,
) -> Result<BenchResult, Box<dyn std::error::Error>> {
    println!("\n--- [O_DIRECT] Тестирование: {} ({}) ---", name, path);

    // 1. Аллокация буфера (512 МБ автоматически выравниваются по границе страниц)
    let mut write_buf = vec![0xCC; DATA_SIZE];
    for byte in write_buf.iter_mut() {
        *byte = 0xCC;
    }

    // 2. Открытие файла с обходом кэша ядра
    let std_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .custom_flags(libc::O_DIRECT | libc::O_SYNC)
        .open(path)?;

    let file = File::from_std(std_file);

    // --- ТЕСТ ЗАПИСИ ---
    println!("🚀 Запуск прямой записи (Direct Write)...");
    let start_write = Instant::now();
    let (res, _write_buf_returned) = file.write_at(write_buf, 0).submit().await;
    res?;
    let write_dur = start_write.elapsed().as_secs_f64();
    let write_speed = (DATA_SIZE as f64 / 1024.0 / 1024.0) / write_dur;

    // --- ТЕСТ ЧТЕНИЯ ---
    println!("🚀 Запуск прямого чтения (Direct Read)...");
    let read_buf = vec![0x00; DATA_SIZE];
    let start_read = Instant::now();
    let (res, _read_buf_returned) = file.read_at(read_buf, 0).await;
    res?;
    let read_dur = start_read.elapsed().as_secs_f64();
    let read_speed = (DATA_SIZE as f64 / 1024.0 / 1024.0) / read_dur;

    file.close().await?;

    if std::fs::remove_file(path).is_ok() {
        println!("🗑️ Временный файл удален.");
    }

    Ok(BenchResult {
        name,
        write_speed,
        write_dur,
        read_speed,
        read_dur,
    })
}

fn main() {
    tokio_uring::start(async {
        // Запускаем тесты последовательно
        let res_new = run_nbd_direct_bench(
            "Samsung 970 EVO (nvme_new)",
            "/mnt/nvme_new/nbdcache_direct.bin",
        )
        .await;
        let res_final = run_nbd_direct_bench(
            "Samsung 970 EVO Plus (nvme_final)",
            "/mnt/nvme_final/nbdcache_direct.bin",
        )
        .await;

        println!(
            "\n================================================================================="
        );
        println!("🏆 СРАВНИТЕЛЬНЫЕ РЕЗУЛЬТАТЫ БЕНЧМАРКА (O_DIRECT + io_uring) 🏆");
        println!(
            "================================================================================="
        );
        println!(
            "{:<35} | {:<20} | {:<20}",
            "Накопитель", "Запись (Direct WAL)", "Чтение (Raw Disk)"
        );
        println!(
            "---------------------------------------------------------------------------------"
        );

        match res_new {
            Ok(r) => println!(
                "{:<35} | {:.2} MB/s ({:.3}s) | {:.2} MB/s ({:.3}s)",
                r.name, r.write_speed, r.write_dur, r.read_speed, r.read_dur
            ),
            Err(e) => eprintln!("Ошибка nvme_new: {}", e),
        }

        match res_final {
            Ok(r) => println!(
                "{:<35} | {:.2} MB/s ({:.3}s) | {:.2} MB/s ({:.3}s)",
                r.name, r.write_speed, r.write_dur, r.read_speed, r.read_dur
            ),
            Err(e) => eprintln!("Ошибка nvme_final: {}", e),
        }
        println!(
            "================================================================================="
        );
    });
}
