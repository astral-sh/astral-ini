use astral_ini::{Delimiters, ErrorKind, Options};
use serde_json::{Value, json};

pub(crate) fn profiles() -> impl Iterator<Item = (bool, &'static str, Options)> {
    [false, true].into_iter().flat_map(|case_sensitive| {
        [
            ("=", Delimiters::Equals),
            ("=:", Delimiters::EqualsAndColon),
        ]
        .into_iter()
        .map(move |(name, delimiters)| {
            (
                case_sensitive,
                name,
                Options::default()
                    .case_sensitive(case_sensitive)
                    .delimiters(delimiters),
            )
        })
    })
}

pub(crate) fn snapshot(input: &str, options: Options, lookups: &[(&str, &str)]) -> Value {
    match options.parse(input) {
        Ok(ini) => json!({
            "defaults": ini.defaults().collect::<Vec<_>>(),
            "sections": ini.sections().map(|(name, section)| {
                (name, section.iter().collect::<Vec<_>>())
            }).collect::<Vec<_>>(),
            "lookups": lookups.iter().map(|&(section, name)| ini.get(section, name)).collect::<Vec<_>>(),
        }),
        Err(error) => json!({
            "error": {
                "kind": match error.kind() {
                    ErrorKind::MissingSection => "MissingSection",
                    ErrorKind::DuplicateSection => "DuplicateSection",
                    ErrorKind::DuplicateOption => "DuplicateOption",
                    ErrorKind::InvalidLine => "InvalidLine",
                },
                "line": error.line(),
            }
        }),
    }
}
