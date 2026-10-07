use std::collections::BTreeMap;
use std::hint::black_box;

use astral_ini::{Delimiters, Options};
use configparser::ini::{Ini, IniDefault};

type Map = std::collections::HashMap<String, std::collections::HashMap<String, Option<String>>>;
type Snapshot = BTreeMap<String, BTreeMap<String, String>>;

#[derive(Clone, Copy)]
enum Workload {
    EntryPoints,
    SetupCfg,
    ExternallyManaged,
}

pub(crate) struct Case {
    pub(crate) name: &'static str,
    pub(crate) input: String,
    pub(crate) captured: bool,
    workload: Workload,
}

impl Case {
    fn options(&self) -> Options {
        Options::default().case_sensitive(true).delimiters(
            if matches!(self.workload, Workload::EntryPoints) {
                Delimiters::Equals
            } else {
                Delimiters::EqualsAndColon
            },
        )
    }

    fn baseline(&self) -> Ini {
        let mut defaults = IniDefault::default();
        defaults.case_sensitive = true;
        defaults.multiline = !matches!(self.workload, Workload::EntryPoints);
        if matches!(self.workload, Workload::EntryPoints) {
            defaults.delimiters = vec!['='];
        }
        Ini::new_from_defaults(defaults)
    }

    /// Verify full documents and selected owned outputs before timing either parser.
    pub(crate) fn validate(&self) {
        let astral = self.options().parse(&self.input).expect(self.name);
        let expected: Snapshot = astral
            .sections()
            .map(|(name, section)| {
                (
                    name.to_owned(),
                    section
                        .iter()
                        .map(|(key, value)| (key.to_owned(), value.to_owned()))
                        .collect(),
                )
            })
            .collect();
        let mut baseline = self.baseline().read(self.input.clone()).expect(self.name);
        let actual: Snapshot = baseline
            .iter()
            .map(|(name, section)| {
                (
                    name.clone(),
                    section
                        .iter()
                        .map(|(key, value)| (key.clone(), value.clone().expect("valued option")))
                        .collect(),
                )
            })
            .collect();
        assert_eq!(actual, expected, "{}: document mismatch", self.name);
        let mut actual = self.extract_baseline(&mut baseline);
        let mut expected = self.extract_astral(&astral);
        actual.sort();
        expected.sort();
        assert_eq!(actual, expected, "{}: extraction mismatch", self.name);
    }

    /// Include construction, lookup or extraction, and destruction of parser and output.
    pub(crate) fn run(&self, parser: Parser, mode: Mode) {
        match parser {
            Parser::Astral => {
                let ini = self
                    .options()
                    .parse(black_box(&self.input))
                    .expect("valid fixture");
                match mode {
                    Mode::Parse => drop(black_box(ini)),
                    Mode::Extract => drop(black_box(self.extract_astral(&ini))),
                }
            }
            Parser::Configparser => {
                let mut parser = self.baseline();
                let mut ini = parser
                    .read(black_box(&self.input).clone())
                    .expect("valid fixture");
                match mode {
                    Mode::Parse => drop(black_box(ini)),
                    Mode::Extract => drop(black_box(self.extract_baseline(&mut ini))),
                }
            }
        }
    }

    fn extract_astral(&self, ini: &astral_ini::Ini<'_>) -> Extracted {
        match self.workload {
            Workload::EntryPoints => Extracted::Scripts(
                ["console_scripts", "gui_scripts"]
                    .into_iter()
                    .filter_map(|name| ini.section(name))
                    .flat_map(|section| section.iter())
                    .map(|(key, value)| (key.to_owned(), value.to_owned()))
                    .collect(),
            ),
            Workload::SetupCfg => Extracted::Value(ini.get("metadata", "name").map(str::to_owned)),
            Workload::ExternallyManaged => {
                Extracted::Value(ini.get("externally-managed", "Error").map(str::to_owned))
            }
        }
    }

