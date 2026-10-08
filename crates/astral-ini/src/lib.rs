//! Read-only INI parsing for Python packaging.
//!
//! Parsing follows CPython 3.12's `ConfigParser(interpolation=None)`: duplicate
//! sections and options are errors, option names are lowercased using Unicode 15,
//! and `DEFAULT` options are inherited. Use [`Options`] for case-sensitive names
//! and `=`-only delimiters.
//!
//! ```
//! use astral_ini::{Delimiters, Options};
//!
//! let ini = Options::default()
//!     .case_sensitive(true)
//!     .delimiters(Delimiters::Equals)
//!     .parse("[console_scripts]\nhello = example:main\n")?;
//! assert_eq!(ini.get("console_scripts", "hello"), Some("example:main"));
//! # Ok::<(), astral_ini::Error>(())
//! ```

mod unicode;

use std::borrow::Cow;
use std::fmt;

use indexmap::IndexMap;
use rustc_hash::FxBuildHasher;

type Properties<'a> = IndexMap<Cow<'a, str>, Cow<'a, str>, FxBuildHasher>;

/// Delimiters accepted between an option name and its value.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Delimiters {
    /// Accept `=` only, as required by `entry_points.txt`.
    Equals,
    /// Accept either `=` or `:`; the first delimiter separates the value.
    #[default]
    EqualsAndColon,
}

/// Parsing options for the supported Python INI dialects.
///
/// Section names are always case-sensitive. Only full-line `#` and `;` comments
/// are removed; inline comment markers and interpolation syntax remain literal.
/// Valueless options and unnamed sections are rejected.
#[derive(Debug, Clone, Copy, Default)]
pub struct Options {
    case_sensitive: bool,
    delimiters: Delimiters,
}

impl Options {
    /// Preserve option names instead of applying Python's Unicode lowercase transform.
    #[must_use]
    pub const fn case_sensitive(mut self, enabled: bool) -> Self {
        self.case_sensitive = enabled;
        self
    }

    /// Select the separators accepted between option names and values.
    #[must_use]
    pub const fn delimiters(mut self, delimiters: Delimiters) -> Self {
        self.delimiters = delimiters;
        self
    }

    /// Parse UTF-8 text with universal newlines (`LF`, `CRLF`, or `CR`).
    ///
    /// Names and single-line values borrow the input when normalization is not
    /// required. Continuations are joined with `\n`. Errors contain a one-based
    /// line number; malformed lines are deferred as in Python's parser, so a
    /// later duplicate can take precedence.
    pub fn parse(self, input: &str) -> Result<Ini<'_>, Error> {
        let mut ini = Ini {
            sections: IndexMap::default(),
            defaults: IndexMap::default(),
            case_sensitive: self.case_sensitive,
        };
        let mut section = None;
        let mut option = None;
        let mut indent = 0;
        let mut blanks = 0;
        let mut invalid = None;

        for (index, line) in Lines(input).enumerate() {
            let line_number = index + 1;
            let value = line.trim_matches(whitespace);
            if value.starts_with(['#', ';']) {
                continue;
            }
            if value.is_empty() {
                if option.is_some() {
                    blanks += 1;
                }
                continue;
            }
            let current_indent = line.chars().take_while(|&c| whitespace(c)).count();
            if let (Some(section), Some(option)) = (section, option)
                && current_indent > indent
            {
                let properties = ini.properties_mut(section);
                let (_, previous) = properties.get_index_mut(option).unwrap();
                let previous = previous.to_mut();
                previous.reserve(blanks + 1 + value.len());
                previous.extend(std::iter::repeat_n('\n', blanks + 1));
                previous.push_str(value);
                blanks = 0;
                continue;
            }
            indent = current_indent;
            blanks = 0;
            if let Some(end) = value
                .strip_prefix('[')
                .and_then(|s| s.rfind(']').map(|i| i + 1))
                && end > 1
            {
                let name = &value[1..end];
                section = Some(if name == "DEFAULT" {
                    SectionIndex::Defaults
                } else {
                    let (index, replaced) = ini.sections.insert_full(name, IndexMap::default());
                    if replaced.is_some() {
                        return Err(Error::new(ErrorKind::DuplicateSection, line_number));
                    }
                    SectionIndex::Named(index)
                });
                option = None;
                continue;
            }
            let Some(section) = section else {
                return Err(Error::new(ErrorKind::MissingSection, line_number));
            };
            let delimiter = match self.delimiters {
                Delimiters::Equals => memchr::memchr(b'=', value.as_bytes()),
                Delimiters::EqualsAndColon => memchr::memchr2(b'=', b':', value.as_bytes()),
            };
            let Some(delimiter) = delimiter else {
                invalid.get_or_insert(Error::new(ErrorKind::InvalidLine, line_number));
                continue;
            };
            let name = value[..delimiter].trim_end_matches(whitespace);
            if name.is_empty() {
                invalid.get_or_insert(Error::new(ErrorKind::InvalidLine, line_number));
            }
            let name = normalize(name, self.case_sensitive);
            let value = value[delimiter + 1..].trim_start_matches(whitespace);
            let properties = ini.properties_mut(section);
            let (index, replaced) = properties.insert_full(name, Cow::Borrowed(value));
            if replaced.is_some() {
                return Err(Error::new(ErrorKind::DuplicateOption, line_number));
            }
            option = (!properties.get_index(index).unwrap().0.is_empty()).then_some(index);
        }
        if let Some(error) = invalid {
            return Err(error);
        }
        Ok(ini)
    }
}

/// An immutable INI document whose strings borrow the input where possible.
#[derive(Debug, Clone)]
pub struct Ini<'a> {
    sections: IndexMap<&'a str, Properties<'a>, FxBuildHasher>,
    defaults: Properties<'a>,
    case_sensitive: bool,
}

