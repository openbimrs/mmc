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

Core implementation, security hardening, and independent-review fixes are complete and landed publicly. Commit `2ef4984` was pushed to `main` on `https://github.com/openbimrs/mmc`, verified from a clean public clone, and hosted CI (gate + mutations) passed on that push. The exact pushed tree carries 37 tests, strict Clippy, rustdoc, package allowlisting/verification, and eight mutation probes. A builder-generated sample validates against both public MMC 2.0 XSDs without vendoring them. Documentation explicitly avoids claiming complete conformance with every DIN edition.

## Next action

Publication readiness reached: gate, mutation probes, package verification, public push, clean-clone verification, and hosted CI all pass on the exact pushed tree. Remaining before crates.io publish: final owner go-ahead.
