# Contributing

Contributions are welcome. Follow the repository context files and run the
relevant verification gate before opening a pull request.

## Licensing contributions

OpenBIM.rs is dual-licensed: `AGPL-3.0-or-later OR LicenseRef-Commercial`
(see [`LICENSING.md`](LICENSING.md)). To keep the commercial option
possible, every contribution is accepted on the following terms, which you
agree to by submitting it (a pull request, patch, issue attachment or any
other material intended for inclusion):

1. **Licence to the project.** Your contribution is licensed under
   `AGPL-3.0-or-later`.
2. **Licence to the copyright holder.** You also grant the copyright holder
   named in [`NOTICE`](NOTICE) a perpetual, worldwide, non-exclusive,
   royalty-free, irrevocable licence to use, reproduce, modify, distribute,
   sublicense and relicense your contribution, including under proprietary
   or commercial terms, as part of OpenBIM.rs or works derived from it.
3. **You keep your copyright.** These terms grant a licence; they do not
   transfer ownership.
4. **You have the right to grant this.** The contribution is your own work,
   or you have permission to submit it on these terms (for example from
   your employer). Identify third-party material, keep its licence,
   attribution and provenance, and do not submit third-party material whose
   licence does not allow these terms.
5. **Patents.** You grant the same parties a licence under any patent claims
   you can license that your contribution necessarily infringes, on the terms
   of section 11 of the AGPL.

If you cannot agree to these terms, say so in the issue or pull request
before submitting; the contribution can then not be merged.

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
