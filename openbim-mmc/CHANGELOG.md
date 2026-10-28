# Changelog

All notable changes follow [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

## [0.1.1] - 2026-10-28

### Changed

- Licence: `AGPL-3.0-or-later OR LicenseRef-Commercial` (was
  `AGPL-3.0-or-later`); a commercial licence is available from the copyright
  holder named in `NOTICE`. See LICENSING.md. The crate ships `LICENSE`,
  `NOTICE` and `LICENSES/LicenseRef-Commercial.txt`.
- Raise MSRV to Rust 1.99.

## [0.1.0] - 2026-09-27

### Changed

- Relicensed repository-authored work from MIT to `AGPL-3.0-or-later`; historical releases remain under their published MIT terms, and third-party material retains its own terms.

### Added

- bounded, namespace-aware MMC 2.0 archive and XML reading;
- typed lossless projections for MultiModel and LinkModel documents;
- deterministic archive construction and writing;
- safe new-file extraction and path-collision protection;
- stable structural and referential-integrity reports;
- a VitePress user guide and GitHub Pages deployment with source/output leakage gates;
- explicit archive/XML preservation guarantees that distinguish exact no-op access from deterministic ZIP reconstruction.

[Unreleased]: https://github.com/openbimrs/mmc/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/openbimrs/mmc/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/openbimrs/mmc/releases/tag/v0.1.0
