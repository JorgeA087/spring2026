

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use std::collections::VecDeque;
use std::fs::File;
use std::io::Write;

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use std::thread;
use std::time::{Duration, Instant};

// ======================================================
// TASK TYPES
// ======================================================

#[derive(Debug, Clone, Copy)]
enum Kind {
    Io,
    Cpu,
}

impl Kind {
    fn cpu_cost(&self) -> usize {
        match self {
            Kind::Io => 10,
            Kind::Cpu => 35,
        }
    }
}

// ======================================================
// TASK
// ======================================================

#[derive(Debug, Clone)]
struct Task {
    id: usize,
    kind: Kind,
    cpu_cost: usize,
    duration: Duration,
    arrival_time: Instant,
}

impl Task {
    fn new(id: usize, kind: Kind) -> Self {
        Self {
            id,
            kind,
            cpu_cost: kind.cpu_cost(),
            duration: Duration::from_millis(200),
            arrival_time: Instant::now(),
        }
    }
}

// ======================================================
// CONFIG
// ======================================================

#[derive(Clone)]
struct Config {
    total_tasks: usize,
    num_workers: usize,
    io_probability: f64,
    cpu_budget: usize,
    monitor_tick_ms: u64,
    seed: u64,
}

impl Config {
    fn default() -> Self {
        Self {
            total_tasks: 1000,
            num_workers: 8,
            io_probability: 0.20,
            cpu_budget: 100,
            monitor_tick_ms: 10,
            seed: 42,
        }
    }
}

// ======================================================
// METRICS
// ======================================================

struct Metrics {
    completed: AtomicUsize,
    io_completed: AtomicUsize,
    cpu_completed: AtomicUsize,

    total_wait_ms: AtomicUsize,
    total_turnaround_ms: AtomicUsize,

    max_wait_ms: AtomicUsize,
}

impl Metrics {
    fn new() -> Self {
        Self {
            completed: AtomicUsize::new(0),
            io_completed: AtomicUsize::new(0),
            cpu_completed: AtomicUsize::new(0),

            total_wait_ms: AtomicUsize::new(0),
            total_turnaround_ms: AtomicUsize::new(0),

            max_wait_ms: AtomicUsize::new(0),
        }
    }
}

// ======================================================
// MONITOR SAMPLE
// ======================================================

#[derive(Clone)]
struct MonitorSample {
    timestamp_ms: u128,
    cpu_usage: usize,
    active_workers: usize,
}

// ======================================================
// RUN SIMULATION
// ======================================================

