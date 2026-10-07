#![no_main]

#[allow(dead_code)]
mod support;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let input = String::from_utf8_lossy(data);
    for (case_sensitive, _, options) in support::profiles() {
        let Ok(ini) = options.parse(&input) else {
            continue;
        };
        for (name, section) in ini.sections() {
            let found = ini.section(name).expect("enumerated section exists");
            assert_eq!(
                section.iter().collect::<Vec<_>>(),
                found.iter().collect::<Vec<_>>()
            );
            for (key, value) in section.iter() {
                assert_eq!(section.get(key), Some(value));
                assert_eq!(ini.get(name, key), Some(value));
                if !case_sensitive {
                    let alternate: String = key
                        .chars()
                        .map(|character| {
                            if character.is_ascii_lowercase() {
                                character.to_ascii_uppercase()
                            } else {
                                character
                            }
                        })
                        .collect();
                    assert_eq!(section.get(&alternate), Some(value));
                }
            }
        }
        for (key, value) in ini.defaults() {
            assert_eq!(ini.get("DEFAULT", key), Some(value));
        }
    }
});
