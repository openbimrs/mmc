#!/usr/bin/env python3
"""Fail closed when the crate package contains undeclared or restricted material."""
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
cmd = ["cargo", "+1.88.0", "package", "-p", "openbim-mmc", "--locked", "--no-verify", "--list"]
result = subprocess.run(cmd, cwd=ROOT, text=True, capture_output=True)
if result.returncode:
    print(result.stdout, end="", file=sys.stderr)
    print(result.stderr, end="", file=sys.stderr)
    raise SystemExit(result.returncode)
paths = [line.strip() for line in result.stdout.splitlines() if line.strip()]
allowed_files = {
    ".cargo_vcs_info.json", "Cargo.lock", "Cargo.toml", "Cargo.toml.orig",
    "README.md", "LICENSE", "CHANGELOG.md",
}
for path in paths:
    if path in allowed_files:
        continue
    if (path.startswith("src/") or path.startswith("tests/")) and path.endswith(".rs"):
        continue
    raise SystemExit(f"unexpected package member: {path}")
blocked = (".pdf", ".xsd", ".mmc", ".zip")
for path in paths:
    if path.lower().endswith(blocked) or "reference" in path.lower():
        raise SystemExit(f"restricted/reference artifact packaged: {path}")
for path in paths:
    source = ROOT / "openbim-mmc" / path
    if source.is_file() and source.suffix in {".rs", ".toml", ".md"}:
        if "[truncated]" in source.read_text(errors="replace"):
            raise SystemExit(f"truncation marker found: {path}")
required = {"Cargo.toml", "README.md", "LICENSE", "src/lib.rs"}
missing = required.difference(paths)
if missing:
    raise SystemExit(f"required package members absent: {sorted(missing)}")
print(f"package allowlist verified: {len(paths)} members")
