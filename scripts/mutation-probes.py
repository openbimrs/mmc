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
    (
        "multimodel-truncation",
        "openbim-mmc/src/xml.rs",
        "if !stack.is_empty() {\n                    return Err(xml_message(\"MultiModel.xml\", \"truncated XML document\"));",
        "if false {\n                    return Err(xml_message(\"MultiModel.xml\", \"truncated XML document\"));",
        ["test", "--test", "conformance", "truncated_multimodel_is_rejected_at_every_open_depth", "--", "--exact"],
    ),
    (
        "link-model-truncation",
        "openbim-mmc/src/xml.rs",
        "if !stack.is_empty() {\n                    return Err(xml_message(path, \"truncated XML document\"));",
        "if false {\n                    return Err(xml_message(path, \"truncated XML document\"));",
        ["test", "--test", "conformance", "truncated_link_model_is_rejected_at_every_open_depth", "--", "--exact"],
    ),
    (
        "linked-model-entity-reference",
        "openbim-mmc/src/xml.rs",
        "value.push(character);",
        "drop(character);",
        ["test", "--test", "conformance", "linked_model_text_resolves_numeric_entity_references", "--", "--exact"],
    ),
    (
        "empty-link-models-container",
        "openbim-mmc/src/builder.rs",
        "if !link_models.is_empty() {",
        "if true {",
        ["test", "--test", "builder", "builder_omits_the_optional_link_models_collection_when_empty", "--", "--exact"],
    ),
    (
        "multimodel-content-after-root",
        "openbim-mmc/src/xml.rs",
        "if stack.is_empty() && !decoded.trim().is_empty() {\n                    return Err(xml_message(\n                        \"MultiModel.xml\",",
        "if false {\n                    return Err(xml_message(\n                        \"MultiModel.xml\",",
        ["test", "--test", "conformance", "rejects_content_outside_the_single_document_root", "--", "--exact"],
    ),
    (
        "link-model-content-after-root",
        "openbim-mmc/src/xml.rs",
        "if stack.is_empty() && !decoded.trim().is_empty() {\n                    return Err(xml_message(\n                        path,",
        "if false {\n                    return Err(xml_message(\n                        path,",
        ["test", "--test", "conformance", "rejects_content_outside_the_single_document_root", "--", "--exact"],
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
