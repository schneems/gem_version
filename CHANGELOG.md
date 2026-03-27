## Unreleased

- Make `serde` an optional dependency behind a `serde` feature flag (breaking change)
- Fix `license` field in Cargo.toml to `BSD-3-Clause` to match the LICENSE file (was incorrectly set to `MIT`)
- Remove `Default` implementation from `GemVersion`. There is no meaningful default version string, and providing one via `Default` could silently hide bugs.
- Fix `Display` implementation to preserve all version segments (e.g. `1.0.0` no longer drops trailing `.0` segments).
- Fixed silently dropping uppercase letters from versions. Previously `1.0.0.Preview1` would be stored internally as `1.0.0.review1` due to a regex mistake. Uppercase characters are now correctly represented in versions.
- Add `Eq` and `Ord` impls for `GemVersion`, enabling use in sorted collections and `.sort()`.
- Fix semver dash comparison: versions like `1.0.0-1` are now correctly treated as pre-releases (less than `1.0.0`). Display is not changed (e.g. `"1.0.0-alpha"` still displays as `"1.0.0-alpha"`, not `"1.0.0.pre.alpha"`).

## v0.3.1

- Relax `serde` version requirement

## v0.3.0

- `GemVersion` now implements `serde::Serialize` and `serde::Deserialize`

## v0.2.0

- `GemVersion` now satisfies clap's trait bounds for use in a command line argument. (https://github.com/schneems/gem_version/pull/3)

## v0.1.1

- `GemVersion` is now `Clone` (https://github.com/schneems/gem_version/pull/1)

## v0.1.0
