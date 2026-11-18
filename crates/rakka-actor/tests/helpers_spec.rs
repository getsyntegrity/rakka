// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/Helpers.scala (Apache-2.0).
//! `util::helpers` (AC-UTIL-002), plan docs/superpowers/plans/2026-10-30-ac-util-core.md.

use rakka_actor::util::helpers::{Requiring, base64, base64_append, to_root_lower_case};

#[test]
fn base64_of_a_small_number_is_dollar_plus_one_digit() {
    // Helpers.base64chars (Helpers.scala:115) = a-z A-Z 0-9 + ~ ; anonymous actors are $a, $b, ...
    assert_eq!(base64(0), "$a");
    assert_eq!(base64(1), "$b");
    assert_eq!(base64(25), "$z");
    assert_eq!(base64(26), "$A");
    assert_eq!(base64(51), "$Z");
    assert_eq!(base64(52), "$0");
    assert_eq!(base64(61), "$9");
    assert_eq!(base64(62), "$+");
    assert_eq!(base64(63), "$~");
}

#[test]
fn base64_of_a_larger_number_writes_the_least_significant_digit_first() {
    // Helpers.scala:117-123: digit = l & 63, then l >>> 6 until it reaches 0.
    assert_eq!(base64(64), "$ab");
    assert_eq!(base64(65), "$bb");
    assert_eq!(base64(4095), "$~~");
    assert_eq!(base64(4096), "$aab");
    assert_eq!(base64(1 << 32), "$aaaaae");
}

#[test]
fn base64_append_extends_the_given_prefix_instead_of_the_dollar() {
    // The `sb` parameter (Helpers.scala:118): Children.scala:126 and ActorRefProvider.scala:440 pass their own prefix.
    let mut sb = String::from("x");
    base64_append(5, &mut sb);
    assert_eq!(sb, "xf");
    let mut sb = String::from("temp/$");
    base64_append(64, &mut sb);
    assert_eq!(sb, "temp/$ab");
    let mut empty = String::new();
    base64_append(0, &mut empty);
    assert_eq!(empty, "a");
}

#[test]
fn base64_treats_negative_numbers_as_unsigned_bit_patterns() {
    // Long.toInt & 63 then `>>> 6`: oracle cases B64-NEG-*. Characterization of the loop above:
    // it passed as soon as the loop existed, so it has no RED of its own.
    assert_eq!(base64(-1), "$~~~~~~~~~~p");
    assert_eq!(base64(-64), "$a~~~~~~~~~p");
    assert_eq!(base64(i64::MIN), "$aaaaaaaaaai");
    assert_eq!(base64(i64::MAX), "$~~~~~~~~~~h");
}

#[test]
fn to_root_lower_case_folds_like_java_with_the_root_locale() {
    // Helpers.scala:40: s.toLowerCase(Locale.ROOT); oracle cases LC-*. Characters are all older than
    // Unicode 10 (the Unicode of the JDK 11 that produced the golden).
    assert_eq!(to_root_lower_case("Hello WORLD 123"), "hello world 123");
    assert_eq!(to_root_lower_case("TITLE"), "title"); // no Turkish dotless i under Locale.ROOT
    assert_eq!(to_root_lower_case(""), "");
    assert_eq!(
        to_root_lower_case("\u{041F}\u{0420}\u{0418}\u{0412}\u{0415}\u{0422}"),
        "\u{043F}\u{0440}\u{0438}\u{0432}\u{0435}\u{0442}"
    );
    assert_eq!(to_root_lower_case("\u{FF21}\u{FF22}"), "\u{FF41}\u{FF42}");
    assert_eq!(to_root_lower_case("\u{10400}"), "\u{10428}"); // outside the BMP
    assert_eq!(to_root_lower_case("\u{00DF}"), "\u{00DF}"); // sharp s has no lower-case mapping
    // U+0130 maps to "i" + U+0307 (two UTF-16 units), not to a single char.
    assert_eq!(to_root_lower_case("\u{0130}"), "i\u{0307}");
}

#[test]
fn to_root_lower_case_picks_the_final_sigma_by_context() {
    assert_eq!(to_root_lower_case("\u{0391}\u{03A3}"), "\u{03B1}\u{03C2}"); // word-final: sigma
    assert_eq!(
        to_root_lower_case("\u{0391}\u{03A3}\u{0391}"),
        "\u{03B1}\u{03C3}\u{03B1}"
    );
    assert_eq!(to_root_lower_case("\u{03A3}"), "\u{03C3}"); // alone: not final
}

#[test]
fn requiring_returns_the_value_when_the_condition_holds() {
    // Helpers.scala:152-155. Non-Copy values come back moved, as `A` does in Scala.
    assert_eq!(5.requiring(true, "unused").unwrap(), 5);
    assert_eq!(String::from("a").requiring(true, "unused").unwrap(), "a");
}

#[test]
fn requiring_fails_with_the_requirement_failed_prefix() {
    let e = 5
        .requiring(false, "idle-cpu-level must be between 1 and 10")
        .unwrap_err();
    assert_eq!(e.java_class(), "IllegalArgumentException");
    assert_eq!(
        e.message(),
        "requirement failed: idle-cpu-level must be between 1 and 10"
    );
    // msg is `=> Any`: any printable value is allowed, the way String.valueOf renders it.
    assert_eq!(
        1.requiring(false, 42).unwrap_err().message(),
        "requirement failed: 42"
    );
    assert_eq!(
        1.requiring(false, "").unwrap_err().message(),
        "requirement failed: "
    );
}

#[test]
fn requiring_with_checks_the_predicate_on_the_value() {
    // Helpers.scala:164-167, used as getInt("max-channels").requiring(_ > 0, "...").
    assert_eq!(3.requiring_with(|n| *n > 0, "must be > 0").unwrap(), 3);
    let e = 0
        .requiring_with(|n| *n > 0, "nr-of-selectors must be > 0")
        .unwrap_err();
    assert_eq!(e.java_class(), "IllegalArgumentException");
    assert_eq!(
        e.message(),
        "requirement failed: nr-of-selectors must be > 0"
    );
    // The predicate sees the value that is returned on success.
    assert_eq!(
        String::from("abc")
            .requiring_with(|s| s.len() == 3, "len")
            .unwrap(),
        "abc"
    );
}
