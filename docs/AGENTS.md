# Documentation map

- `.vitepress/config.ts` owns the `/mmc/` site URL, navigation, and metadata.
- `.vitepress/theme/` contains small token-based visual overrides.
- `index.md` is the public landing page; `guide/`, `architecture/`, `api/`, and `project/` are the maintained user guide.
- `capabilities.md` is the source of truth for implemented guarantees and explicit non-goals.
- `security.md` documents untrusted-input and extraction boundaries.
- `adr/0001-format-boundary.md` records why MMC is independent from ICDD and DIN domain profiles.
- `ROADMAP.md` remains the repository-level engineering roadmap; keep it aligned with `project/roadmap.md` when scope changes.
- Never add normative standards, schemas, or corpus files. `scripts/check-docs-leakage.py` scans source and generated output.

Build from the repository root with `scripts/build-docs.sh`.
