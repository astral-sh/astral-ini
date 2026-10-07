//! Interleaved parser timings with raw samples as JSON lines.

mod support;

use std::time::{Duration, Instant};

use serde_json::json;
use support::{Case, Mode, Parser};

#[cfg(all(
    feature = "benchmark-jemalloc",
    target_os = "linux",
    any(target_arch = "x86_64", target_arch = "aarch64")
))]
#[global_allocator]
static ALLOCATOR: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

fn main() {
    let mut rounds = 6;
    let mut samples = 19;
    let mut sample_ms = 3;
    let mut warmup_ms = 10;
    let mut case_filter = String::new();
    let mut parser_filter = String::new();
    let mut mode_filter = String::new();
    let mut list = cfg!(debug_assertions);
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--bench" => {}
            "--list" | "--test" => list = true,
            "--rounds" => {
                rounds = args
                    .next()
                    .expect("round count")
                    .parse()
                    .expect("positive integer")
            }
            "--samples" => {
                samples = args
                    .next()
                    .expect("sample count")
                    .parse()
                    .expect("positive integer")
            }
            "--sample-ms" => {
                sample_ms = args
                    .next()
                    .expect("sample duration")
                    .parse()
                    .expect("positive integer")
            }
            "--warmup-ms" => {
                warmup_ms = args
                    .next()
                    .expect("warmup duration")
                    .parse()
                    .expect("nonnegative integer")
            }
            "--case" => case_filter = args.next().expect("case substring"),
            "--parser" => parser_filter = args.next().expect("parser name"),
            "--mode" => mode_filter = args.next().expect("mode name"),
            _ => panic!("unknown argument: {arg}"),
        }
    }
    assert!(rounds > 0 && samples > 0 && sample_ms > 0);
    let cases: Vec<_> = support::cases()
        .into_iter()
        .filter(|case| case.name.contains(&case_filter))
        .collect();
    let parsers: Vec<_> = Parser::ALL
        .into_iter()
        .filter(|parser| parser_filter.is_empty() || parser.name() == parser_filter)
        .collect();
    let modes: Vec<_> = Mode::ALL
        .into_iter()
        .filter(|mode| mode_filter.is_empty() || mode.name() == mode_filter)
        .collect();
    assert!(
        !cases.is_empty() && !parsers.is_empty() && !modes.is_empty(),
        "filters matched no workloads"
    );
    for case in &cases {
        case.validate();
        if list {
            println!(
                "{}",
                json!({"case": case.name, "bytes": case.input.len(), "captured": case.captured})
            );
        }
    }
    if list {
        return;
    }
    let allocator = if cfg!(all(
        feature = "benchmark-jemalloc",
        target_os = "linux",
        any(target_arch = "x86_64", target_arch = "aarch64")
    )) {
        "jemalloc"
    } else {
        "system"
    };
    for round in 0..rounds {
        for (index, case) in cases.iter().enumerate() {
            for mode in &modes {
                let mut order = parsers.clone();
                if (round + index) % 2 == 1 {
                    order.reverse();
                }
                for parser in order {
                    let iterations = calibrate(
                        case,
                        parser,
                        *mode,
                        Duration::from_millis(warmup_ms),
                        Duration::from_millis(sample_ms),
                    );
                    let values: Vec<_> = (0..samples)
                        .map(|_| {
                            elapsed(case, parser, *mode, iterations).as_secs_f64() * 1e9
                                / iterations as f64
                        })
                        .collect();
                    let mut sorted = values.clone();
                    sorted.sort_by(f64::total_cmp);
                    println!(
                        "{}",
                        json!({
                            "case": case.name, "bytes": case.input.len(), "captured": case.captured,
                            "parser": parser.name(), "mode": mode.name(), "allocator": allocator,
                            "round": round, "iterations": iterations, "samples_ns": values,
                            "median_ns": sorted[sorted.len() / 2],
                        })
                    );
                }
            }
        }
    }
}

/// Calibrate batches so timer calls stay outside the parse loop.
fn calibrate(case: &Case, parser: Parser, mode: Mode, warmup: Duration, target: Duration) -> u64 {
    let start = Instant::now();
    let mut iterations = 1;
    loop {
        let duration = elapsed(case, parser, mode, iterations);
        if duration >= target {
            if start.elapsed() >= warmup {
                return iterations;
            }
        } else {
            iterations *= 2;
        }
    }
}

fn elapsed(case: &Case, parser: Parser, mode: Mode, iterations: u64) -> Duration {
    let start = Instant::now();
    for _ in 0..iterations {
        case.run(parser, mode);
    }
    start.elapsed()
}
