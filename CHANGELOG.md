# Changelog

## 0.0.1

Released on 2026-10-08.

### Enhancements

- Parse Python packaging INI files through a read-only API compatible with
  supported CPython 3.12 `ConfigParser` behavior, with opt-in case-sensitive
  names and `=`-only delimiters. (astral-sh/astral-ini#1,
  https://github.com/astral-sh/astral-ini/pull/1)

### Performance

- Reduce parsing time by 19.7–21.5% on captured packaging files by using faster
  section and option maps. (astral-sh/astral-ini#19,
  https://github.com/astral-sh/astral-ini/pull/19)
- Publish benchmarks against `configparser` across representative Python
  packaging workloads, including measurement provenance.
  (astral-sh/astral-ini#5, https://github.com/astral-sh/astral-ini/pull/5;
  astral-sh/astral-ini#21, https://github.com/astral-sh/astral-ini/pull/21;
  astral-sh/astral-ini#22, https://github.com/astral-sh/astral-ini/pull/22)
- Reduce parsing time by 1.7–2.7% on captured packaging files by avoiding
  redundant value trimming. (astral-sh/astral-ini#9,
  https://github.com/astral-sh/astral-ini/pull/9)

### Other changes

- Document the supported `ConfigParser` compatibility contract, including
  syntax, error behavior, and pinned Unicode casing. (astral-sh/astral-ini#2,
  https://github.com/astral-sh/astral-ini/pull/2; astral-sh/astral-ini#13,
  https://github.com/astral-sh/astral-ini/pull/13)
- Include the Apache 2.0 and MIT license texts in the published crate.
  (astral-sh/astral-ini#6, https://github.com/astral-sh/astral-ini/pull/6)
- Point the published crate metadata to the current repository.
  (astral-sh/astral-ini#11, https://github.com/astral-sh/astral-ini/pull/11)
