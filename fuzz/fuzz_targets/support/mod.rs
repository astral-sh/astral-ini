use astral_ini::{Delimiters, ErrorKind, Ini, Options};
use serde_json::{Value, json};

pub(crate) fn profiles() -> impl Iterator<Item = (bool, &'static str, Options)> {
    [false, true].into_iter().flat_map(|case_sensitive| {
        [Delimiters::Equals, Delimiters::EqualsAndColon]
            .into_iter()
            .map(move |delimiters| {
                (
                    case_sensitive,
                    if delimiters == Delimiters::Equals {
                        "="
                    } else {
                        "=:"
                    },
                    Options::default()
                        .case_sensitive(case_sensitive)
                        .delimiters(delimiters),
                )
            })
    })
}

pub(crate) fn snapshot(input: &str, options: Options) -> Value {
    match options.parse(input) {
        Ok(ini) => contents(&ini),
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

pub(crate) fn contents(ini: &Ini<'_>) -> Value {
    json!({
        "defaults": ini.defaults().collect::<Vec<_>>(),
        "sections": ini.sections().map(|(name, section)| {
            (name, section.iter().collect::<Vec<_>>())
        }).collect::<Vec<_>>(),
    })
}
