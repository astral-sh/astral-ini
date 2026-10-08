# astral-ini

A high-performance INI parser designed for Python packaging.

> [!WARNING]
>
> This README was written by a human, but all code changes, PR summaries, and
> additional documentation were authored entirely by GPT-6 Astra in Codex.

## Highlights

- Three runtime dependencies by default: `indexmap`, `memchr`, and `rustc-hash`.
- No unsafe Rust in the library.
- Tested against Python's `ConfigParser` with interpolation disabled.

## Benchmarks

Parse INI, collect owned entry-point names and values or the package name from
`setup.cfg`, and drop the output (`astral_ini::Ini`).

| Parser             |  pip | Babel | setuptools | virtualenv | Requests setup.cfg |
| ------------------ | ---: | ----: | ---------: | ---------: | -----------------: |
| astral-ini         | 0.34 |  1.27 |       3.36 |       1.99 |               0.97 |
| configparser 3.2.0 | 2.06 |  9.26 |      34.34 |      17.97 |               5.98 |

<sub>Times in microseconds (µs);
[lower is better](https://github.com/astral-sh/astral-ini/blob/4b5d0b59810fd680af966a4563f0656bea23d2fb/docs/performance.md).</sub>

## Example usage

Use `Ini` to read UTF-8 text and look up values. Names and values borrow the
input where possible.

For example, to read a package name from `setup.cfg`:

```rust
use astral_ini::Ini;

let ini = Ini::parse("[metadata]\nname = example\n")?;

assert_eq!(ini.get("metadata", "name"), Some("example"));
```

For `entry_points.txt`, use `Options` to preserve option names and accept only
`=` separators:

```rust
use astral_ini::{Delimiters, Options};

let ini = Options::default()
    .case_sensitive(true)
    .delimiters(Delimiters::Equals)
    .parse("[console_scripts]\nhello = example:main\n")?;

assert_eq!(ini.get("console_scripts", "hello"), Some("example:main"));
```

`Ini::parse` follows Python's `ConfigParser(interpolation=None)`, with
case-insensitive option names, strict duplicate detection, multiline values, and
`DEFAULT` inheritance. Interpolation, writing, mutation, and merging multiple
files are outside its scope. See [conformance](docs/conformance.md) for the
supported Python behavior.

## License

astral-ini is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  <https://opensource.org/licenses/MIT>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in astral-ini by you, as defined in the Apache-2.0 license, shall be
dually licensed as above, without any additional terms or conditions.
