//! simdcsv - Fast SIMD CSV parser
//!
//! A high-performance CSV parser leveraging SIMD intrinsics and LLVM vectorization.

use clap::Parser;
use simdcsv::{io::get_corpus, parser::parse_csv, CSV_PADDING};
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(name = "simdcsv")]
#[command(about = "A fast SIMD parser for CSV files", long_about = None)]
struct Args {
    /// CSV file to parse
    #[arg(value_name = "FILE")]
    file: String,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Dump parsed field positions
    #[arg(short, long)]
    dump: bool,

    /// Number of iterations for benchmarking
    #[arg(short, long, default_value = "100")]
    iterations: usize,
}

fn main() {
    let args = Args::parse();

    if args.verbose {
        println!("[verbose] loading {}", args.file);
    }

    // Load file into memory with padding
    let buffer = match get_corpus(&args.file, CSV_PADDING) {
        Ok(buf) => buf,
        Err(e) => {
            eprintln!("Could not load the file {}: {}", args.file, e);
            std::process::exit(1);
        }
    };

    if args.verbose {
        println!("[verbose] loaded {} ({} bytes)", args.file, buffer.len());
    }

    // Warmup run
    let pcsv = parse_csv(buffer.data());

    if args.verbose {
        println!("number of indexes found    : {}", pcsv.indexes.len());
        if !pcsv.indexes.is_empty() {
            println!(
                "number of bytes per index : {:.2}",
                buffer.len() as f64 / pcsv.indexes.len() as f64
            );
        }
    }

    // Benchmark runs
    let mut total_time = 0.0;

    for _ in 0..args.iterations {
        let start = Instant::now();
        let _ = parse_csv(buffer.data());
        total_time += start.elapsed().as_secs_f64();
    }

    if args.dump {
        for (i, &idx) in pcsv.indexes.iter().enumerate() {
            print!("{}: ", idx);
            if i < pcsv.indexes.len() - 1 {
                let start = idx as usize;
                let end = pcsv.indexes[i + 1] as usize;
                if start < buffer.len() && end <= buffer.len() {
                    let field = &buffer.data()[start..end];
                    if let Ok(s) = std::str::from_utf8(field) {
                        print!("{}", s);
                    }
                }
            }
            println!();
        }
    }

    let volume = args.iterations as f64 * buffer.len() as f64;

    if args.verbose {
        println!("Total time in (s)          = {:.6}", total_time);
        println!("Number of iterations       = {}", args.iterations);
    }

    // Calculate and display performance metrics
    let gb_per_s = volume / total_time / (1024.0 * 1024.0 * 1024.0);
    println!(" GB/s: {:.5}", gb_per_s);

    if args.verbose {
        println!("[verbose] done");
    }
}
