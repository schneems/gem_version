
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

## Diverging behavior

Ruby's Gem::Version reference implementation converts `-` to `.pre.` [commit introduced](https://github.com/ruby/rubygems/commit/df88d165d89cc7ece9b746589e58d93b76a33628). This means the value you put in is not the value you get out:

```ruby
puts Gem::Version.new("1.0.0-alpha1")
# => "1.0.0.pre.alpha1"
```

It seems the coupling between comparison logic and representation was accidental at the time of introduction. This library diverges by showing the original string (and decoupling that from the comparison representation):

```rust
use std::str::FromStr;
use gem_version::GemVersion;

let version = GemVersion::from_str("1.0.0-alpha1").unwrap();

// Version does not have `.pre.` in it:
assert_eq!("1.0.0-alpha1", &version.to_string());

// But it performs comparisons as if it did:
assert_eq!(
    GemVersion::from_str("1.0.0.pre.alpha1").unwrap(),
    version
);
```

If you want the upstream (reference) behavior that also changes the display output, you can make a newtype that uses `replace("-", ".pre.")` on the input `String`.
