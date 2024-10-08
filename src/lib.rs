#![doc = include_str!("../README.md")]

use std::cmp;
use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

/// See module docs for a usage example
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct GemVersion {
    segments: Vec<VersionSegment>,
}

impl fmt::Display for GemVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let version_string = self
            .segments
            .iter()
            .map(|segment| match segment {
                VersionSegment::String(s) => s.clone(),
                VersionSegment::U32(i) => i.to_string(),
            })
            .collect::<Vec<String>>()
            .join(".");
        write!(f, "{version_string}")
    }
}

fn validation_regex() -> &'static fancy_regex::Regex {
    static VALIDATION_REGEX: OnceLock<fancy_regex::Regex> = OnceLock::new();
    VALIDATION_REGEX.get_or_init(|| {
        fancy_regex::Regex::new(
            "\\A\\s*([0-9]+(?>\\.[0-9a-zA-Z]+)*(-[0-9A-Za-z-]+(\\.[0-9A-Za-z-]+)*)?)?\\s*\\z",
        )
        .expect("Internal error: Bad Regex")
    })
}

fn segment_regex() -> &'static regex::Regex {
    static SEGMENT_REGEX: OnceLock<regex::Regex> = OnceLock::new();
    SEGMENT_REGEX.get_or_init(|| {
        regex::Regex::new("[0-9]+|[a-z]+").expect("Internal Error: Invalid Regular Expression!")
    })
}

impl TryFrom<String> for GemVersion {
    type Error = VersionError;

    fn try_from(version_string: String) -> Result<Self, Self::Error> {
        Self::from_str(&version_string)
    }
}

impl From<GemVersion> for String {
    fn from(version: GemVersion) -> String {
        version.to_string()
    }
}

impl FromStr for GemVersion {
    type Err = VersionError;

    fn from_str(version_string: &str) -> Result<Self, Self::Err> {
        if version_string.trim().is_empty() {
            Ok(GemVersion {
                segments: vec![VersionSegment::U32(0)],
            })
        } else if validation_regex().is_match(version_string).unwrap_or(false) {
            let (segments_l, segments_r) = segment_regex()
                .find_iter(version_string)
                .map(|regex_match| {
                    regex_match.as_str().parse::<u32>().ok().map_or_else(
                        || VersionSegment::String(regex_match.as_str().to_string()),
                        VersionSegment::U32,
                    )
                })
                .fold(
                    (vec![], vec![]),
                    |(mut acc_segments_l, mut acc_segments_r), item| {
                        match item {
                            item @ VersionSegment::U32(_) if acc_segments_r.is_empty() => {
                                acc_segments_l.push(item);
                            }
                            _ => acc_segments_r.push(item),
                        }

                        (acc_segments_l, acc_segments_r)
                    },
                );

            let is_zero_segment = |v: &VersionSegment| *v == VersionSegment::U32(0);
            let segments_l = drop_right_while(segments_l, is_zero_segment);
            let segments_r = drop_right_while(segments_r, is_zero_segment);

            let mut segments = segments_l;
            segments.extend(segments_r);

            Ok(GemVersion { segments })
        } else {
            Err(VersionError::InvalidVersion(String::from(version_string)))
        }
    }
}

impl PartialEq<GemVersion> for GemVersion {
    fn eq(&self, other: &Self) -> bool {
        self.partial_cmp(other) == Some(Ordering::Equal)
    }
}

