#!/usr/bin/env python3
"""Fail closed if a generated documentation site contains private material."""
from pathlib import Path
import sys

if len(sys.argv) != 2:
    raise SystemExit("usage: check-docs-leakage.py <site-directory>")

root = Path(sys.argv[1])
if not root.is_dir():
    raise SystemExit(f"documentation output is missing: {root}")

forbidden_suffixes = {
    ".mmc", ".ifc", ".ifczip", ".gaeb", ".xsd", ".pdf", ".docx", ".xlsx"
}
forbidden_name_fragments = (".xsd.", ".pdf.", ".ifc.", ".mmc.", ".gaeb.")
forbidden_text = (
    b"/home/friedrich/",
    b"/mnt/backup/",
    b"/mnt/archive/",
    b"%PDF-",
    b"<xs:schema",
    b"<xsd:schema",
    b"http://www.w3.org/2001/XMLSchema",
)
forbidden_path_components = {"references", "schemas"}
violations: list[str] = []
files = [path for path in root.rglob("*") if path.is_file()]

for path in files:
    relative = path.relative_to(root)
    lower_name = path.name.lower()
    if forbidden_path_components.intersection(part.lower() for part in relative.parts):
        violations.append(f"forbidden published path component: {relative}")
    if path.suffix.lower() in forbidden_suffixes or any(
        fragment in lower_name for fragment in forbidden_name_fragments
    ):
        violations.append(f"forbidden published file type: {relative}")
    if path.stat().st_size > 5 * 1024 * 1024:
        violations.append(f"unexpected file larger than 5 MiB: {relative}")
    data = path.read_bytes()
    for marker in forbidden_text:
        if marker in data:
            violations.append(f"forbidden content marker in {relative}: {marker.decode()}")

if not files:
    violations.append("documentation output is empty")

if violations:
    print("documentation leakage check failed:", file=sys.stderr)
    for violation in violations:
        print(f"- {violation}", file=sys.stderr)
    raise SystemExit(1)

print(f"documentation leakage check passed: {len(files)} files")
