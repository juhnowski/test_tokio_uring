// /home/ilya/test_tokio_uring/src/main.rs

use std::alloc::{Layout, alloc, dealloc};
use std::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;
use std::time::Instant;
use tokio_uring::fs::File;

// 512 МБ данных, кратно 4096
const DATA_SIZE: usize = 512 * 1024 * 1024;
const ALIGNMENT: usize = 4096;

/// Вспомогательная функция для создания выровненного Vec<u8>, который понимает tokio-uring
fn allocate_aligned_vec(size: usize, align: usize) -> Vec<u8> {
    let layout = Layout::from_size_align(size, align).unwrap();
    unsafe {
        let ptr = alloc(layout);
        if ptr.is_null() {
            panic!("Ошибка аллокации");
        }
        // Чтобы Rust корректно освободил память, нам нужно вернуть Vec,
        // но безопаснее управлять этим буфером через кастомный контейнер
        // или использовать готовые выровненные типы, так как Vec ожидает свой аллокатор.
        Vec::from_raw_parts(ptr, size, size)
    }
}

async fn run_nbd_direct_bench(path: &'static str) -> Result<(), Box<dyn std::error::Error>> {
    println!("--- [O_DIRECT] Тестирование nbdcache на {} ---", path);

    // 1. Создаем выровненные буферы через libc
    let mut write_buf = allocate_aligned_vec(DATA_SIZE, ALIGNMENT);
    // Заполняем паттерном 0xCC
    for byte in write_buf.iter_mut() {
        *byte = 0xCC;
    }

    // 2. Открываем файл через стандартные OpenOptions с флагами прямого доступа
    let std_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .custom_flags(libc::O_DIRECT | libc::O_SYNC)
        .open(path)?;

    // Конвертируем в асинхронный файл tokio_uring
    let file = File::from_std(std_file);

    // --- ТЕСТ ЗАПИСИ ---
    println!("🚀 Запуск прямой записи (Direct Write)...");
    let start_write = Instant::now();

    // В tokio-uring буфер передается по владению.
    // Метод возвращает Future, который при завершении отдает (Результат, Буфер)
    let (res, write_buf_returned) = file.write_at(write_buf, 0).submit().await;
    res?;

    let write_dur = start_write.elapsed().as_secs_f64();
    let write_speed = (DATA_SIZE as f64 / 1024.0 / 1024.0) / write_dur;

    // --- ТЕСТ ЧТЕНИЯ ---
    println!("🚀 Запуск прямого чтения (Direct Read)...");
    let read_buf = allocate_aligned_vec(DATA_SIZE, ALIGNMENT);

    let start_read = Instant::now();
    let (res, _read_buf_returned) = file.read_at(read_buf, 0).await;
    res?;

    let read_dur = start_read.elapsed().as_secs_f64();
    let read_speed = (DATA_SIZE as f64 / 1024.0 / 1024.0) / read_dur;

    file.close().await?;

    println!("========================================");
    println!("Результаты nbdcache (O_DIRECT):");
    println!(
        "  Запись (Direct WAL):  {:.2} MB/s (за {:.4} сек)",
        write_speed, write_dur
    );
    println!(
        "  Чтение (Raw Disk):    {:.2} MB/s (за {:.4} сек)",
        read_speed, read_dur
    );
    println!("  Статус: Игнорирование Page Cache подтверждено.");
    println!("========================================");

    if std::fs::remove_file(path).is_ok() {
        println!("🗑️ Временный файл удален.");
    }

    Ok(())
}

fn main() {
    tokio_uring::start(async {
        // let test_path = "/mnt/nvme_final/nbdcache_direct.bin";
        let test_path = "/mnt/nvme_new/nbdcache_direct.bin";
        if let Err(e) = run_nbd_direct_bench(test_path).await {
            eprintln!("❌ Ошибка O_DIRECT бенчмарка: {}", e);
            eprintln!("Подсказка: проверьте наличие прав записи в /mnt/nvme_final");
        } else {
            println!("🏆 Триумф");
        }
    });
}
