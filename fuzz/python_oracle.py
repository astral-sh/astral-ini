"""Run the persistent Python oracle with its own memory ceiling."""

from pathlib import Path
import resource
import runpy
import sys
import unicodedata


resource.setrlimit(resource.RLIMIT_AS, (512 * 1024 * 1024,) * 2)
if sys.version_info[:2] != (3, 12) or unicodedata.unidata_version != "15.0.0":
    raise RuntimeError("The differential oracle requires CPython 3.12 and Unicode 15.0.0")
script = Path(__file__).resolve().parents[1] / "scripts/generate_conformance.py"
sys.argv = [str(script), "--stdin"]
runpy.run_path(str(script), run_name="__main__")
