# Performance

The benchmark compares `astral-ini` with `configparser` 3.2.0 using uv's current
`Ini::read` API. Both parsers receive the same valid documents, and the harness
checks full parsed output and extracted values for equality before timing.

On a Linux x86-64 AMD EPYC Milan VM, pinned to CPU 26, the 2026-10-07 run measured
these geometric speedups over `configparser`. Parentheses show the range of the
six round aggregates, not confidence intervals.

| Inputs | Operation | System allocator | jemalloc |
| --- | --- | --- | --- |
| 11 captured files | Parse and drop | 6.63× (6.56–6.69×) | 5.67× (5.59–5.87×) |
| 11 captured files | Owned extraction | 5.76× (5.68–5.84×) | 4.98× (4.94–5.01×) |
| 12 synthetic cases | Parse and drop | 8.23× (8.17–8.30×) | 6.97× (6.91–7.16×) |
| 12 synthetic cases | Owned extraction | 6.71× (6.67–6.77×) | 5.84× (5.81–5.88×) |

Every case, operation, and allocator took less time in all six paired rounds.
The smallest geometric speedup was 1.84× for the synthetic comment-heavy parse;
the smallest improvement in any individual round was 43.6% less time.
[`performance.csv`](performance.csv) includes all 92 case/operation/allocator
comparisons, absolute times, and round ranges.

Removing the redundant trailing-value trim reduced captured-file parse time by
1.69% with the system allocator and 2.72% with jemalloc; owned extraction improved
by 2.23% and 3.63%. The synthetic Unicode case cost about 6 ns more to parse and
10 ns more to extract with the system allocator, consistently across all six
rounds. Captured `configparser` control aggregates also shifted between builds
(−1.75% to +1.16%); these comparisons use direct timings without normalization.
[`trim-comparison.csv`](trim-comparison.csv) includes every case and its paired
control, including the small costs that aggregate improvements can hide.

Across the 11 captured files, parsing made 110 allocation or reallocation
requests versus 1,162, requesting 16,061 versus 59,239 bytes.
Owned extraction made 139 allocation or reallocation
requests versus 1,191, requesting 17,918 versus 61,096 bytes.
No case increased total allocation requests or requested bytes. Some cases made
more reallocations as ordered maps grew, while making fewer allocations overall.

The binaries used rustc 1.98.1-dev (`f6270311094`), optimization level 3, thin
LTO, and one codegen unit. Both allocator builds used identical source files;
source and binary hashes were checked before and after timing. The run contains
1,104 observations and 20,976 samples. Compiler, source, binary, and raw-result
hashes are recorded in [`performance-provenance.json`](performance-provenance.json).
The binaries were built in isolated directories. The run used six balanced
orders of three parser builds, shuffled cases, and alternating allocator order;
the table uses this build and its own `configparser` timings.

```console
cargo bench --bench parse > system.jsonl
cargo bench --bench parse --features benchmark-jemalloc > jemalloc.jsonl
cargo run --release --example allocations > allocations.jsonl
python3 scripts/analyze_benchmarks.py system.jsonl jemalloc.jsonl --output results
```

`parse` measures parsing and destruction of the complete result. `extract` also
owns the names and raw values of console and GUI entry points, the package name
from `setup.cfg`, or the message from `EXTERNALLY-MANAGED`. Entry-point validation,
Python object-reference parsing, filesystem access, and installation are outside
the measurement. These are parser and extraction workloads, not uv end-to-end
timings.

The baseline includes its required owned input and the map clone returned by
`Ini::read`; the new parser borrows the input. `configparser` has no public parsing
API that omits that result clone. Options and parser state are constructed for
each document, as they are in uv. Every timed iteration drops the parser and its
output. File loading, fixture generation, and equality checks are excluded.
Both parsers use case-sensitive option names for these workloads, matching uv.
The baseline moves the external-management message out of its owned map, as uv
does, while `astral-ini` copies its borrowed value into the owned result.

Eleven captured files cover nine public packages' entry points and two
`setup.cfg` files. Twelve synthetic cases cover tiny inputs, Unicode, CRLF,
comments, long values, many options and sections, continuations, and localized
external-management messages. Source versions, hashes, and licenses are in
[`sources.json`](../crates/astral-ini/tests/fixtures/workloads/sources.json).
The captured urllib3 file is checked for Python compatibility but excluded from
the comparison: `configparser` truncates its semicolon environment markers.

By default, each case has six rounds with 19 samples per parser and operation.
The parser order alternates each round; each sample measures a calibrated batch
lasting at least approximately 3 ms after a 10 ms warmup. JSON lines include raw
samples, batch size, allocator, and round. Run on an otherwise idle machine,
optionally pinning the executable to one CPU. Compare paired rounds per case and
report captured and synthetic results separately; aggregate speedups alone can
hide regressions.

Debug builds, including default `cargo test --all-targets`, run equality checks
without timing.
Use `cargo bench` for measurements. Filter or shorten a run with:

```console
cargo bench --bench parse -- --list
cargo bench --bench parse -- --case entry-points --mode extract --rounds 2 --samples 9
```

`--parser astral-ini` or `--parser configparser` selects one implementation;
`--sample-ms` and `--warmup-ms` control sample and warmup duration. Allocation
counts run separately with the instrumented system allocator so its counters do
not affect timing. `bytes_allocated` counts allocation requests and positive
reallocation growth; it is not peak live memory. Do not add `bytes_reallocated`
to it again.
