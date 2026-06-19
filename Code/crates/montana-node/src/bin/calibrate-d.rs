// Standalone benchmark — измеряет SHA-256 rate этой машины и calculates
// recommended params.d0 для target window 60 секунд.
//
// Использование:
//   cargo run --release --bin calibrate-d
//
// Output на stderr — детали runs; на stdout — recommended D (для scripting).

use std::time::Instant;

use mt_timechain::ssha_step;

// 10-минутный бенчмарк: 3 запуска × 1 миллиард итераций ≈ 200 сек/run.
// Total ≈ 600 секунд wall-clock на reference machine ~5 MH/s.
const BENCH_ITERS: u64 = 1_000_000_000;
const RUNS: usize = 3;
const TARGET_WINDOW_SECONDS: f64 = 60.0;

fn main() {
    eprintln!("=== Калибровка D под target window {TARGET_WINDOW_SECONDS:.1} сек ===");
    eprintln!("Запуск {RUNS} бенчмарков по {BENCH_ITERS} итераций SHA-256.");
    eprintln!();

    let bench_input = [0u8; 32];
    let mut rates_per_sec: Vec<f64> = Vec::with_capacity(RUNS);

    for run_idx in 1..=RUNS {
        let start = Instant::now();
        let _ = ssha_step(&bench_input, BENCH_ITERS);
        let elapsed = start.elapsed().as_secs_f64();
        let rate = (BENCH_ITERS as f64) / elapsed.max(0.0001);
        rates_per_sec.push(rate);
        eprintln!(
            "  run {run_idx}/{RUNS}: {BENCH_ITERS} за {:.3}s = {:.3} MH/s",
            elapsed,
            rate / 1_000_000.0
        );
    }

    rates_per_sec.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median_rate = rates_per_sec[RUNS / 2];
    let exact_d = (median_rate * TARGET_WINDOW_SECONDS) as u64;

    eprintln!();
    eprintln!(
        "Median rate:  {:.6} MH/s (single-thread)",
        median_rate / 1_000_000.0
    );
    eprintln!("Exact D:      {exact_d} (для точного 60.0 сек на этой машине)");
    eprintln!(
        "Estimated окно при D = {exact_d}: {:.4} секунд",
        exact_d as f64 / median_rate
    );
    eprintln!();
    eprintln!("Установите в crates/mt-genesis/src/lib.rs (genesis_params().d0):");
    eprintln!("    d0: {exact_d},");

    println!("{exact_d}");
}
