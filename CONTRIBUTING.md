# Contributing

## Checks

```console
cargo test --workspace --all-targets --all-features --locked
cargo test --workspace --doc --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo fmt --all --check
cargo fmt --manifest-path fuzz/Cargo.toml --all --check
```

Check the committed compatibility corpus against CPython 3.12.13:

```console
python3 scripts/generate_conformance.py --check
python3 scripts/generate_unicode.py --check
python3 scripts/check_unicode.py
```

Use `--write` to regenerate it. The generator's `--stdin` mode accepts JSON lines
containing `input`, `case_sensitive`, and `delimiters` (`"="` or `"=:"`). It returns
ordered defaults and section items, or the error category and one-based line.
Parsing uses `ConfigParser(interpolation=None)` and universal newlines.
The Unicode check compares every Unicode scalar's lowercase mapping and sigma
contexts against Python. See [Performance](docs/performance.md) for equivalent
workload benchmarks and allocation measurements.

## Fuzzing

| Target | Checks |
| --- | --- |
| `reader` | Newline normalization and error locations across all four parser profiles |
| `lookup` | Section and option lookups agree with iteration, including inherited defaults and case folding |
| `python` | Arbitrary text against Python's defaults, section items, and error category and line |
| `structured` | Grammar-generated sections, options, continuations, comments, Unicode, and malformed lines against Python |

The byte-input targets replace invalid UTF-8 with U+FFFD before parsing: the
library accepts Rust strings, so decoding invalid bytes is the caller's policy.
Both differential targets check all four combinations of option-name casing and
delimiters through a persistent Python process. The structured target generates
at most 128 lines per input; this bounds fuzzing work, not the parser API.

The seed corpus includes all Python conformance inputs, captured uv workloads,
invalid UTF-8 sequences, and grammar rules. Install a nightly Rust toolchain and
run from the repository root:

```console
cargo install cargo-fuzz --version 0.13.2 --locked
python3 fuzz/seed_corpus.py
cargo +nightly fuzz run python fuzz/generated/python -- -dict=fuzz/ini.dict -max_len=16384 -len_control=0 -max_total_time=900 -timeout=5 -rss_limit_mb=2048 -print_final_stats=1
```

Use CPython 3.12.13 for the differential targets; set `ASTRAL_INI_PYTHON` when it
is not `python3`. Its subprocess has a separate 512 MiB address-space limit.
Replace `python` in the command and corpus path to run another target. Reproduce
and minimize saved failures with `cargo +nightly fuzz run TARGET PATH` and
`cargo +nightly fuzz tmin TARGET PATH`, then add a regression case to the Python
fixture generator.

CI runs each target with AddressSanitizer for 30 seconds on pull requests and
900 seconds on scheduled or manual runs. Campaign artifacts contain logs,
compiler and Python versions, source and seed hashes, the resulting corpus, and
any failures. The shared corpus is updated only by successful runs on `main`.
