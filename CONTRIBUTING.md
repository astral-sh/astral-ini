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

See [Conformance](docs/conformance.md) for the compatibility contract and corpus
generator, and [Performance](docs/performance.md) for benchmarks and allocation
measurements.

## Fuzzing

| Target | Checks |
| --- | --- |
| `reader` | Newline normalization and error locations across all four parser profiles |
| `lookup` | Section and option lookups agree with iteration, including inherited defaults and name normalization |
| `python` | Arbitrary text against Python's defaults, section items, and error category and line |
| `lookup_python` | Independently mutated documents, section names, and option names against Python's lookup results |
| `structured` | Grammar-generated sections, options, continuations, comments, Unicode, and malformed lines against Python |

The byte-input targets replace invalid UTF-8 with U+FFFD before parsing: the
library accepts Rust strings, so decoding invalid bytes is the caller's policy.
The differential targets check all four combinations of option-name casing and
delimiters through a persistent Python process. The structured target generates
at most 128 lines per input; this bounds fuzzing work, not the parser API.

The seed corpus includes all Python conformance inputs, captured uv workloads,
invalid UTF-8 sequences, lookup hits and misses, and grammar rules. Install a
nightly Rust toolchain and run from the repository root:

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

## Releases

Releases can only be performed by Astral team members.

### Setup

Before the first release, configure the protected `release-gate` and `release`
environments and release tag rules through
[Astral's GitHub policies](https://github.com/astral-sh/github-policies).
Register `astral-ini` in
[crates-policies](https://github.com/astral-sh/crates-policies) and run its
**Apply** workflow to bootstrap the crate and configure Trusted Publishing for
this repository's `release.yml` workflow and `release` environment. Crates.io
authentication uses OIDC; this repository does not need a `CARGO_REGISTRY_TOKEN`
secret.

Give the existing `astral-automations-bot` and `astral-releases-bot` GitHub Apps
access to this repository. Create an `automations` environment restricted to
`main` and configure these secrets:

| Environment   | Secret                 | Value                                                     |
| ------------- | ---------------------- | --------------------------------------------------------- |
| `automations` | `STS_API_URL`          | Astral's automation-broker base URL, without `/exchange`. |
| `automations` | `OPENAI_API_KEY`       | An API key for the Codex changelog rewrite.               |
| `release`     | `RELEASES_STS_API_URL` | Astral's release-broker base URL, without `/exchange`.    |

The brokers read `.github/secure-token-service.json` and
`.github/secure-token-service-release.json` from `main`, so merge these policies
before running the workflows.

### Prepare and publish

1. Run **Prepare release** from `main`. Leave `version` empty for automatic
   detection, or provide an exact stable Cargo version without a leading `v`.
   The first release defaults to the version already in `Cargo.toml`.
2. Review the generated version changes and changelog, then merge the release
   PR.
3. Run **Release** from `main` with the prepared version. Select **Dry-run** to
   validate the package and release notes without publishing.
4. When publishing, approve the protected `release-gate` deployment.

Rooster chooses the next version from merged pull requests: `breaking` selects a
minor bump, other changes select a patch bump, and internal changes are
excluded. Preparation updates the workspace version and both Cargo lockfiles,
rewrites the newest changelog section with Codex, and assigns the release PR to
the person who started the workflow.

Release validates the version, changelog, and package before publishing through
Trusted Publishing in the `release` environment. It then creates the
`v<version>` tag and GitHub release with the prepared notes. Retries skip the
crate upload if that version already exists on crates.io.