impl<'a> Ini<'a> {
    /// Parse with Python's default delimiters and case-insensitive option names.
    ///
    /// See [`Options::parse`] for newline handling, borrowing, and error precedence.
    pub fn parse(input: &'a str) -> Result<Self, Error> {
        Options::default().parse(input)
    }

    /// Return sections in input order, excluding `DEFAULT`.
    pub fn sections(&self) -> impl Iterator<Item = (&'a str, Section<'_, 'a>)> {
        self.sections
            .iter()
            .map(|(&name, properties)| (name, self.view(properties)))
    }

    /// Return explicitly defined defaults in input order.
    pub fn defaults(&self) -> impl Iterator<Item = (&str, &str)> {
        self.defaults
            .iter()
            .map(|(name, value)| (name.as_ref(), value.as_ref()))
    }

    /// Return a section, including its inherited defaults. `DEFAULT` is always present.
    pub fn section(&self, name: &str) -> Option<Section<'_, 'a>> {
        let properties = if name == "DEFAULT" {
            &self.defaults
        } else {
            self.sections.get(name)?
        };
        Some(self.view(properties))
    }

    /// Look up a value, falling back to `DEFAULT` only when the section exists.
    pub fn get(&self, section: &str, name: &str) -> Option<&str> {
        self.section(section)?.get(name)
    }

    fn view<'s>(&'s self, properties: &'s Properties<'a>) -> Section<'s, 'a> {
        Section {
            properties,
            defaults: &self.defaults,
            case_sensitive: self.case_sensitive,
        }
    }

    fn properties_mut(&mut self, section: SectionIndex) -> &mut Properties<'a> {
        match section {
            SectionIndex::Defaults => &mut self.defaults,
            SectionIndex::Named(index) => self.sections.get_index_mut(index).unwrap().1,
        }
    }
}

/// A section with read-only access to its own and inherited options.
#[derive(Debug, Clone, Copy)]
pub struct Section<'s, 'a> {
    properties: &'s Properties<'a>,
    defaults: &'s Properties<'a>,
    case_sensitive: bool,
}

impl<'s> Section<'s, '_> {
    /// Look up an option using the document's case sensitivity, including defaults.
    ///
    /// Lookup preserves surrounding whitespace. Case-insensitive lookups use
    /// Python's lowercase transform, which keeps `ß` and `ss` distinct.
    pub fn get(self, name: &str) -> Option<&'s str> {
        let name = normalize(name, self.case_sensitive);
        self.properties
            .get(name.as_ref())
            .or_else(|| self.defaults.get(name.as_ref()))
            .map(Cow::as_ref)
    }

    /// Iterate local options first, then inherited defaults not overridden locally.
    /// Each group retains input order, matching Python's section proxy.
    pub fn iter(self) -> impl Iterator<Item = (&'s str, &'s str)> {
        self.properties
            .iter()
            .chain(
                self.defaults
                    .iter()
                    .filter(move |(name, _)| !self.properties.contains_key(name.as_ref())),
            )
            .map(|(name, value)| (name.as_ref(), value.as_ref()))
    }
}

#[derive(Clone, Copy)]
enum SectionIndex {
    Defaults,
    Named(usize),
}

/// The reason an INI document could not be parsed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    /// An option or other non-comment content occurred before any section.
    MissingSection,
    /// A named section appeared more than once.
    DuplicateSection,
    /// An option appeared more than once in the same section after case normalization.
    DuplicateOption,
    /// A line was neither a section, an option with a name and value delimiter, nor a continuation.
    InvalidLine,
}

/// A parsing failure and its one-based input line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Error {
    kind: ErrorKind,
    line: usize,
}

impl Error {
    fn new(kind: ErrorKind, line: usize) -> Self {
        Self { kind, line }
    }

    /// Return the reason parsing failed.
    pub const fn kind(self) -> ErrorKind {
        self.kind
    }

    /// Return the one-based line where the error was detected.
    pub const fn line(self) -> usize {
        self.line
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let reason = match self.kind {
            ErrorKind::MissingSection => "expected a section header",
            ErrorKind::DuplicateSection => "duplicate section",
            ErrorKind::DuplicateOption => "duplicate option",
            ErrorKind::InvalidLine => "invalid option or section header",
        };
        write!(f, "{reason} at line {}", self.line)
    }
}

impl std::error::Error for Error {}

/// Python's `str.isspace()` also includes the ASCII information separators.
fn whitespace(c: char) -> bool {
    c.is_whitespace() || matches!(c, '\u{1c}'..='\u{1f}')
}

/// Apply Python's option-name transform, borrowing names that remain unchanged.
fn normalize(name: &str, case_sensitive: bool) -> Cow<'_, str> {
    if case_sensitive || (name.is_ascii() && !name.bytes().any(|b| b.is_ascii_uppercase())) {
        return Cow::Borrowed(name);
    }
    let lower = unicode::lowercase(name);
    if lower == name {
        Cow::Borrowed(name)
    } else {
        Cow::Owned(lower)
    }
}

/// File-style universal newlines without treating other Unicode whitespace as a line break.
struct Lines<'a>(&'a str);

impl<'a> Iterator for Lines<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.is_empty() {
            return None;
        }
        let Some(end) = memchr::memchr2(b'\r', b'\n', self.0.as_bytes()) else {
            return Some(std::mem::take(&mut self.0));
        };
        let line = &self.0[..end];
        let crlf =
            self.0.as_bytes()[end] == b'\r' && self.0.as_bytes().get(end + 1) == Some(&b'\n');
        self.0 = &self.0[end + 1 + usize::from(crlf)..];
        Some(line)
    }
}
