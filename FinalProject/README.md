# CPU/IO Task Scheduler Simulation

## Project Summary

This project is a multithreaded scheduling simulator written in Rust.

The simulator models CPU-bound and IO-bound tasks competing for workers under a limited CPU budget. Different scheduling strategies were tested to compare worker utilization, throughput, wait times, and overall performance.

The final implementation uses a two-queue scheduling system with weighted task selection to improve throughput while maintaining fairness between IO and CPU tasks.

---

# Build and Run

## Build

```bash
cargo build
```

## Run

```bash
cargo run --bin FIFO_balanced
```
```bash
cargo run --bin FIFO_stressed
```
```bash
cargo run --bin optimized_balanced
```
```bash
cargo run --bin optimized_stressed
```


---

# Command Examples

## Build project

```bash
cargo build
```

## Run optimized scheduler

```bash
cargo run
```

## Run in release mode

```bash
cargo run --release
```

## Clean project

```bash
cargo clean
```

---

# Summary of Design

The system contains several concurrent components:

- Generator thread
- Worker threads
- Monitor thread

## Generator Thread

Creates randomized tasks using a configurable IO probability.

Tasks are placed into either:

- IO queue
- CPU queue

---

## Worker Threads

Workers continuously:

1. pull tasks from queues
2. check CPU budget availability
3. simulate task execution
4. update shared metrics

Workers use a weighted scheduling policy to balance throughput and fairness.

---

## Monitor Thread

The monitor periodically records:

- CPU usage
- active workers
- queue sizes

Data is written to:

```text
monitor_log.csv
```

---

## Shared State

Shared data is protected using:

- `Arc`
- `Mutex`
- atomic variables

Shared structures include:

- queues
- metrics
- CPU usage counters
- monitor samples

---

# Summary of Experiments

Several scheduling strategies were tested.

## 1. Random FIFO Baseline

- single shared queue
- tasks processed in arrival order
- simpler and fairer
- lower throughput

### Results

- moderate CPU utilization
- lower worker usage
- lower throughput
- lower complexity

---

## 2. Two Queue System

- separate IO and CPU queues
- weighted scheduling policy
- IO tasks prioritized while CPU tasks still execute regularly

### Results

- higher worker utilization
- higher CPU usage
- improved throughput
- reduced total runtime

However, aggressive IO prioritization sometimes increased average wait time for CPU tasks.

---

## Final Observation

The best overall performance came from a balanced weighted scheduler that favored IO tasks without starving CPU tasks.

This improved:

- throughput
- worker activity
- CPU utilization

while keeping fairness more balanced than a pure IO-priority system.

---
---
# Tool Use Disclosure

AI tools were used during development of this project for:

- code debugging assistance
- concurrency design discussion
- scheduling policy experimentation
- README formatting help

---
# Author

Jorge Arreola