    fn extract_baseline(&self, ini: &mut Map) -> Extracted {
        match self.workload {
            Workload::EntryPoints => Extracted::Scripts(
                ["console_scripts", "gui_scripts"]
                    .into_iter()
                    .filter_map(|name| ini.get(name))
                    .flat_map(|section| section.iter())
                    .map(|(key, value)| {
                        (key.clone(), value.as_ref().expect("valued option").clone())
                    })
                    .collect(),
            ),
            Workload::SetupCfg => Extracted::Value(
                ini.get("metadata")
                    .and_then(|s| s.get("name"))
                    .cloned()
                    .flatten(),
            ),
            Workload::ExternallyManaged => Extracted::Value(
                ini.get_mut("externally-managed")
                    .and_then(|section| section.remove("Error"))
                    .flatten(),
            ),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Extracted {
    Scripts(Vec<(String, String)>),
    Value(Option<String>),
}

impl Extracted {
    fn sort(&mut self) {
        if let Self::Scripts(scripts) = self {
            scripts.sort_unstable();
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum Parser {
    Astral,
    Configparser,
}

impl Parser {
    pub(crate) const ALL: [Self; 2] = [Self::Astral, Self::Configparser];

    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Astral => "astral-ini",
            Self::Configparser => "configparser",
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum Mode {
    Parse,
    Extract,
}

impl Mode {
    pub(crate) const ALL: [Self; 2] = [Self::Parse, Self::Extract];

    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Parse => "parse",
            Self::Extract => "extract",
        }
    }
}

pub(crate) fn cases() -> Vec<Case> {
    let mut cases = Vec::new();
    macro_rules! captured {
        ($name:literal, $kind:ident) => {
            cases.push(Case {
                name: $name,
                input: include_str!(concat!("../../tests/fixtures/workloads/", $name, ".ini"))
                    .to_owned(),
                captured: true,
                workload: Workload::$kind,
            });
        };
    }
    captured!("entry-points-pip", EntryPoints);
    captured!("entry-points-pytest", EntryPoints);
    captured!("entry-points-babel", EntryPoints);
    captured!("entry-points-setuptools", EntryPoints);
    captured!("entry-points-virtualenv", EntryPoints);
    captured!("entry-points-keyring", EntryPoints);
    captured!("entry-points-cffi", EntryPoints);
    captured!("entry-points-pygments", EntryPoints);
    captured!("entry-points-uvicorn", EntryPoints);
    captured!("setup-cfg-uv", SetupCfg);
    captured!("setup-cfg-requests", SetupCfg);

    let synthetic = [
        ("empty", String::new(), Workload::EntryPoints),
        ("one-option", "[console_scripts]\na=b:c\n".to_owned(), Workload::EntryPoints),
        ("crlf", "[console_scripts]\r\na = b:c\r\n".to_owned(), Workload::EntryPoints),
        ("unicode", "[console_scripts]\nεκτέλεση = πακέτο:κύριο\n".to_owned(), Workload::EntryPoints),
        ("long-value", format!("[console_scripts]\na = {}:main\n", "package".repeat(4096)), Workload::EntryPoints),
        ("many-options", format!("[console_scripts]\n{}", (0..1000).map(|i| format!("script{i} = package{i}:main\n")).collect::<String>()), Workload::EntryPoints),
        ("many-sections", (0..256).map(|i| format!("[plugin{i}]\na = package:main\n")).collect(), Workload::EntryPoints),
        ("comments", format!("{}[metadata]\nname = example\n", "# a comment with no options\n".repeat(1000)), Workload::SetupCfg),
        ("setup-multiline", "[metadata]\nname = example\n[options]\ninstall_requires =\n    requests>=2\n    anyio>=4\n".to_owned(), Workload::SetupCfg),
        ("external-message", "[externally-managed]\nError = Use a virtual environment.\n    Run python -m venv .venv.\n\n    See your distributor's documentation.\n".to_owned(), Workload::ExternallyManaged),
        ("external-localized", "[externally-managed]\nError = Use a virtual environment.\nError-fr = Utilisez un environnement virtuel.\nError-de = Verwenden Sie eine virtuelle Umgebung.\n".to_owned(), Workload::ExternallyManaged),
        ("long-multiline", format!("[externally-managed]\nError = first\n{}", "    continuation\n".repeat(2000)), Workload::ExternallyManaged),
    ];
    cases.extend(synthetic.into_iter().map(|(name, input, workload)| Case {
        name,
        input,
        captured: false,
        workload,
    }));
    cases
}