fn run_simulation(config: Config) {

    let start =
        Instant::now();

    let metrics =
        Arc::new(Metrics::new());

    let cpu_usage =
        Arc::new(AtomicUsize::new(0));

    let active_workers =
        Arc::new(AtomicUsize::new(0));

    let shutdown =
        Arc::new(AtomicBool::new(false));

    let generator_done =
        Arc::new(AtomicBool::new(false));

    let monitor_samples =
        Arc::new(Mutex::new(Vec::<MonitorSample>::new()));

    // ==================================================
    // SHARED FIFO QUEUE
    // ==================================================

    let queue =
        Arc::new(Mutex::new(VecDeque::<Task>::new()));

    // ==================================================
    // GENERATOR THREAD
    // ==================================================

    let gen_queue =
        Arc::clone(&queue);

    let gen_done_flag =
        Arc::clone(&generator_done);

    let generator_config =
        config.clone();

    let generator =
        thread::spawn(move || {

        let mut rng =
            StdRng::seed_from_u64(generator_config.seed);

        for id in 0..generator_config.total_tasks {

            thread::sleep(Duration::from_millis(20));

            let kind =
                if rng.random_bool(generator_config.io_probability) {
                    Kind::Io
                } else {
                    Kind::Cpu
                };

            let task =
                Task::new(id, kind);

            gen_queue
                .lock()
                .unwrap()
                .push_back(task);
        }

        gen_done_flag.store(true, Ordering::Relaxed);
    });

    // ==================================================
    // WORKERS
    // ==================================================

    let mut worker_handles = Vec::new();

    for _worker_id in 0..config.num_workers {

        let worker_queue =
            Arc::clone(&queue);

        let worker_cpu =
            Arc::clone(&cpu_usage);

        let worker_active =
            Arc::clone(&active_workers);

        let worker_metrics =
            Arc::clone(&metrics);

        let worker_done =
            Arc::clone(&generator_done);

        let worker_config =
            config.clone();

        let handle =
            thread::spawn(move || {

            loop {

                let maybe_task = {

                    let mut q =
                        worker_queue.lock().unwrap();

                    q.pop_front()
                };

                match maybe_task {

                    Some(task) => {

                        // ==========================
                        // CPU ADMISSION CONTROL
                        // ==========================

                        loop {

                            let current_cpu =
                                worker_cpu.load(Ordering::Relaxed);

                            if current_cpu + task.cpu_cost
                                <= worker_config.cpu_budget {

                                worker_cpu.fetch_add(
                                    task.cpu_cost,
                                    Ordering::Relaxed
                                );

                                break;
                            }

                            thread::sleep(Duration::from_millis(1));
                        }

                        worker_active.fetch_add(
                            1,
                            Ordering::Relaxed
                        );

                        let begin =
                            Instant::now();

                        let wait_ms =
                            begin
                                .duration_since(task.arrival_time)
                                .as_millis() as usize;

                        // ==========================
                        // SIMULATE WORK
                        // ==========================

                        thread::sleep(task.duration);

                        let turnaround_ms =
                            Instant::now()
                                .duration_since(task.arrival_time)
                                .as_millis() as usize;

                        worker_cpu.fetch_sub(
                            task.cpu_cost,
                            Ordering::Relaxed
                        );

                        worker_active.fetch_sub(
                            1,
                            Ordering::Relaxed
                        );

                        // ==========================
                        // METRICS
                        // ==========================

                        worker_metrics
                            .completed
                            .fetch_add(1, Ordering::Relaxed);

                        worker_metrics
                            .total_wait_ms
                            .fetch_add(wait_ms, Ordering::Relaxed);

                        worker_metrics
                            .total_turnaround_ms
                            .fetch_add(turnaround_ms, Ordering::Relaxed);

                        loop {

                            let current =
                                worker_metrics
                                    .max_wait_ms
                                    .load(Ordering::Relaxed);

                            if wait_ms <= current {
                                break;
                            }

                            if worker_metrics
                                .max_wait_ms
                                .compare_exchange(
                                    current,
                                    wait_ms,
                                    Ordering::Relaxed,
                                    Ordering::Relaxed
                                )
                                .is_ok()
                            {
                                break;
                            }
                        }

                        match task.kind {

                            Kind::Io => {
                                worker_metrics
                                    .io_completed
                                    .fetch_add(1, Ordering::Relaxed);
                            }

                            Kind::Cpu => {
                                worker_metrics
                                    .cpu_completed
                                    .fetch_add(1, Ordering::Relaxed);
                            }
                        }
                    }

                    None => {

                        // ==========================
                        // PROPER TERMINATION
                        // ==========================

                        if worker_done.load(Ordering::Relaxed)
                            && worker_queue.lock().unwrap().is_empty()
                        {
                            break;
                        }

                        thread::sleep(Duration::from_millis(1));
                    }
                }
            }
        });

        worker_handles.push(handle);
    }

    // ==================================================
    // MONITOR THREAD
    // ==================================================

    let monitor_cpu =
        Arc::clone(&cpu_usage);

    let monitor_active =
        Arc::clone(&active_workers);

    let monitor_shutdown =
        Arc::clone(&shutdown);

    let monitor_vec =
        Arc::clone(&monitor_samples);

    let monitor =
        thread::spawn(move || {

        while !monitor_shutdown.load(Ordering::Relaxed) {

            let sample =
                MonitorSample {

                    timestamp_ms:
                        start.elapsed().as_millis(),

                    cpu_usage:
                        monitor_cpu.load(Ordering::Relaxed),

                    active_workers:
                        monitor_active.load(Ordering::Relaxed),
                };

            monitor_vec
                .lock()
                .unwrap()
                .push(sample);

            thread::sleep(
                Duration::from_millis(
                    config.monitor_tick_ms
                )
            );
        }
    });

    // ==================================================
    // WAIT
    // ==================================================

    generator.join().unwrap();

    for handle in worker_handles {
        handle.join().unwrap();
    }

    shutdown.store(true, Ordering::Relaxed);

    monitor.join().unwrap();

    // ==================================================
    // SAVE CSV
    // ==================================================

    let samples =
        monitor_samples.lock().unwrap();

    let mut file =
        File::create("monitor_log.csv")
            .unwrap();

    writeln!(
        file,
        "timestamp_ms,cpu_usage,active_workers"
    )
    .unwrap();

    for s in samples.iter() {

        writeln!(
            file,
            "{},{},{}",
            s.timestamp_ms,
            s.cpu_usage,
            s.active_workers
        )
        .unwrap();
    }

    // ==================================================
    // FINAL METRICS
    // ==================================================

    let runtime =
        start.elapsed().as_millis();

    let completed =
        metrics.completed.load(Ordering::Relaxed);

    let io_done =
        metrics.io_completed.load(Ordering::Relaxed);

    let cpu_done =
        metrics.cpu_completed.load(Ordering::Relaxed);

    let avg_wait =
        metrics.total_wait_ms.load(Ordering::Relaxed)
            as f64
            / completed as f64;

    let avg_turnaround =
        metrics.total_turnaround_ms.load(Ordering::Relaxed)
            as f64
            / completed as f64;

    let max_wait =
        metrics.max_wait_ms.load(Ordering::Relaxed);

    let avg_cpu =
        samples.iter()
            .map(|s| s.cpu_usage as f64)
            .sum::<f64>()
            / samples.len() as f64;

    let avg_workers =
        samples.iter()
            .map(|s| s.active_workers as f64)
            .sum::<f64>()
            / samples.len() as f64;

    // ==================================================
    // OUTPUT
    // ==================================================

    println!();
    println!("== RANDOM FIFO BASELINE ==");

    println!(
        "{} tasks, random IO/CPU arrival order",
        config.total_tasks
    );

    println!();
    println!("— results —");

    println!(
        "{:<24}: {} ms",
        "total_runtime",
        runtime
    );

    println!(
        "{:<24}: {} (IO={}, CPU={})",
        "tasks completed",
        completed,
        io_done,
        cpu_done
    );

    println!(
        "{:<24}: {:.2} ms",
        "avg wait time",
        avg_wait
    );

    println!(
        "{:<24}: {:.2} ms",
        "avg turnaround time",
        avg_turnaround
    );

    println!(
        "{:<24}: {} ms",
        "max wait time",
        max_wait
    );

    println!(
        "{:<24}: {:.2} %",
        "avg CPU usage",
        avg_cpu
    );

    println!(
        "{:<24}: {:.2} / {}",
        "avg workers active",
        avg_workers,
        config.num_workers
    );

    println!(
        "{:<24}: {}",
        "monitor samples",
        samples.len()
    );

    println!(
        "{:<24}: monitor_log.csv",
        "monitor csv"
    );
}

// ======================================================
// MAIN
// ======================================================

fn main() {

    run_simulation(Config::default());
}