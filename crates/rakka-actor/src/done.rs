// Derived from Apache Pekko 1.7.1 actor/src/main/scala/org/apache/pekko/Done.scala (Apache-2.0).

use std::fmt;

/// Signals completion without a value; clearer than `()`.
///
/// Pekko's `Done.getInstance()` and `Done.done()` are Java facades for the
/// singleton: in Rust the value is `Done` itself. Whether to use `()` instead
/// is Owner question Q-UTIL-5.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Done;

impl fmt::Display for Done {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Done")
    }
}
