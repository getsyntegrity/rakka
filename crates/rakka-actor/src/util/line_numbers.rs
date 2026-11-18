// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/LineNumbers.scala (Apache-2.0).
//! `LineNumbers` (AC-UTIL-016.8): the source origin of a closure, shown in the `Debug` text of Typed behaviours.
//!
//! Pekko reads the JVM bytecode of the class that defines a closure and reports the source file and the line range of
//! its `LineNumberTable` (`SourceFileLines("Foo.scala", 29, 31)`, shown as `Foo.scala:29-31`). Rust has no bytecode, so
//! under Owner decision Q-UTIL-16 the origin is the place where the closure was handed over, captured by
//! `#[track_caller]` ([`closure_origin`]): the file name without directory and one line number (`from == to`).
//!
//! What is reproduced exactly: the text of every [`LineNumbersResult`] (`toString`: `file:line`, `file:from-to`, the
//! bare file name, `NoSourceInfo`, `UnknownSourceFormat(<explanation>)`) and [`pretty_name`]. What is not: the line
//! ranges and file names the Scala or Java compiler produced. A closure whose site was not captured has
//! [`LineNumbersResult::NoSourceInfo`], and [`pretty_name`] then falls back to its type name
//! (`std::any::type_name`), as Q-UTIL-16 (b) asks. `UnknownSourceFormat` and `SourceFile` exist for parity of the
//! result type; nothing in Rust produces them.

use std::fmt;
use std::panic::Location;

/// `LineNumbers.Result`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LineNumbersResult {
    /// `NoSourceInfo`: nothing is known about where the closure was defined.
    NoSourceInfo,
    /// `UnknownSourceFormat(explanation)`.
    UnknownSourceFormat(String),
    /// `SourceFile(filename)`.
    SourceFile(String),
    /// `SourceFileLines(filename, from, to)`.
    SourceFileLines {
        /// Source file name, without directory.
        filename: String,
        /// First line.
        from: i32,
        /// Last line.
        to: i32,
    },
}

impl fmt::Display for LineNumbersResult {
    /// The Scala `toString` of each case.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoSourceInfo => f.write_str("NoSourceInfo"),
            Self::UnknownSourceFormat(explanation) => {
                write!(f, "UnknownSourceFormat({explanation})")
            }
            Self::SourceFile(filename) => f.write_str(filename),
            Self::SourceFileLines { filename, from, to } if from != to => {
                write!(f, "{filename}:{from}-{to}")
            }
            Self::SourceFileLines { filename, from, .. } => write!(f, "{filename}:{from}"),
        }
    }
}

impl LineNumbersResult {
    /// The origin described by a caller location: its file name (no directory) and its line (`from == to`).
    pub fn from_location(location: &Location<'_>) -> Self {
        let path = location.file();
        let filename = path.rsplit(['/', '\\']).next().unwrap_or(path);
        let line = i32::try_from(location.line()).unwrap_or(i32::MAX);
        Self::SourceFileLines {
            filename: filename.to_owned(),
            from: line,
            to: line,
        }
    }
}

/// The origin of the closure being handed over at the call site: `LineNumbers(closure)` of Pekko, with the caller's
/// location in place of the bytecode line table. Call it from the constructor that receives the closure, which should
/// itself be `#[track_caller]` so that the user's call is reported.
#[track_caller]
pub fn closure_origin() -> LineNumbersResult {
    LineNumbersResult::from_location(Location::caller())
}

/// `LineNumbers.prettyName`: `<class>` without source info, `<class>(<explanation>)` for an unknown format,
/// `<class>(<file>)` for a file without lines and `<package>/<file>:<from>[-<to>]` otherwise. `class_name` is the
/// closure's type name (Pekko: `getClass.getName`) and `package` the name of its package (Rust: module path).
pub fn pretty_name(class_name: &str, package: &str, result: &LineNumbersResult) -> String {
    match result {
        LineNumbersResult::NoSourceInfo => class_name.to_owned(),
        LineNumbersResult::UnknownSourceFormat(message) => format!("{class_name}({message})"),
        LineNumbersResult::SourceFile(file) => format!("{class_name}({file})"),
        lines @ LineNumbersResult::SourceFileLines { .. } => format!("{package}/{lines}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn site() -> LineNumbersResult {
        closure_origin()
    }

    #[test]
    fn text_of_each_result_is_the_scala_to_string() {
        let lines = |from, to| LineNumbersResult::SourceFileLines {
            filename: "A.scala".to_owned(),
            from,
            to,
        };
        assert_eq!(lines(22, 22).to_string(), "A.scala:22");
        assert_eq!(lines(29, 31).to_string(), "A.scala:29-31");
        assert_eq!(lines(9, 5).to_string(), "A.scala:9-5");
        assert_eq!(LineNumbersResult::NoSourceInfo.to_string(), "NoSourceInfo");
        assert_eq!(
            LineNumbersResult::UnknownSourceFormat("parse error: x".to_owned()).to_string(),
            "UnknownSourceFormat(parse error: x)"
        );
        assert_eq!(
            LineNumbersResult::SourceFile("A.scala".to_owned()).to_string(),
            "A.scala"
        );
    }

    #[test]
    fn origin_is_this_file_and_the_call_line() {
        let origin = site();
        let LineNumbersResult::SourceFileLines { filename, from, to } = origin else {
            panic!("expected a location");
        };
        assert_eq!(filename, "line_numbers.rs");
        assert_eq!(from, to);
        assert!(from > 0);
        let expected = line!() + 1;
        let direct = closure_origin();
        assert_eq!(direct.to_string(), format!("line_numbers.rs:{expected}"));
    }

    #[test]
    fn pretty_name_of_each_result() {
        let lines = LineNumbersResult::SourceFileLines {
            filename: "F.rs".to_owned(),
            from: 3,
            to: 3,
        };
        assert_eq!(pretty_name("c", "m::n", &lines), "m::n/F.rs:3");
        assert_eq!(pretty_name("c", "m", &LineNumbersResult::NoSourceInfo), "c");
        assert_eq!(
            pretty_name(
                "c",
                "m",
                &LineNumbersResult::UnknownSourceFormat("x".to_owned())
            ),
            "c(x)"
        );
        assert_eq!(
            pretty_name("c", "m", &LineNumbersResult::SourceFile("F.rs".to_owned())),
            "c(F.rs)"
        );
    }
}
