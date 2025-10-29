//! # simdcsv
//!
//! A fast SIMD parser for CSV files as defined by RFC 4180.
//!
//! This library leverages SIMD intrinsics (AVX2 on x86_64, NEON on ARM)
//! and LLVM's vectorization capabilities for high-performance CSV parsing.

pub mod io;
pub mod memory;
pub mod parser;
pub mod portability;

pub use parser::{parse_csv, ParsedCsv};

/// CSV padding size for safe SIMD reads
pub const CSV_PADDING: usize = 64;
