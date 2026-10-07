#!/usr/bin/env python3
"""Compare paired parser timings, preserving per-case and per-round results."""

import argparse
import csv
import json
from collections import defaultdict
from pathlib import Path
from statistics import geometric_mean


def analyze(paths):
    measurements = defaultdict(dict)
    for path in paths:
        for line in path.read_text().splitlines():
            row = json.loads(line)
            key = row["allocator"], row["mode"], row["case"]
            pair = row["parser"], row["round"]
            if pair in measurements[key]:
                raise ValueError(f"duplicate measurement: {key}, {pair}")
            samples = sorted(row["samples_ns"])
            if not samples or row["median_ns"] != samples[len(samples) // 2]:
                raise ValueError(f"incorrect median: {key}, {pair}")
            measurements[key][pair] = row

    cases = []
    grouped = defaultdict(list)
    for (allocator, mode, case), rows in sorted(measurements.items()):
        rounds = sorted({round for _, round in rows})
        expected = {(parser, round) for parser in ("astral-ini", "configparser") for round in rounds}
        if set(rows) != expected:
            raise ValueError(f"unpaired measurements: {allocator}, {mode}, {case}")
        kinds = {row["captured"] for row in rows.values()}
        sizes = {row["bytes"] for row in rows.values()}
        if len(kinds) != 1 or len(sizes) != 1:
            raise ValueError(f"inconsistent case metadata: {case}")
        captured = kinds.pop()
        ratios = [rows["astral-ini", round]["median_ns"] / rows["configparser", round]["median_ns"] for round in rounds]
        ratio = geometric_mean(ratios)
        cases.append({
            "allocator": allocator,
            "mode": mode,
            "case": case,
            "captured": captured,
            "bytes": sizes.pop(),
            "rounds": len(rounds),
            "astral_ns": geometric_mean(rows["astral-ini", round]["median_ns"] for round in rounds),
            "configparser_ns": geometric_mean(rows["configparser", round]["median_ns"] for round in rounds),
            "speedup": 1 / ratio,
            "change_percent": 100 * (ratio - 1),
            "min_change_percent": 100 * (min(ratios) - 1),
            "max_change_percent": 100 * (max(ratios) - 1),
            "slower_rounds": sum(value > 1 for value in ratios),
        })
        grouped[allocator, mode, captured].append(dict(zip(rounds, ratios, strict=True)))

    groups = []
    for (allocator, mode, captured), pairs in sorted(grouped.items()):
        rounds = sorted(pairs[0])
        if any(sorted(pair) != rounds for pair in pairs):
            raise ValueError(f"inconsistent rounds: {allocator}, {mode}, {captured}")
        ratios = [geometric_mean(pair[round] for pair in pairs) for round in rounds]
        groups.append({
            "allocator": allocator,
            "mode": mode,
            "captured": captured,
            "cases": len(pairs),
            "rounds": len(rounds),
            "speedup": 1 / geometric_mean(ratios),
            "change_percent": 100 * (geometric_mean(ratios) - 1),
            "min_change_percent": 100 * (min(ratios) - 1),
            "max_change_percent": 100 * (max(ratios) - 1),
        })
    if not cases:
        raise ValueError("no measurements")
    return cases, groups


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("inputs", nargs="+", type=Path, help="JSON lines from the parse benchmark")
    parser.add_argument("--output", type=Path, required=True, help="directory for summary.json and cases.csv")
    args = parser.parse_args()
    cases, groups = analyze(args.inputs)
    args.output.mkdir(parents=True, exist_ok=True)
    with (args.output / "cases.csv").open("w", newline="") as file:
        writer = csv.DictWriter(file, fieldnames=cases[0].keys(), lineterminator="\n")
        writer.writeheader()
        writer.writerows({key: round(value, 4) if isinstance(value, float) else value for key, value in case.items()} for case in cases)
    summary = {"groups": groups, "regressions": [case for case in cases if case["slower_rounds"]]}
    (args.output / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
