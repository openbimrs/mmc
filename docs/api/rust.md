# Rust API

The complete public Rust API is generated from source documentation:

- [Latest release on docs.rs](https://docs.rs/openbim-mmc)
- [Crate source on GitHub](https://github.com/openbimrs/mmc/tree/main/openbim-mmc/src)

## Main entry points

| API | Purpose |
| --- | --- |
| `MmcArchive::parse` | Parse an in-memory archive with default limits |
| `MmcArchive::parse_with_limits` | Parse with an explicit resource budget |
| `MmcArchive::validate` | Report structural and referential findings |
| `MmcArchive::extract_to` | Extract under the safe new-file policy |
| `MmcArchive::original_bytes` | Access the exact original archive bytes |
| `MmcArchive::to_deterministic_bytes` | Re-emit a reproducible archive |
| `MmcArchiveBuilder` | Construct a typed MMC 2.0 archive |

Until `openbim-mmc` is published, the docs.rs URL will not resolve. The source and this guide remain the authoritative pre-release documentation.
