# openbimrs/mmc

Pure-Rust implementation of DIN 18290-1 / Multi-Model Container 2.0 mechanics.

## Ground rules

- `openbim-mmc` owns `.mmc` ZIP, `MultiModel.xml`, and LinkModel XML mechanics.
- Application models are opaque bytes. GAEB, IFC, LV, cost, and invoicing semantics belong in optional domain/profile crates, not this core.
- Use `zip`, `quick-xml`, `iri-string`, and `uuid` directly; do not introduce first-party codec wrappers.
- Never vendor DIN/CEN/ISO documents or schema annexes. Tests use original synthetic fixtures.
- Archive names and XML-declared relative locations are untrusted. Preserve strict path safety, case-fold duplicate rejection, budgets, and symlink-safe extraction.
- Rust MSRV is `1.88.0`; run `scripts/gate.sh` before landing.
- Keep `CHANGELOG.md` in Keep-a-Changelog format and record architecture decisions under `docs/adr/`.
- The public documentation site is VitePress under `docs/`, built with `scripts/build-docs.sh`, and deployed to `/mmc/` by `.github/workflows/pages.yml`.
