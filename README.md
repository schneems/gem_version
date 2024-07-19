
# Gem version

## What

Implement Ruby's `Gem::Version` comparison logic in Rust:

- [Gem::Version](https://github.com/rubygems/rubygems/blob/ecc8e895b69063562b9bf749b353948e051e4171/lib/rubygems/version.rb)
- [Gem::Version tests](https://github.com/rubygems/rubygems/blob/ecc8e895b69063562b9bf749b353948e051e4171/test/rubygems/test_gem_version.rb)

The main use case is for the Heroku Ruby buildpack <https://github.com/heroku/buildpacks-ruby> and associated ecosystem of managing Ruby logic inside of Rust.

## Install

Add it to your cargo.toml:

```shell
$ cargo add gem_version
```

## Use

```rust
use std::str::FromStr;
use gem_version::GemVersion;

let version = GemVersion::from_str("1.0.0").unwrap();
assert!(version < GemVersion::from_str("2.0.0").unwrap());
```
