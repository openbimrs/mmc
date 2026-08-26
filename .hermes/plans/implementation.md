# MMC implementation plan

## Goal

Ship a public, production-safe pure-Rust `openbim-mmc` crate for MMC 2.0 archives used in DIN 18290 workflows without conflating MMC with ISO 21597 ICDD or DIN 18290 Parts 2–4 domain semantics.

## Constraints

- Rust 1.88.0.
- Direct maintained ZIP/XML/IRI/UUID dependencies.
- No restricted standards or unlicensed upstream schemas in repository/package.
- No `mmc` alias: crates.io already contains unrelated `ferrosoft/mmc` 0.1.0.
- Synthetic fixtures only.

## Workstreams

1. Typed lossless projections of `MultiModel.xml` and LinkModel 2.0 XML.
2. Strict bounded ZIP admission and exact-root discovery.
3. Cross-document referential validation.
4. Safe extraction with new-file semantics and symlink rejection.
5. Deterministic archive builder/writer.
6. Package, mutation, feature-isolation, CI, docs, and review gates.

## Validation

- Test-first happy path and adversarial corpus.
- `fmt`, `build`, `test`, strict Clippy, docs, isolated crate build.
- Package content allowlist and external consumer smoke test.
- Mutate root namespace/path/linked-model and measured-decompression checks and prove tests fail.

## Risk and rollback

- XML schemas are not vendored because the public upstream repository lacks a license. Implement against the normative model using original code and synthetic fixtures.
- External absolute URIs are represented but never fetched.
- Core is metadata/link aware but never parses GAEB or IFC payloads.

## Milestone status

Core implementation, security hardening, and independent-review fixes are complete and landed publicly. A delayed exact-tree review of `c280ec7` exposed three additional release blockers; all were reproduced before correction: truncated XML acceptance, schema-invalid empty `LinkModels`, and dropped XML entity references in `LinkedModel` text. The current candidate fixes those defects with 41 tests and 12 mutation probes. Documentation explicitly avoids claiming complete conformance with every DIN edition.

## Next action

Commit the follow-up corrections, run the complete exact-tree Rust 1.88/package/mutation gate, obtain an independent review of that exact commit, then push and repeat clean-public-clone/package-consumer/XSD verification before declaring crates.io readiness.
