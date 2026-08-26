# Architecture

## Dependency direction

```text
application / DIN profile
    ├── openbim-mmc
    ├── openbim-gaeb      (when GAEB semantics are needed)
    └── canonical IFC     (when IFC semantics are needed)
```

`openbim-mmc` depends directly on maintained ZIP, XML, IRI, Unicode, and UUID crates. It does not depend on the OpenBIM facade, ICDD, IFC, or GAEB.

## Owned boundary

The crate owns:

- MMC archive admission and deterministic emission;
- `MultiModel.xml` discovery and typed projection;
- embedded LinkModel discovery and typed projection;
- model/resource/link identity and referential checks;
- opaque payload retention and safe extraction.

It does not own:

- IFC or GAEB parsing;
- DIN 18290 Parts 2–4 LV, cost, or invoicing semantics;
- ISO 21597 ICDD containers;
- MMC↔ICDD conversion policy.

Adapters that combine formats depend on both canonical cores. Neither container format depends on the other merely because it can carry the other format’s data.

## Parsed and source views

A parsed archive exposes typed structures for inspection and validation while retaining source bytes. This separation lets callers inspect imperfect but admissible archives without pretending the typed projection captures every possible lexical detail.

See [ADR 0001 on GitHub](https://github.com/openbimrs/mmc/blob/main/docs/adr/0001-format-boundary.md) for the accepted boundary decision.
