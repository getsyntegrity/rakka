// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/PekkoVersion.scala and project/VersionGenerator.scala (Apache-2.0).
//! `PekkoVersion.require` and `UnsupportedPekkoVersion` (AC-UTIL-005.1).

use std::fmt;

use crate::actor::NumberFormatError;
use crate::actor::jstring::parse_int;

/// `Version.current`: the version of Pekko this crate ports (Q-CFG-5: "2.0.0").
pub const CURRENT: &str = "2.0.0";

/// `UnsupportedPekkoVersion`: the Pekko in use is older or incompatible with what a library requires.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnsupportedPekkoVersion {
    message: String,
}

impl UnsupportedPekkoVersion {
    /// Simple name of the Java class.
    pub fn java_class(&self) -> &'static str {
        "UnsupportedPekkoVersion"
    }

    /// `getMessage`.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for UnsupportedPekkoVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for UnsupportedPekkoVersion {}

/// What `PekkoVersion.require` can throw.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RequireError {
    Unsupported(UnsupportedPekkoVersion),
    NumberFormat(NumberFormatError),
}

impl fmt::Display for RequireError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported(e) => e.fmt(f),
            Self::NumberFormat(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for RequireError {}

/// `PekkoVersion.require(libraryName, requiredVersion)` against [`CURRENT`].
pub fn require(library: &str, required: &str) -> Result<(), RequireError> {
    require_against(library, required, CURRENT)
}

/// The pre-release marker of a version: `-M<n>` or `-RC<n>`.
#[derive(Clone, Copy, PartialEq, Eq)]
enum PreRelease {
    Milestone,
    ReleaseCandidate,
}

/// A string matching `(\d+)\.(\d+)\.(\d+)(?:-(M|RC)(\d+))?` in full, with ASCII digits only.
struct Parsed<'a> {
    major: &'a str,
    minor: &'a str,
    patch: &'a str,
    pre_release: Option<(PreRelease, &'a str)>,
}

fn digits_prefix(text: &str) -> Option<(&str, &str)> {
    let end = text.bytes().take_while(u8::is_ascii_digit).count();
    (end > 0).then(|| text.split_at(end))
}

fn parse(version: &str) -> Option<Parsed<'_>> {
    let (major, rest) = digits_prefix(version)?;
    let (minor, rest) = digits_prefix(rest.strip_prefix('.')?)?;
    let (patch, rest) = digits_prefix(rest.strip_prefix('.')?)?;
    if rest.is_empty() {
        return Some(Parsed {
            major,
            minor,
            patch,
            pre_release: None,
        });
    }
    let rest = rest.strip_prefix('-')?;
    let (kind, rest) = if let Some(rest) = rest.strip_prefix("RC") {
        (PreRelease::ReleaseCandidate, rest)
    } else {
        (PreRelease::Milestone, rest.strip_prefix('M')?)
    };
    let (number, rest) = digits_prefix(rest)?;
    rest.is_empty().then_some(Parsed {
        major,
        minor,
        patch,
        pre_release: Some((kind, number)),
    })
}

/// `String.toInt`: ASCII digits only reach here, so the only failure is overflow.
fn to_int(digits: &str) -> Result<i32, RequireError> {
    parse_int(digits).map_err(RequireError::NumberFormat)
}

