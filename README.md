# stim-rs

Native Rust bindings for [Stim](https://github.com/quantumlib/Stim), Google's high-performance stabilizer circuit simulator.  
The crate vendors Stim as a git submodule, builds it with CMake, and exposes a small safe API for parsing circuits, computing reference data, and sampling measurement results.

## Prerequisites

- Rust ≥ 1.70 (Rust 2021 edition)
- A C++20 toolchain (clang or gcc) plus `cmake` and `ninja`/`make`
- Python/pybind11 are *not* required (Stim's optional bindings are skipped automatically)

## Building

```bash
git clone https://github.com/your-org/stim-rs
cd stim-rs
git submodule update --init --recursive
cargo test
```

The build script compiles Stim as a static library via CMake and then compiles a small C++ shim via `cxx-build`. If you need to force a specific SIMD width for Stim, set `STIM_RS_SIMD_WIDTH` to 64/128/256 before running Cargo.

## Usage

```rust
use stim_rs::{Circuit, Sampler, SamplerOptions, SurfaceCodeConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let circuit = Circuit::surface_code(SurfaceCodeConfig {
        rounds: 3,
        distance: 3,
        ..Default::default()
    })?;

    println!("Qubits: {}", circuit.count_qubits()?);
    println!("Measurements: {}", circuit.count_measurements()?);

    let reference = circuit.reference_sample()?;
    println!("Reference sample: {:?}", reference);

    let mut sampler = Sampler::new(&circuit, SamplerOptions::default())?;
    let samples = sampler.sample(4)?;
    println!("Samples: {:?}", samples);

    Ok(())
}
```

You can also run the included example to dump a small surface-code circuit, build its decoding graph, and run a few decoding attempts with [fusion-blossom](https://crates.io/crates/fusion-blossom):

```bash
cargo run --example surface_code
```

Need to peek at the generated detector error model? Use the new DEM API:

```rust
use stim_rs::{Circuit, DemOptions, DetectorErrorModel};

let circuit = Circuit::from_text(
    "R 0
     R 1
     CX 0 1
     M 0 1
     DETECTOR rec[-1] rec[-2]",
)?;
let dem = DetectorErrorModel::from_circuit(&circuit, DemOptions::default())?;
println!("detectors = {}", dem.count_detectors()?);
let dem_twice = (&dem + &dem).flattened();
assert!(dem.approx_equals(&dem_twice.repeated(1), 1e-9));

for inst in dem.flattened_error_instructions()? {
    for (group_idx, group) in inst.separated_target_groups().iter().enumerate() {
        println!("error group {group_idx}: {:?}", group);
    }
}
```

## Examples

The `examples/` directory contains small walkthroughs you can run directly:

- `cargo run --example basic_circuit` – parse a deterministic circuit, sample it, and inspect its DEM.
- `cargo run --example dem_builder` – build a detector error model by hand, mutate it, and combine it with one extracted from a circuit.
- `cargo run --example surface_code` – generate a surface-code circuit, build a decoding graph, and estimate the logical error rate with `fusion-blossom`.

## Project layout

- `stim/` – upstream Stim source (git submodule)
- `include/stim_rs/` – C++ declarations shared with the shim
- `src/ffi.rs` – `cxx` bridge definitions
- `src/ffi/shim.cc` – implementation of the safe C++ surface
- `src/circuit.rs` – circuit wrapper, reference data helpers, detection-event batches, and surface-code generation.
- `src/dem.rs` – detector-error-model types, mutation helpers, arithmetic, and coordinate queries.
- `src/sampler.rs` – measurement sampler wrapper and options.
- `src/error.rs` / `src/util.rs` – shared error type plus small helpers.
- `src/lib.rs` – re-exports of the safe Rust API (minimal glue).

## Status

The current API focuses on circuits, detector-error-model inspection/mutation, and measurement & detection-event sampling. Pull requests that add more Stim subsystems (e.g., additional simulators, converters, layout tooling) are welcome.
