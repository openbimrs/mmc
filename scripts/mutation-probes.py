#!/usr/bin/env python3
"""Prove that critical security and conformance tests detect weakened guards."""
from pathlib import Path
import os
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
PROBES = [
    (
        "backslash-path",
        "openbim-mmc/src/path.rs",
        "} else if path.contains('\\\\') {",
        "} else if false {",
        ["test", "--test", "security", "rejects_unsafe_archive_names_and_ambiguous_normalization", "--", "--exact"],
    ),
    (
        "missing-payload",
        "openbim-mmc/src/validation.rs",
        "} else if archive.entry(path).is_none() {",
        "} else if false {",
        ["test", "--test", "archive", "semantic_validation_reports_missing_payloads_unknown_models_and_short_links", "--", "--exact"],
    ),
    (
        "link-cardinality",
        "openbim-mmc/src/validation.rs",
        "if link.relata.len() < 2 {",
        "if link.relata.is_empty() {",
        ["test", "--test", "archive", "semantic_validation_reports_missing_payloads_unknown_models_and_short_links", "--", "--exact"],
    ),
    (
        "link-model-distinctness",
        "openbim-mmc/src/validation.rs",
        "if distinct_models.len() < 2 {",
        "if false {",
        ["test", "--test", "schema", "reports_links_that_do_not_span_two_application_models", "--", "--exact"],
    ),
    (
        "namespace-exactness",
        "openbim-mmc/src/xml.rs",
        "Ok(namespace.as_ref() == expected.as_bytes())",
        "Ok(true)",
        ["test", "--test", "security", "rejects_wrong_xml_root_namespace_and_doctype", "--", "--exact"],
    ),
    (
        "actual-uncompressed-total",
        "openbim-mmc/src/archive.rs",
        ".min(total_remaining)",
        ".min(usize::MAX)",
        ["test", "--test", "limits", "rejects_forged_uncompressed_sizes_using_actual_output_budgets", "--", "--exact"],
    ),
    (
        "actual-compression-ratio",
        "openbim-mmc/src/archive.rs",
        "let ratio_total = ((bytes.len() as u128) * (limits.max_compression_ratio as u128))",
        "let ratio_total = ((bytes.len() as u128) * (limits.max_compression_ratio as u128) * 10_000_000)",
        ["test", "--test", "limits", "rejects_forged_uncompressed_sizes_using_actual_output_budgets", "--", "--exact"],
    ),
    (
        "zip-size-consistency",
        "openbim-mmc/src/archive.rs",
        "if payload.len() as u64 != declared {",
        "if false {",
        ["test", "--test", "limits", "rejects_forged_uncompressed_sizes_using_actual_output_budgets", "--", "--exact"],
    ),
]

with tempfile.TemporaryDirectory(prefix="openbim-mmc-mutations-") as temporary:
    base = Path(temporary)
    for name, relative, old, new, cargo_args in PROBES:
        checkout = base / name
        shutil.copytree(ROOT, checkout, ignore=shutil.ignore_patterns(".git", "target"))
        source = checkout / relative
        text = source.read_text()
        if text.count(old) != 1:
            raise SystemExit(f"{name}: mutation anchor count was {text.count(old)}, expected 1")
        source.write_text(text.replace(old, new))
        env = os.environ.copy()
        env["CARGO_TARGET_DIR"] = str(base / "target")
        result = subprocess.run(
            ["cargo", "+1.88.0", cargo_args[0], "--locked", *cargo_args[1:]],
            cwd=checkout,
            env=env,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.STDOUT,
        )
        if result.returncode == 0:
            raise SystemExit(f"{name}: mutation survived its regression test")
        print(f"caught mutation: {name}")
print(f"mutation gate verified: {len(PROBES)} of {len(PROBES)} caught")
