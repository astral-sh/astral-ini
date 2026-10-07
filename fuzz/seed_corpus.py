#!/usr/bin/env python3
"""Seed fuzz targets from Python conformance cases and uv workload fixtures."""

import argparse
import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
TARGETS = ("reader", "lookup", "python", "structured")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("target", nargs="?", choices=TARGETS)
    arguments = parser.parse_args()
    fixtures = ROOT / "crates/astral-ini/tests/fixtures"
    cases = json.loads((fixtures / "python.json").read_text())["cases"]
    inputs = {case["input"].encode() for case in cases}
    inputs.update(path.read_bytes() for path in (fixtures / "workloads").glob("*.ini"))
    inputs.update(
        (
            b"",
            b"\xff",
            b"[s]\nkey=\xff\xfe\x80\xc0\xaf\n",
            b"[s]\n\xed\xa0\x80=value\n",
            b"[s]\n\xf4\x90\x80\x80=value\n",
        )
    )
    structured = {
        bytes((line, name, value, newline))
        for line in range(12)
        for name in range(11)
        for value in range(10)
        for newline in range(3)
    }
    # Start with multi-line state transitions as well as individual grammar rules.
    structured.update(bytes((0, name, 1, 0, 3, name, 2, 0, 7, 0, 0, 0, 3, name, 3, 2)) for name in range(11))
    report = {}
    for target in (arguments.target,) if arguments.target else TARGETS:
        corpus = ROOT / "fuzz/generated" / target
        corpus.mkdir(parents=True, exist_ok=True)
        seeds = structured if target == "structured" else inputs
        for data in seeds:
            (corpus / hashlib.sha256(data).hexdigest()).write_bytes(data)
        report[target] = {"seeds": len(seeds), "bytes": sum(map(len, seeds))}
    print(json.dumps(report, sort_keys=True))


if __name__ == "__main__":
    main()
