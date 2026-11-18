// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/Helpers.scala (Apache-2.0).
//! `Helpers`: name generation, case folding and `requiring`. `ConfigOps` (duration reads,
//! `renderWithRedactions`) waits for AC-CFG.

use std::cmp::Ordering;
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

/// `Helpers.isWindows` (AC-UTIL-026): the platform the program is built for (Q-UTIL-13: Pekko tests
/// `os.name` for the substring `win`; no JVM that reports it on another system is known).
pub const IS_WINDOWS: bool = cfg!(windows);

/// `Helpers.compareIdentityHash(a, b)` (AC-UTIL-024): a total order over objects by identity. Rust has no
/// `identityHashCode`; the identity is the address of the referent (Q-UTIL-10), compared as an
/// unsigned number, so unlike a hash it never ties for two distinct live objects.
#[must_use]
pub fn compare_identity_hash<T: ?Sized>(a: &T, b: &T) -> Ordering {
    let address = |reference: &T| std::ptr::from_ref(reference).cast::<()>() as usize;
    address(a).cmp(&address(b))
}

/// `Helpers.identityHashComparator(comp)`: orders by identity, and falls back to `comp` only when the
/// identities tie for two objects that are not equal (the `comp` must be consistent with equality).
pub fn identity_hash_comparator<T: PartialEq + ?Sized>(
    comp: impl Fn(&T, &T) -> Ordering,
) -> impl Fn(&T, &T) -> Ordering {
    move |a, b| match compare_identity_hash(a, b) {
        Ordering::Equal if a != b => comp(a, b),
        by_identity => by_identity,
    }
}

/// `Helpers.currentTimeMillisToUTCString(timestamp)` (AC-UTIL-025): `HH:mm:ss.SSSUTC` of the time of
/// day. Pekko pads each field with zeros by comparing against 10/100, so a negative timestamp keeps
/// the minus sign of every field (`-1` gives `00:00:00.00-1UTC`); that quirk is reproduced.
#[must_use]
pub fn current_time_millis_to_utc_string(timestamp: i64) -> String {
    let time_of_day = timestamp % 86_400_000;
    let hours = time_of_day / 3_600_000;
    let minutes = time_of_day / 60_000 % 60;
    let seconds = time_of_day / 1000 % 60;
    let ms = time_of_day % 1000;
    let zero_if = |condition: bool| if condition { "0" } else { "" };
    format!(
        "{}{hours}:{}{minutes}:{}{seconds}.{}{}{ms}UTC",
        zero_if(hours < 10),
        zero_if(minutes < 10),
        zero_if(seconds < 10),
        zero_if(ms < 100),
        zero_if(ms < 10),
    )
}

/// `Helpers.timestamp(time)` (AC-UTIL-025) for a zone that is a fixed offset from UTC: the pattern
/// `MM/dd/yyyy HH:mm:ss.SSS` of `time` (milliseconds since the epoch). Years print as Java's `yyyy`
/// (year of era, at least four digits, `+` prefix beyond 9999). The system default zone Pekko uses
/// needs a time zone database: pending Q-UTIL-18.
#[must_use]
pub fn timestamp_at_utc_offset(time: i64, utc_offset_seconds: i32) -> String {
    let local_seconds = time.div_euclid(1000) + i64::from(utc_offset_seconds);
    let millis = time.rem_euclid(1000);
    let (year, month, day) = civil_from_epoch_day(local_seconds.div_euclid(86_400));
    let second_of_day = local_seconds.rem_euclid(86_400);
    let year_of_era = if year > 0 { year } else { 1 - year };
    let year_text = if year_of_era > 9999 {
        format!("+{year_of_era}")
    } else {
        format!("{year_of_era:04}")
    };
    format!(
        "{month:02}/{day:02}/{year_text} {:02}:{:02}:{:02}.{millis:03}",
        second_of_day / 3600,
        second_of_day / 60 % 60,
        second_of_day % 60
    )
}

/// Proleptic Gregorian (year, month, day) of a day count since 1970-01-01.
fn civil_from_epoch_day(epoch_day: i64) -> (i64, i64, i64) {
    let shifted = epoch_day + 719_468; // days since 0000-03-01
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_from_march = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_from_march + 2) / 5 + 1;
    let month = if month_from_march < 10 {
        month_from_march + 3
    } else {
        month_from_march - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    (year, month, day)
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
