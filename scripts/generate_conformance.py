#!/usr/bin/env python3
"""Compare the reader with CPython's raw ConfigParser file semantics."""

import argparse
import configparser
import hashlib
import io
import itertools
import json
from pathlib import Path
import random
import sys
import unicodedata

FIXTURE = Path(__file__).resolve().parents[1] / "crates/astral-ini/tests/fixtures/python.json"


def parse(request):
    parser = configparser.ConfigParser(
        interpolation=None,
        delimiters=tuple(request["delimiters"]),
    )
    if request["case_sensitive"]:
        parser.optionxform = str
    try:
        parser.read_file(io.StringIO(request["input"], newline=None))
    except configparser.MissingSectionHeaderError as error:
        return {"error": {"kind": "MissingSection", "line": error.lineno}}
    except configparser.DuplicateSectionError as error:
        return {"error": {"kind": "DuplicateSection", "line": error.lineno}}
    except configparser.DuplicateOptionError as error:
        return {"error": {"kind": "DuplicateOption", "line": error.lineno}}
    except configparser.ParsingError as error:
        return {"error": {"kind": "InvalidLine", "line": error.errors[0][0]}}
    return {
        "defaults": list(map(list, parser.defaults().items())),
        "sections": [
            [name, list(map(list, parser[name].items()))] for name in parser.sections()
        ],
    }


