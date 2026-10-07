#[path = "unicode_data.rs"]
mod data;

/// Lowercase with CPython 3.12's Unicode tables, including contextual final sigma.
pub(crate) fn lowercase(source: &str) -> String {
    if source.is_ascii() {
        return source.to_ascii_lowercase();
    }
    let mut result = String::with_capacity(source.len());
    let mut preceding_cased = false;
    for (index, character) in source.char_indices() {
        if character.is_ascii() {
            result.push(character.to_ascii_lowercase());
        } else if character == '\u{130}' {
            result.push_str("i\u{307}");
        } else if character == '\u{3a3}' {
            let following_cased = source[index + character.len_utf8()..]
                .chars()
                .find(|&c| !contains(data::CASE_IGNORABLE, c))
                .is_some_and(|c| contains(data::CASED, c));
            result.push(if preceding_cased && !following_cased {
                '\u{3c2}'
            } else {
                '\u{3c3}'
            });
        } else {
            result.push(lowercase_character(character));
        }
        if !contains(data::CASE_IGNORABLE, character) {
            preceding_cased = contains(data::CASED, character);
        }
    }
    result
}

/// Apply a single-character mapping; the caller handles expansion and final sigma.
fn lowercase_character(character: char) -> char {
    let codepoint = u32::from(character);
    let index = data::LOWERCASE.partition_point(|&(start, _, _, _)| start <= codepoint);
    if let Some(&(start, end, step, offset)) =
        index.checked_sub(1).and_then(|i| data::LOWERCASE.get(i))
        && codepoint <= end
        && (codepoint - start) % step == 0
    {
        return char::from_u32(codepoint.checked_add_signed(offset).unwrap()).unwrap();
    }
    character
}

/// Test membership in sorted, non-overlapping inclusive Unicode ranges.
fn contains(ranges: &[(u32, u32)], character: char) -> bool {
    let codepoint = u32::from(character);
    let index = ranges.partition_point(|&(_, end)| end < codepoint);
    ranges
        .get(index)
        .is_some_and(|&(start, _)| start <= codepoint)
}

#[cfg(test)]
mod tests {
    use super::lowercase;

    #[test]
    fn contextual_sigma() {
        assert_eq!(lowercase("ΟΣ"), "ος");
        assert_eq!(lowercase("ΟΣΑ"), "οσα");
        assert_eq!(lowercase("ΟΣ'"), "ος'");
        assert_eq!(lowercase("ΟΣ'Α"), "οσ'α");
        assert_eq!(lowercase("Ο\u{301}Σ\u{301}"), "ο\u{301}ς\u{301}");
        assert_eq!(lowercase("\u{345}Σ"), "\u{345}σ");
    }

    #[test]
    fn pinned_unicode_version() {
        assert_eq!(lowercase("İKẞ"), "i\u{307}kß");
        // These uppercase mappings were introduced after Unicode 15.
        assert_eq!(
            lowercase("\u{1c89}\u{a7cb}\u{10d50}"),
            "\u{1c89}\u{a7cb}\u{10d50}"
        );
    }
}
