#![no_main]

#[path = "support/python.rs"]
mod python;
mod support;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    python::compare(&String::from_utf8_lossy(data), &[]);
});