def inputs():
    focused = {
        "empty": "",
        "comments-only": "# first\n  ; second\n\n",
        "entry-points": "[console_scripts]\nPyTool=package.cli:main [extra]\npytool=package.cli:other\n[gui_scripts]\nwidget=package.gui:run\n",
        "externally-managed": "[externally-managed]\nError=This environment is managed.\n Use a virtual environment.\n\n See the documentation.\nError-de_DE=Diese Umgebung ist verwaltet.\n",
        "setup-cfg": "[metadata]\nname=some-project\n[options]\ninstall_requires=\n requests>=2\n packaging; python_version > '3.8'\n",
        "defaults-before": "[DEFAULT]\nFirst=1\nSecond=2\n[s]\nLocal=3\nFirst=4\n[t]\n",
        "defaults-after": "[s]\nLocal=3\nFirst=4\n[DEFAULT]\nFirst=1\nSecond=2\n[t]\n",
        "repeated-default": "[DEFAULT]\na=1\n[DEFAULT]\nb=2\n[s]\n",
        "repeated-default-option": "[DEFAULT]\na=1\n[s]\n[DEFAULT]\na=2\n",
        "case-distinct-sections": "[s]\nx=1\n[S]\nx=2\n[default]\nx=3\n",
        "duplicate-section": "[s]\na=1\n[s]\nb=2\n",
        "duplicate-option": "[s]\na=1\na=2\n",
        "duplicate-folded-option": "[s]\nKEY=1\nkey=2\n",
        "sectionless": "a=b\n[s]\nc=d\n",
        "valueless": "[s]\nflag\n",
        "empty-key": "[s]\n=empty\n",
        "empty-colon-key": "[s]\n:empty\n",
        "invalid-before-duplicate-section": "[s]\ninvalid\n[s]\n",
        "invalid-before-duplicate-option": "[s]\ninvalid\na=1\na=2\n",
        "invalid-before-missing-section": "invalid\n[s]\ninvalid\n",
        "multiple-invalid": "[s]\nfirst invalid\nsecond invalid\n",
        "invalid-after-option": "[s]\na=1\ninvalid\n  continuation\n",
        "invalid-before-option": "[s]\ninvalid\na=1\n  continuation\n",
        "blank-section": "[]\na=1\n",
        "space-section": "[ ]\na=1\n",
        "section-whitespace": "  [  s  ]  \na=1\n",
        "header-trailing-text": "[s] trailing text\na=1\n",
        "header-last-bracket": "[s] and [t] trailing\na=1\n",
        "header-nested": "[[s]]\na=1\n",
        "unterminated-header-as-option": "[s]\n[t=1\n",
        "unterminated-header": "[s]\n[t\n",
        "indented-header-continuation": "[s]\na=1\n  [t]\n b=2\n",
        "section-resets-continuation": "[s]\na=1\n[t]\n b=2\n",
        "indented-first-option": "  [s]\n a=1\n b=2\n",
        "indent-relative": "[s]\n  a=1\n   b=2\n  c=3\n d=4\n",
        "tab-counts-one": "[s]\n\ta=1\n b=2\n  c=3\n",
        "unicode-indent-count": "[s]\n\u00a0a=1\n b=2\n  c=3\n",
        "blank-continuation": "[s]\na=1\n\n\n  two\n\n",
        "blank-before-next-option": "[s]\na=1\n\n\nb=2\n",
        "blank-before-section": "[s]\na=1\n\n[t]\nb=2\n",
        "comment-not-blank": "[s]\na=1\n # skipped\n ; skipped\n two\n",
        "comment-and-blank": "[s]\na=1\n\n # skipped\n\n two\n",
        "blank-initial-value": "[s]\na=\n\n two\n",
        "inline-comments-literal": "[s]\na=one # literal\nb=two;literal\nc=three ; literal\n",
        "comment-key-literal": "[s]\na#b=1\na;b=2\n",
        "percent-literal": "[s]\na=%(missing)s\nb=100%\nc=%%\n",
        "quotes-literal": "[s]\na='value'\nb=\"value\"\n",
        "backslash-literal": "[s]\na=C:\\path\\name\nb=first\\nsecond\n",
        "colon-in-key": "[s]\ncli:tool=package:run\n",
        "first-delimiter": "[s]\na:b=c:d=e\n",
        "unicode-case": "[s]\n\u0130=one\n\u212a=two\n\u1e9e=three\n\u039f\u03a3=four\n\u039f\u03a3\u0391=five\n",
        "sigma-context": "[s]\n\u039f\u03a3'=a\n\u039f\u03a3'\u0391=b\n\u039f\u0301\u03a3\u0301=c\n\u03a3=d\n",
        "unicode-version": "[s]\n\u1c89=one\n\ua7cb=two\n\U00010d50=three\n",
        "unicode-duplicate": "[s]\n\u212a=one\nk=two\n",
        "unicode-whitespace": "\u2003[s]\u2003\n\u00a0a\u2009=\u3000value\u202f\n",
        "ascii-control-whitespace": "[s]\n\x1ca\x1d=\x1ev\x1f\n",
        "embedded-control-whitespace": "[s]\na\x1cb=one\na\x1db=two\na\x1eb=three\na\x1fb=four\n",
        "bom": "\ufeff[s]\na=1\n",
        "bom-in-value": "[s]\na=\ufeffvalue\n",
        "nul": "[s\x00]\na\x00=\x00value\n",
        "unicode-line-separators": "[s]\na=one\u0085two\u2028three\u2029four\n",
        "empty-after-error": "[s]\n=1\n continuation\n",
        "header-delimiters": "[a=b:c]\nkey=value\n",
        "default-empty": "[DEFAULT]\n[s]\n",
        "default-without-sections": "[DEFAULT]\na=1\n",
        "long-value": "[s]\na=" + "abcdefgh" * 2048 + "\n",
        "many-continuations": "[s]\na=first\n" + " continuation\n" * 128,
        "many-options": "[s]\n" + "".join(f"key{i}=value{i}\n" for i in range(128)),
        "many-sections": "".join(f"[s{i}]\na={i}\n" for i in range(128)),
    }
    yield from focused.items()

    workloads = FIXTURE.parent / "workloads"
    for entry in json.loads((workloads / "sources.json").read_text()):
        data = (workloads / entry["file"]).read_bytes()
        if hashlib.sha256(data).hexdigest() != entry["sha256"]:
            raise SystemExit(f"Captured workload differs: {entry['file']}")
        if not (workloads / entry["license_file"]).is_file():
            raise SystemExit(f"Missing workload license: {entry['file']}")
        yield "workload-" + entry["file"], data.decode("utf-8")

    for newline in ("\n", "\r\n", "\r"):
        for name in ("entry-points", "externally-managed", "setup-cfg", "blank-continuation", "invalid-before-duplicate-option"):
            yield f"newlines-{name}-{newline!r}", focused[name].replace("\n", newline)
    yield "mixed-newlines", "[s]\ra=1\r\nb=2\nc=3\r\n d\r"
    yield "carriage-return-header", "[s\r]\na=1\n"

    whitespace = ("", " ", "\t", "  ", "\u00a0", "\u2003", "\x1c")
    for index, (first, second, third) in enumerate(itertools.product(whitespace, repeat=3)):
        yield f"indent-{index}", f"[s]\n{first}a=1\n{second}b=2\n{third}c=3\n"
    for index, (left, right) in enumerate(itertools.product(whitespace, repeat=2)):
        yield f"trim-{index}", f"{left}[s]{right}\n{left}key{right}={left}value{right}\n"

    randomizer = random.Random(20261007)
    fragments = (
        "[s]", "[t]", "[DEFAULT]", "[ ]", "[]", "[s] trailing", "[s]x]",
        "a=one", "A=two", "a:three", "b=four", "b=", "a=", "a;b=three",
        "[incomplete=value", " invalid", "invalid", "=empty", ":empty",
        "  continuation", "\tcontinuation", "\u00a0a=value", "  [t]", " # comment",
        "", " ", ";comment", "#comment", "\x1ca=value", "\u212a=v", "k=v",
    )
    for index in range(600):
        newline = randomizer.choice(("\n", "\r\n", "\r"))
        lines = [randomizer.choice(fragments) for _ in range(randomizer.randrange(1, 14))]
        if randomizer.randrange(3):
            lines.insert(0, "[s]")
        yield f"mixed-{index}", newline.join(lines) + (newline if randomizer.randrange(2) else "")


