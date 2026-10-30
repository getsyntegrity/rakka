// Derived from Apache Pekko 1.7.1 actor/src/main/scala/org/apache/pekko/NotUsed.scala (Apache-2.0).

use std::fmt;

/// Marks a generic position whose actual value is of no importance (the
/// materialized value of a stream that returns nothing, for example).
///
/// Pekko's `NotUsed.getInstance()` and `NotUsed.notUsed()` are Java facades for
/// the singleton: in Rust the value is `NotUsed` itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NotUsed;

impl fmt::Display for NotUsed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("NotUsed")
    }
}
