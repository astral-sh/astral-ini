# /// script
# requires-python = ">=3.12"
# [tool.uv]
# no-build = true
# exclude-newer = "P7D"
# ///

"""Replace the latest changelog section."""

from __future__ import annotations

import argparse
from pathlib import Path


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("changelog", type=Path)
    parser.add_argument("candidate", type=Path)
    args = parser.parse_args()

    changelog = args.changelog.read_text(encoding="utf-8")
    candidate = args.candidate.read_text(encoding="utf-8").rstrip("\n")
    sections = changelog.split("\n## ", maxsplit=2)
    headings = [line for line in candidate.splitlines() if line.startswith("## ")]
    if len(sections) < 2 or headings != [f"## {sections[1].splitlines()[0]}"]:
        raise ValueError("Replacement must contain exactly the newest release heading")
    preamble = sections[0]
    historical_releases = f"\n\n## {sections[2]}" if len(sections) == 3 else "\n"
    args.changelog.write_text(
        f"{preamble}\n{candidate}{historical_releases}",
        encoding="utf-8",
    )


if __name__ == "__main__":
    main()