impl PartialOrd<GemVersion> for GemVersion {
    fn partial_cmp(&self, other: &GemVersion) -> Option<Ordering> {
        let max = cmp::max(self.segments.len(), other.segments.len());

        let default = VersionSegment::U32(0);

        for index in 0..max {
            let segment_l = self.segments.get(index).unwrap_or(&default);
            let segment_r = other.segments.get(index).unwrap_or(&default);

            if segment_l == segment_r {
                continue;
            }

            return match (segment_l, segment_r) {
                (VersionSegment::U32(_), VersionSegment::String(_)) => Some(Ordering::Greater),
                (VersionSegment::U32(a), VersionSegment::U32(b)) => a.partial_cmp(b),
                (VersionSegment::String(_), VersionSegment::U32(_)) => Some(Ordering::Less),
                (VersionSegment::String(a), VersionSegment::String(b)) => {
                    // We have yet to verify that the sorting rules for strings are the same between
                    // Rust's and Ruby's standard library. Tests seem to pass, but here be dragons!
                    a.partial_cmp(b)
                }
            };
        }

        Some(Ordering::Equal)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum VersionError {
    InvalidVersion(String),
}

impl std::error::Error for VersionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl fmt::Display for VersionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VersionError::InvalidVersion(version) => {
                write!(f, "Invalid version string: {}", version)
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
enum VersionSegment {
    String(String),
    U32(u32),
}

fn drop_right_while<A, P: Fn(&A) -> bool>(i: Vec<A>, pred: P) -> Vec<A> {
    // There is probably a more efficient way to do this.
    let mut ret = i.into_iter().rev().skip_while(pred).collect::<Vec<A>>();
    ret.reverse();
    ret
}

#[cfg(test)]
mod test {
    use super::*;
    use clap::Parser;

    #[test]
    // https://github.com/rubygems/rubygems/blob/ecc8e895b69063562b9bf749b353948e051e4171/test/rubygems/test_gem_version.rb#L83-L89
    fn test_initialize() {
        for version in &["1.0", "1.0 ", " 1.0 ", "1.0\n", "\n1.0\n", "1.0"] {
            assert_eq!(v(version), v("1.0"));
        }
    }

    #[test]
    // https://github.com/rubygems/rubygems/blob/ecc8e895b69063562b9bf749b353948e051e4171/test/rubygems/test_gem_version.rb#L111-L115
    fn empty_version() {
        assert_eq!(v(""), v("0"));
        assert_eq!(v("   "), v("0"));
        assert_eq!(v(" "), v("0"));
    }

    #[test]
    // https://github.com/rubygems/rubygems/blob/ecc8e895b69063562b9bf749b353948e051e4171/test/rubygems/test_gem_version.rb#L140-L162
    fn spaceship() {
        assert_eq!(v("1.0"), v("1.0.0"));
        assert_eq!(v("1"), v("1.0.0"));

        assert!(v("1.0") > v("1.0.a"));
        assert!(v("1.8.2") > v("0.0.0"));
        assert!(v("1.8.2") > v("1.8.2.a"));
        assert!(v("1.8.2.b") > v("1.8.2.a"));
        assert!(v("1.8.2.a") < v("1.8.2"));
        assert!(v("1.8.2.a10") > v("1.8.2.a9"));
        assert_eq!(v(""), v("0"));

        assert_eq!(v("0.beta.1"), v("0.0.beta.1"));
        assert!(v("0.0.beta") < v("0.0.beta.1"));
        assert!(v("0.0.beta") < v("0.beta.1"));

        assert!(v("5.a") < v("5.0.0.rc2"));
        assert!(v("5.x") > v("5.0.0.rc2"));

        assert_eq!(v("1.9.3"), v("1.9.3"));
        assert!(v("1.9.3") > v("1.9.2.99"));
        assert!(v("1.9.3") < v("1.9.3.1"));
    }

    #[test]
    // https://github.com/rubygems/rubygems/blob/ecc8e895b69063562b9bf749b353948e051e4171/test/rubygems/test_gem_version.rb#L91-L109
    fn invalid_versions() {
        assert_eq!(
            "junk".parse::<GemVersion>(),
            Err(VersionError::InvalidVersion(String::from("junk")))
        );
        assert_eq!(
            "1.0\n2.0".parse::<GemVersion>(),
            Err(VersionError::InvalidVersion(String::from("1.0\n2.0")))
        );
        assert_eq!(
            "1..2".parse::<GemVersion>(),
            Err(VersionError::InvalidVersion(String::from("1..2")))
        );
        assert_eq!(
            "1.2\\ 3.4".parse::<GemVersion>(),
            Err(VersionError::InvalidVersion(String::from("1.2\\ 3.4")))
        );
        assert_eq!(
            "2.3422222.222.222222222.22222.ads0as.dasd0.ddd2222.2.qd3e.".parse::<GemVersion>(),
            Err(VersionError::InvalidVersion(String::from(
                "2.3422222.222.222222222.22222.ads0as.dasd0.ddd2222.2.qd3e."
            )))
        );
    }

    // Test helper method
    fn v(s: &str) -> GemVersion {
        s.parse().unwrap()
    }

    #[derive(Parser, Debug)]
    struct Args {
        #[arg(long)]
        version: GemVersion,
    }
}
