# Capability contract

## Implemented

| Capability | Status |
| --- | --- |
| Bounded `.mmc` ZIP parsing | <span class="status-implemented">Implemented</span> |
| Exact root `MultiModel.xml` discovery | <span class="status-implemented">Implemented</span> |
| Namespace-aware MultiModel and LinkModel parsing | <span class="status-implemented">Implemented</span> |
| Structural and referential reporting | <span class="status-implemented">Implemented</span> |
| Safe controlled extraction | <span class="status-implemented">Implemented</span> |
| Deterministic archive construction | <span class="status-implemented">Implemented</span> |
| Public MMC 2.0 schema-shaped writer | <span class="status-implemented">Implemented</span> |
| IFC, GAEB, LV, cost, or invoice semantics | <span class="status-boundary">Higher layer</span> |
| Complete current DIN 18290 conformance certification | <span class="status-boundary">Not claimed</span> |

## Preservation and rewriting

“Byte-identical rewrite” conflates three different contracts. `openbim-mmc` states them separately:

1. **Original input:** `MmcArchive::original_bytes()` returns the exact archive bytes supplied by the caller.
2. **Entry content:** original MultiModel XML, embedded LinkModel XML, and opaque payload entry bytes remain available exactly. Deterministic re-emission preserves those entry contents.
3. **ZIP envelope:** `to_deterministic_bytes()` deliberately reconstructs ordering, timestamps, headers, and compression. Its output is reproducible, not byte-identical to an arbitrary source ZIP envelope.

The crate does not offer typed in-place XML editing, so it does not claim that unknown XML syntax can be mixed with typed mutations without lexical change. Callers needing a no-op byte copy should use `original_bytes()`; callers needing a deterministic archive should use `to_deterministic_bytes()`.

## Standards scope

The typed projection and writer target the public buildingSMART MMC `2.0.0` schema shape. Public prior art does not establish redistribution rights for the schemas and does not prove complete conformance with current licensed DIN material. Schemas and normative documents are therefore not packaged or published with this site.
