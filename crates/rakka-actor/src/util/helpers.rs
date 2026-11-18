// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/Helpers.scala (Apache-2.0).
//! `Helpers`: name generation, case folding and `requiring`. `ConfigOps` (duration reads,
//! `renderWithRedactions`) waits for AC-CFG.

use std::fmt::Display;

use crate::actor::IllegalArgumentError;

/// `Helpers.base64(l)`: `"$"` followed by the base-64 digits of `l`, least significant first.
pub fn base64(l: i64) -> String {
    let mut out = String::from("$");
    base64_append(l, &mut out);
    out
}

/// `Helpers.base64(l, sb)`: appends the digits of `l` to `sb` without the `"$"` prefix.
pub fn base64_append(l: i64, sb: &mut String) {
    // `l >>> 6` in Scala is an unsigned shift: negative numbers terminate too.
    let mut rest = l as u64;
    loop {
        sb.push(char::from(BASE64_CHARS[(rest & 63) as usize]));
        rest >>= 6;
        if rest == 0 {
            return;
        }
    }
}

/// `Helpers.toRootLowerCase(s)`: `s.toLowerCase(Locale.ROOT)`.
pub fn to_root_lower_case(s: &str) -> String {
    // Rust applies the full Unicode lower-case mapping with the context-dependent final sigma,
    // like Java, and has no locale-specific rules.
    s.to_lowercase()
}

/// `Helpers.base64chars`.
const BASE64_CHARS: &[u8; 64] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789+~";

/// `Helpers.Requiring`: `value.requiring(cond, msg)` returns `value` or fails with
/// `IllegalArgumentException("requirement failed: " + msg)`. Import it like Scala's
/// `import Helpers.Requiring`.
pub trait Requiring: Sized {
    /// `requiring(cond: Boolean, msg: => Any)`.
    fn requiring(self, cond: bool, msg: impl Display) -> Result<Self, IllegalArgumentError> {
        if cond {
            Ok(self)
        } else {
            Err(IllegalArgumentError::new(format!(
                "requirement failed: {msg}"
            )))
        }
    }

    /// `requiring(cond: A => Boolean, msg: => Any)`.
    fn requiring_with(
        self,
        cond: impl FnOnce(&Self) -> bool,
        msg: impl Display,
    ) -> Result<Self, IllegalArgumentError> {
        let holds = cond(&self);
        self.requiring(holds, msg)
    }
}

impl<A> Requiring for A {}
