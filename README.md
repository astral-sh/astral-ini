# astral-ini

A high-performance INI parser designed for Python packaging.

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
`DEFAULT` inheritance. `Options` selects case-sensitive names and `=`-only
separators for `entry_points.txt`.

The reader accepts UTF-8 text and borrows names and values where possible.
Interpolation, writing, mutation, and merging multiple files are outside its
scope. See [conformance](docs/conformance.md) for the supported Python behavior.

## Benchmarks

Owned extraction from captured packaging files:

| Parser | pip | Babel | setuptools | virtualenv | Requests setup.cfg |
| --- | ---: | ---: | ---: | ---: | ---: |
| astral-ini | 0.38 | 1.56 | 4.20 | 2.46 | 1.17 |
| configparser 3.2.0 | 1.99 | 9.13 | 33.87 | 17.57 | 5.69 |

Times in microseconds on Linux x86-64 with the system allocator; lower is better.
See [performance](docs/performance.md) for all workloads and measurement details.

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
