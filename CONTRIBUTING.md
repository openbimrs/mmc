# Contributing

Contributions are welcome. Follow the repository context files and run the
relevant verification gate before opening a pull request.

## Licensing contributions

Unless an explicitly signed agreement says otherwise, every contribution
submitted to this repository is licensed under `AGPL-3.0-or-later`. Submit only
work that you have the right to license. Identify third-party material and
preserve its license, attribution, and provenance.

## Releasing

Releases publish from CI through crates.io trusted publishing
(`.github/workflows/release.yml`); no one needs a crates.io token.

1. Bump `version` in `openbim-mmc/Cargo.toml`, run
   `cargo update -p openbim-mmc`, and date the `## [Unreleased]` changelog
   section as `## [x.y.z] - YYYY-MM-DD` (with its compare link). Copy
   `CHANGELOG.md` to `openbim-mmc/CHANGELOG.md`; the gate rejects a drifted copy.
2. Run `scripts/gate.sh` and `python3 scripts/mutation-probes.py`, and
   `cargo semver-checks` against the last release to confirm the bump matches
   the change.
3. Merge to `main`, then push an annotated tag `vx.y.z` on that commit.

The workflow refuses a tag that is not on `main`, does not match the manifest
version, or has no changelog section; it gates the tagged commit, publishes to
crates.io from the `crates.io` environment, and creates the GitHub release from
the changelog. Re-running a partly failed release skips what is already live.
To rehearse, run the workflow by hand with an existing tag: it gates and
packages, and publishes nothing.

### First publication of a new crate

Trusted publishing cannot create a crate. The first version of a new crate is
therefore published by hand by a maintainer with a personal crates.io token
(`cargo publish -p <crate> --locked` from the tagged commit on `main`). Then:

1. On crates.io, open the crate's Settings -> Trusted Publishing and add a
   GitHub publisher: repository `openbimrs/mmc`, workflow `release.yml`,
   environment `crates.io`.
2. Push the release tag. The workflow finds the version already live, skips
   publishing, and only creates the GitHub release.

Every later version is published by the workflow. crates.io trusts this
repository, the file name `release.yml` and the `crates.io` environment;
renaming either needs the same change there.