/// `PekkoVersion.require(libraryName, requiredVersion, currentVersion)`.
///
/// Versions that do not match the numeric pattern (snapshots, timestamps, anything else) always pass. Numbers
/// are first compared as text and, only when the text differs, as `Int`s, exactly like Pekko: `"2.5.6"`
/// against `"2.05.5"` passes because the minor numbers are equal as ints and the patch is never looked at.
pub fn require_against(library: &str, required: &str, current: &str) -> Result<(), RequireError> {
    if required == current {
        return Ok(());
    }
    let (Some(current_parsed), Some(required_parsed)) = (parse(current), parse(required)) else {
        return Ok(());
    };
    let unsupported = || {
        Err(RequireError::Unsupported(UnsupportedPekkoVersion {
            message: format!(
                "Current version of Pekko is [{current}], but {library} requires version [{required}]"
            ),
        }))
    };
    if required_parsed.major != current_parsed.major {
        return unsupported();
    }
    if required_parsed.minor != current_parsed.minor {
        return if to_int(required_parsed.minor)? > to_int(current_parsed.minor)? {
            unsupported()
        } else {
            Ok(())
        };
    }
    if required_parsed.patch != current_parsed.patch {
        return if to_int(required_parsed.patch)? > to_int(current_parsed.patch)? {
            unsupported()
        } else {
            Ok(())
        };
    }
    match (required_parsed.pre_release, current_parsed.pre_release) {
        (None, Some(_)) => unsupported(),
        (Some((PreRelease::Milestone, required_number)), Some((PreRelease::Milestone, number)))
        | (
            Some((PreRelease::ReleaseCandidate, required_number)),
            Some((PreRelease::ReleaseCandidate, number)),
        ) => {
            if to_int(required_number)? > to_int(number)? {
                unsupported()
            } else {
                Ok(())
            }
        }
        (Some((PreRelease::ReleaseCandidate, _)), Some((PreRelease::Milestone, _))) => {
            unsupported()
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn passes(required: &str, current: &str) -> bool {
        require_against("PekkoVersionSpec", required, current).is_ok()
    }

    #[test]
    fn spec_versions_that_are_satisfied() {
        for (required, current) in [
            ("2.5.6", "2.5.6"),
            ("2.5.6", "2.5.7"),
            ("2.5.6", "2.6.0"),
            ("2.5.6", "2.5.7-RC10"),
            ("2.6.0-RC1", "2.6.0-RC1"),
            ("2.6.0-RC1", "2.6.0-RC2"),
            ("2.6.0-RC1", "2.7.0"),
            ("2.5.6", "2.5.7-M10"),
            ("2.5.7-M9", "2.5.7-M10"),
            ("2.5.7-M9", "2.5.8"),
            ("2.5.7-M9", "2.5.7-RC1"),
            ("2.5.6", "2.5-SNAPSHOT"),
            ("2.5.6", "2.5-20180109-133700"),
            ("2.5-SNAPSHOT", "2.5-SNAPSHOT"),
            ("nonsense", "2.5.6"),
            ("2.5.6", "nonsense"),
        ] {
            assert!(passes(required, current), "{required} / {current}");
        }
    }

    #[test]
    fn spec_versions_that_are_refused_name_the_library() {
        for (required, current) in [
            ("2.5.6", "2.5.6-RC1"),
            ("2.5.6-RC2", "2.5.6-RC1"),
            ("2.5.6", "2.5.6-M1"),
            ("2.5.7-M10", "2.5.7-M9"),
            ("2.5.7-RC1", "2.5.7-M9"),
            ("2.5.6", "3.0.0"),
            ("2.5.6", "1.0.0"),
            ("2.5.6", "2.4.19"),
            ("2.5.6", "2.5.5"),
        ] {
            let error = require_against("my-lib", required, current).unwrap_err();
            let RequireError::Unsupported(error) = error else {
                panic!("expected UnsupportedPekkoVersion for {required} / {current}");
            };
            assert_eq!(error.java_class(), "UnsupportedPekkoVersion");
            assert_eq!(
                error.to_string(),
                format!(
                    "Current version of Pekko is [{current}], but my-lib requires version [{required}]"
                )
            );
        }
    }

    #[test]
    fn numbers_beyond_int_fail_like_to_int_and_equal_text_short_circuits() {
        let error = require_against("l", "2.5.99999999999", "2.5.6").unwrap_err();
        assert_eq!(
            error.to_string(),
            "For input string: \"99999999999\"".to_owned()
        );
        assert!(matches!(error, RequireError::NumberFormat(_)));
        assert!(passes("2.5.99999999999", "2.5.99999999999"));
        assert!(
            passes("2.5.6", "2.05.5"),
            "equal minors as ints end the comparison"
        );
    }

    #[test]
    fn public_require_uses_the_current_version() {
        assert_eq!(CURRENT, "2.0.0");
        assert!(require("l", "2.0.0").is_ok());
        let error = require("l", "1.7.1").unwrap_err();
        assert_eq!(
            error.to_string(),
            "Current version of Pekko is [2.0.0], but l requires version [1.7.1]"
        );
        assert!(std::error::Error::source(&error).is_none());
    }
}
