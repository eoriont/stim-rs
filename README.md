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

## Project layout

- `stim/` – upstream Stim source (git submodule)
- `include/stim_rs/` – C++ declarations shared with the shim
- `src/ffi.rs` – `cxx` bridge definitions
- `src/ffi/shim.cc` – implementation of the safe C++ surface
- `src/lib.rs` – ergonomic Rust API

## Status

The current API focuses on circuits and measurement sampling. Pull requests extending coverage to additional Stim functionality (detector samplers, DEM support, etc.) are welcome.
