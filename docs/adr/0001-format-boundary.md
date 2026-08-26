# ADR 0001: Keep MMC independent from ICDD and domain profiles

- **Status:** accepted
- **Date:** 2026-08-26

## Context

DIN 18290-1 defines the MMC 2.0 ZIP envelope, `MultiModel.xml`, and LinkModel XML. It states that ISO 21597 was considered and permits ICDD ontologies for RDF/OWL models, but normal MMC 2.0 and the GAEB-based DIN 18290 Parts 2–4 are not ICDD archives.

## Decision

`openbim-mmc` directly uses `zip` and `quick-xml` and owns only MMC mechanics. It has no dependency on ICDD, GAEB, or IFC.

DIN 18290 LV, cost, and invoice profiles will depend on this crate and the canonical `openbim-gaeb` crate. Full model validation may additionally depend on canonical IFC. An MMC↔ICDD conversion layer must be a separate adapter depending on both formats.

No normative DIN PDF or schema is redistributed in crate packages. The public buildingSMART-community/MMC schema repository was inspected as prior art at commit `5b440d3a1c0dd33043589ecb6017e8a790385f7c`; it has no repository license and no source was copied or vendored.

## Consequences

- Core users do not pay for domain dependencies.
- Cargo feature unification cannot make GAEB/IFC mandatory.
- Each format retains its own archive and vocabulary invariants.
- Domain support remains additive without a dependency cycle.

## Verification

`scripts/gate.sh` checks the no-default-feature build and package contents. `scripts/mutation-probes.sh` demonstrates that security and conformance tests fail when critical guards are weakened.
