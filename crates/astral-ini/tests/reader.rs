//! Public reader contracts for Python packaging inputs.

use astral_ini::{Delimiters, ErrorKind, Ini, Options};

#[test]
fn entry_points() {
    let ini = Options::default()
        .case_sensitive(true)
        .delimiters(Delimiters::Equals)
        .parse("[console_scripts]\nTool=example:main\ntool=example:other\n")
        .unwrap();
    let scripts = ini.section("console_scripts").unwrap();
    assert_eq!(
        scripts.iter().collect::<Vec<_>>(),
        [("Tool", "example:main"), ("tool", "example:other")]
    );
    assert_eq!(scripts.get("TOOL"), None);
}

#[test]
fn inherited_defaults_and_multiline_values() {
    let ini = Ini::parse("[DEFAULT]\nShared=default\n[externally-managed]\nError: Use a virtual environment.\n\n See the documentation.\nShared=local\n[empty]\n").unwrap();
    assert_eq!(
        ini.get("externally-managed", "ERROR"),
        Some("Use a virtual environment.\n\nSee the documentation.")
    );
    assert_eq!(ini.get("externally-managed", "shared"), Some("local"));
    assert_eq!(ini.get("empty", "shared"), Some("default"));
    assert_eq!(ini.get("missing", "shared"), None);
    assert_eq!(
        ini.section("DEFAULT").unwrap().iter().collect::<Vec<_>>(),
        [("shared", "default")]
    );
}

#[test]
fn errors_preserve_python_precedence() {
    for (input, kind, line) in [
        ("name=example\n", ErrorKind::MissingSection, 1),
        ("[s]\ninvalid\na=1\nA=2\n", ErrorKind::DuplicateOption, 4),
        ("[s]\ninvalid\n[s]\n", ErrorKind::DuplicateSection, 3),
        ("[s]\ninvalid\nother invalid\n", ErrorKind::InvalidLine, 2),
    ] {
        let error = Ini::parse(input).unwrap_err();
        assert_eq!((error.kind(), error.line()), (kind, line));
        assert!(error.to_string().contains(&line.to_string()));
    }
}
