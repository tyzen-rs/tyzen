# Tyzen Benchmark

These benchmarks are designed for reproducible README performance claims.

- `codegen` is a synthetic microbenchmark for renderer throughput.
- `e2e` is a realistic benchmark for full `tyzen::generate(...)` flow.
- `cold-process` runs generation in a fresh process for each sample.
- `build-time-delta` compares `cargo check` baseline vs Tyzen macro fixture.

## What It Measures

`codegen` measures:

- `metadata_build`: building synthetic `TypeMeta` entries in-memory
- `render_only`: rendering generated TypeScript from metadata
- `total`: end-to-end time for both steps

Results are reported as median and p95.

## Scenarios

- `small`: 20 types, 10 fields/type
- `medium`: 200 types, 16 fields/type
- `large`: 1000 types, 24 fields/type

`e2e` measures:

- inventory collection + namespace resolution
- full TypeScript rendering
- file write to disk
- cleanup

`e2e_heavy` measures:

- same E2E flow as `e2e`
- much larger dataset (900 types, 300 commands, 10 namespaces)
- intended for 500KB+ output stress and release-to-release comparison

`cold-process` measures:

- fresh-process generation latency (`cargo run --release` per sample)
- includes process startup + binary launch overhead

`build-time-delta` measures:

- baseline compile-check (`examples/cold_baseline.rs`)
- compile-check with Tyzen derive/command macros (`examples/cold_generate_fixture.rs`)
- median delta and ratio

## Run

From repository root:

```bash
cargo bench -p tyzen --bench codegen
cargo bench -p tyzen --bench e2e
cargo bench -p tyzen --bench e2e_heavy
./tyzen/bench/scripts/run_cold_generate.sh 10 2
./tyzen/bench/scripts/run_build_time_delta.sh 6 1
```

## Output

Benchmark output is written to:

- `tyzen/bench/results/latest.json` (microbenchmark)
- `tyzen/bench/results/e2e-latest.json` (realistic flow)
- `tyzen/bench/results/e2e-heavy-latest.json` (heavy realistic flow)
- `tyzen/bench/results/cold-generate-latest.json` (cold process flow)
- `tyzen/bench/results/build-time-delta-latest.json` (build-time comparison)

This file is intended to be committed or attached to release notes when updating README benchmark numbers.

## Environment Template (for README)

When publishing numbers, include:

- CPU model
- RAM
- OS version
- Rust toolchain (`rustc --version`)
- Command used (`cargo bench -p tyzen --bench codegen` or `cargo bench -p tyzen --bench e2e`)
- Run counts (`warmup_runs`, `measured_runs`) from results JSON
