# MMC implementation plan

## Goal

Ship a public, production-safe pure-Rust `openbim-mmc` crate for DIN 18290-1 / MMC 2.0 archives without conflating MMC with ISO 21597 ICDD or DIN 18290 Parts 2–4 domain semantics.

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
- Mutate root namespace/path/linked-model checks and prove tests fail.

## Risk and rollback

- XML schemas are not vendored because the public upstream repository lacks a license. Implement against the normative model using original code and synthetic fixtures.
- External absolute URIs are represented but never fetched.
- Core is metadata/link aware but never parses GAEB or IFC payloads.

## Next action

Write failing integration tests for archive admission, typed views, links, extraction, and deterministic writing.
