#!/usr/bin/env python3
"""Check every Unicode scalar and its sigma contexts against CPython 3.12."""

import argparse
import json
from pathlib import Path
import shlex
import subprocess
import tempfile
import unicodedata

MODULE = Path(__file__).resolve().parents[1] / "crates/astral-ini/src/unicode.rs"
HARNESS = r'''
#[path = MODULE_PATH]
mod unicode;

fn main() {
    use std::io::Write;
    let mut output = std::io::BufWriter::new(std::io::stdout().lock());
    for codepoint in 0..=0x10ffff {
        let Some(character) = char::from_u32(codepoint) else { continue };
        let source = character.to_string();
        let lower = unicode::lowercase(&source);
        let left = unicode::lowercase(&format!(" {character}Σ")).ends_with('ς');
        let right = unicode::lowercase(&format!("AΣ{character}A")).chars().nth(1) == Some('ς');
        if lower != source || left || !right {
            let mapping = lower.chars().map(|c| format!("{:x}", u32::from(c))).collect::<Vec<_>>().join(",");
            writeln!(output, "{codepoint:x} {mapping} {} {}", u8::from(left), u8::from(right)).unwrap();
        }
    }
}
'''


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--rustc", default="rustc", help="Compiler command")
    arguments = parser.parse_args()
    if unicodedata.unidata_version != "15.0.0":
        raise SystemExit("Run this check with CPython 3.12 (Unicode 15.0.0)")
    with tempfile.TemporaryDirectory(prefix="astral-ini-unicode-") as directory:
        source = Path(directory) / "check.rs"
        executable = Path(directory) / "check"
        source.write_text(HARNESS.replace("MODULE_PATH", json.dumps(str(MODULE))))
        subprocess.run(
            [*shlex.split(arguments.rustc), "--edition=2024", "-O", str(source), "-o", str(executable)],
            check=True,
        )
        result = subprocess.run([executable], check=True, text=True, capture_output=True)
    actual = iter(result.stdout.splitlines())
    checked = 0
    for codepoint in range(0x110000):
        if 0xD800 <= codepoint <= 0xDFFF:
            continue
        character = chr(codepoint)
        lower = character.lower()
        left = (" " + character + "Σ").lower().endswith("ς")
        right = ("AΣ" + character + "A").lower()[1] == "ς"
        if lower != character or left or not right:
            mapping = ",".join(f"{ord(c):x}" for c in lower)
            expected = f"{codepoint:x} {mapping} {int(left)} {int(right)}"
            received = next(actual, None)
            if received != expected:
                raise SystemExit(f"Unicode mismatch: expected {expected!r}, got {received!r}")
        checked += 1
    if next(actual, None) is not None:
        raise SystemExit("Unexpected extra Unicode mappings")
    print(f"Checked lowercase and both sigma contexts for {checked:,} Unicode scalars")


if __name__ == "__main__":
    main()
