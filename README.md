# simdcsv

**simdcsv** is a high-performance CSV parser that leverages SIMD intrinsics (AVX2 on x86_64, NEON on ARM/aarch64) to achieve extremely fast parsing speeds. It is designed to be a robust and efficient solution for processing large CSV files, compliant with [RFC 4180](https://tools.ietf.org/html/rfc4180).

This project is heavily inspired by and adopts techniques from the [simdjson](https://github.com/lemire/simdjson) project.

## 🚀 Performance

**simdcsv** uses hardware acceleration to parse CSV data at multi-gigabyte per second speeds.

On an **Apple Silicon computer** (ARM64), benchmarks show:

- **simdcsv (Rust)**: ~4.1 GB/s (Safe Rust implementation)
- **Reference (C++)**: ~4.8 GB/s

The Rust implementation achieves competitive performance while maintaining memory safety guarantees for the core logic (outside of essential SIMD intrinsics). Version 0.1.1 introduced a significant performance bump for ARM architectures by implementing better pipelining and instruction batching.

## ✨ Features

- **Blazing Fast**: Uses SIMD instructions to process multiple bytes in parallel.
- **Cross-Platform**: Optimized for both x86_64 (AVX2) and ARM64 (NEON) architectures.
- **Safe Rust**: The core logic is implemented in safe Rust, minimizing memory safety risks.
- **RFC 4180 Compliant**: correctly handles quoted fields, escaped quotes, and CRLF line endings.

## 📦 Usage

### Rust

Add `simdcsv` to your `Cargo.toml`:

```toml
[dependencies]
simdcsv = "0.1.1"
```

Use it in your code:

```rust
use simdcsv::{parse_csv, get_corpus, CSV_PADDING};

fn main() {
    // Load file with required padding
    let buffer = get_corpus("data.csv", CSV_PADDING).unwrap();
    
    // Parse
    let pcsv = parse_csv(buffer.data());
    
    println!("Found {} fields", pcsv.indexes.len());
}
```

### CLI Tool

You can run the included CLI tool to benchmark or inspect CSV files:

```bash
# Build release version
cargo build --release

# Run benchmark
target/release/simdcsv examples/nfl.csv --iterations 1000

# Dump parsed fields
target/release/simdcsv examples/nfl.csv --dump
```

## 🛠 Building from Source

To build the project, ensuring you have the latest Rust toolchain installed:

```bash
git clone https://github.com/jagtesh/simdcsv.git
cd simdcsv
cargo build --release
```

## 📜 License

This project is licensed under the Apache License, Version 2.0 - see the `LICENSE` file for details.

The original C++ implementation and inspiration come from the work of [Daniel Lemire](https://github.com/lemire) and [Geoff Langdale](https://github.com/geofflangdale) on [simdjson](https://github.com/lemire/simdjson).
