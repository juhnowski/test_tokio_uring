// /home/ilya/test_tokio_uring/src/main.rs

use std::time::Instant;
use tokio_uring::fs::File;

// 512 МБ для серьезного прогрева NVMe
const DATA_SIZE: usize = 512 * 1024 * 1024;

async fn run_nbd_bench(path: &'static str) -> Result<(), Box<dyn std::error::Error>> {
    // Исправлено: добавлены let, правильный байтовый литерал 0xCC и тип u8
    let write_buf = vec![0xCCu8; DATA_SIZE];

    println!("--- Тестирование nbdcache I/O на {} ---", path);

    // ТЕСТ ЗАПИСИ (Имитация WAL journal сброса)
    let start_write = Instant::now();
    let file = File::create(path).await?;

    // В tokio-uring владение вектором переходит рантайму
    // Исправлено: write_buf теперь определен выше через let
    let (res, _returned_buf) = file.write_at(write_buf, 0).await;
    res?;

    // Гарантируем сброс на физический носитель
    file.sync_all().await?;

    let write_dur = start_write.elapsed().as_secs_f64();
    let write_speed = (DATA_SIZE as f64 / 1024.0 / 1024.0) / write_dur;
    file.close().await?;

    // ТЕСТ ЧТЕНИЯ (Имитация фонового scrubbing / Merkle tree проверки)
    let start_read = Instant::now();
    let file = File::open(path).await?;
    let read_buf = vec![0u8; DATA_SIZE];

    let (res, _read_buf) = file.read_at(read_buf, 0).await;
    res?;

    let read_dur = start_read.elapsed().as_secs_f64();
    let read_speed = (DATA_SIZE as f64 / 1024.0 / 1024.0) / read_dur;
    file.close().await?;

    println!("Результаты nbdcache (Enterprise Bench):");
    println!(
        "  Запись (WAL sync):  {:.2} MB/s (за {:.4} сек)",
        write_speed, write_dur
    );
    println!(
        "  Чтение (Scrubbing): {:.2} MB/s (за {:.4} сек)",
        read_speed, read_dur
    );

    // Очистка
    if std::fs::remove_file(path).is_ok() {
        println!("🗑️ Временный файл удален.");
    }

    Ok(())
}

fn main() {
    // Вход в рантайм io_uring
    tokio_uring::start(async {
        // Указываем путь к нашему подготовленному NVMe разделу в NixOS
        let test_path = "/mnt/nvme_final/nbdcache_test.bin";

        if let Err(e) = run_nbd_bench(test_path).await {
            eprintln!("❌ Ошибка при выполнении бенчмарка: {}", e);
        } else {
            println!("🏆 Триумф");
        }
    });
}
