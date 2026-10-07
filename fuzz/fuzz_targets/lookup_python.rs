#![no_main]

#[path = "support/python.rs"]
mod python;
mod support;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Some((lengths, data)) = data.split_first_chunk::<4>() else {
        return;
    };
    // Two little-endian lengths precede the section, option, and document bytes.
    // Lengths wrap within the remaining bytes so every mutation stays decodable.
    let section_len = usize::from(u16::from_le_bytes([lengths[0], lengths[1]])) % (data.len() + 1);
    let (section, data) = data.split_at(section_len);
    let name_len = usize::from(u16::from_le_bytes([lengths[2], lengths[3]])) % (data.len() + 1);
    let (name, input) = data.split_at(name_len);
    let input = String::from_utf8_lossy(input);
    let section = String::from_utf8_lossy(section);
    let name = String::from_utf8_lossy(name);
    python::compare(&input, &[(&section, &name)]);
});
