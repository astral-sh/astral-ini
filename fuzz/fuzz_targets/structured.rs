#![no_main]

#[path = "support/python.rs"]
mod python;
mod support;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let names = [
        "key", "KEY", "other", "Σ", "ΟΣ", "İ", "ſ", "a:b", "a=b", "\u{a0}", "\0",
    ];
    let values = [
        "", "value", "a=b:c", "#literal", ";literal", "%(key)s", "${s:key}", "\0", "\u{2028}",
        "\u{85}",
    ];
    let endings = ["\n", "\r", "\r\n"];
    let mut input = String::from("[DEFAULT]\nbase = default\n[section]\n");
    // Bound generated work independently of the raw-input fuzz target.
    for chunk in data.as_chunks::<4>().0.iter().take(128) {
        let name = names[usize::from(chunk[1]) % names.len()];
        let value = values[usize::from(chunk[2]) % values.len()];
        let line = match chunk[0] % 12 {
            0 => format!("{name}={value}"),
            1 => format!("{name}:{value}"),
            2 => format!("[{name}]"),
            3 => format!("\t{value}"),
            4 => format!("  {name} = {value}"),
            5 => format!("#{value}"),
            6 => format!(";{value}"),
            7 => String::new(),
            8 => String::from("[DEFAULT]"),
            9 => format!("[{name}] trailing [{value}]"),
            10 => name.to_owned(),
            _ => format!("={value}"),
        };
        input.push_str(&line);
        input.push_str(endings[usize::from(chunk[3]) % endings.len()]);
    }
    python::compare(&input, &[]);
});
