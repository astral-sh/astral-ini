//! Count system-allocator requests separately from timing benchmarks.

#[path = "../benches/support/mod.rs"]
mod support;

use serde_json::json;
use stats_alloc::{INSTRUMENTED_SYSTEM, Region, StatsAlloc};
use std::alloc::System;
use support::{Mode, Parser};

#[global_allocator]
static ALLOCATOR: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

fn main() {
    for case in support::cases() {
        case.validate();
        for parser in Parser::ALL {
            for mode in Mode::ALL {
                let region = Region::new(ALLOCATOR);
                case.run(parser, mode);
                let stats = region.change();
                println!(
                    "{}",
                    json!({
                        "case": case.name, "bytes": case.input.len(), "captured": case.captured,
                        "parser": parser.name(), "mode": mode.name(),
                        "allocations": stats.allocations, "reallocations": stats.reallocations,
                        "deallocations": stats.deallocations,
                        "bytes_allocated": stats.bytes_allocated,
                        "bytes_deallocated": stats.bytes_deallocated,
                        "bytes_reallocated": stats.bytes_reallocated,
                    })
                );
            }
        }
    }
}
