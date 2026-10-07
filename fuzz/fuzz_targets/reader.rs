#![no_main]

mod support;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let input = String::from_utf8_lossy(data);
    let normalized = input.replace("\r\n", "\n").replace('\r', "\n");
    for (_, _, options) in support::profiles() {
        let parsed = support::snapshot(&input, options);
        assert_eq!(parsed, support::snapshot(&normalized, options));
        if let Some(error) = parsed.get("error") {
            let line = error["line"].as_u64().unwrap() as usize;
            assert!(
                (1..=normalized.bytes().filter(|&byte| byte == b'\n').count() + 1).contains(&line)
            );
        }
    }
});