def cases():
    for name, source in inputs():
        for case_sensitive, delimiters in itertools.product((False, True), ("=", "=:")):
            request = {
                "input": source,
                "case_sensitive": case_sensitive,
                "delimiters": delimiters,
            }
            yield {"name": name, **request, "expected": parse(request)}


def main():
    argument_parser = argparse.ArgumentParser(description=__doc__)
    mode = argument_parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--stdin", action="store_true", help="Answer newline-delimited JSON requests")
    mode.add_argument("--write", action="store_true", help="Regenerate the checked-in fixtures")
    mode.add_argument("--check", action="store_true", help="Check fixtures with this Python runtime")
    arguments = argument_parser.parse_args()
    if arguments.stdin:
        for line in sys.stdin:
            print(json.dumps(parse(json.loads(line)), ensure_ascii=True), flush=True)
    elif arguments.write:
        result = {
            "python": sys.version.split()[0],
            "unicode": unicodedata.unidata_version,
            "cases": list(cases()),
        }
        header = json.dumps({key: result[key] for key in ("python", "unicode")}, indent=2)
        entries = ",\n".join("    " + json.dumps(case, ensure_ascii=True) for case in result["cases"])
        FIXTURE.write_text(header.removesuffix("\n}") + ',\n  "cases": [\n' + entries + "\n  ]\n}\n")
        print(f"Wrote {len(result['cases'])} cases to {FIXTURE}")
    else:
        fixture = json.loads(FIXTURE.read_text())
        for stored, actual in itertools.zip_longest(fixture["cases"], cases()):
            if stored != actual:
                name = (stored or actual)["name"]
                raise SystemExit(f"Mismatch: {name}\nStored: {stored!r}\nGenerated: {actual!r}")
        print(f"Checked {len(fixture['cases'])} cases with CPython {sys.version.split()[0]}")


if __name__ == "__main__":
    main()
