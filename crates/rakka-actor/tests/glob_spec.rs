// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/Glob.scala and actor-tests GlobSpec.scala (Apache-2.0).
//! `Glob` (AC-UTIL-016.6): `?` is one UTF-16 code unit, `*` any run (also empty), the rest literal.
//! Port of GlobSpec; the regex side of its exhaustive case is replaced by a naive recursive
//! definition of the glob (Q-UTIL-8: `Helpers.makePattern` is not reproduced).

use std::time::{Duration, Instant};

use rakka_actor::util::glob_matches;

#[test]
fn literals_match_only_themselves() {
    assert!(glob_matches("abc", "abc"));
    assert!(!glob_matches("abc", "abd"));
    assert!(glob_matches("", ""));
    assert!(!glob_matches("", "a"));
    assert!(!glob_matches("a", ""));
}

#[test]
fn question_mark_is_exactly_one_character() {
    assert!(glob_matches("a?c", "abc"));
    assert!(!glob_matches("a?c", "ac"));
    assert!(!glob_matches("a?c", "abbc"));
    assert!(glob_matches("???", "abc"));
}

#[test]
fn star_is_any_run_of_characters() {
    assert!(glob_matches("*", ""));
    assert!(glob_matches("*", "anything"));
    assert!(glob_matches("a*", "a"));
    assert!(glob_matches("a*c", "abbbc"));
    assert!(!glob_matches("a*c", "abbbd"));
    assert!(glob_matches("**", "ab"));
    assert!(glob_matches("*b*", "abc"));
}

#[test]
fn question_mark_consumes_one_utf16_unit_not_one_code_point() {
    let astral = "\u{1F600}";
    assert!(!glob_matches("?", astral));
    assert!(glob_matches("??", astral));
    assert!(glob_matches("a??b", "a\u{1F600}b"));
    assert!(glob_matches("?\u{1F600}", "x\u{1F600}"));
    assert!(glob_matches("*", astral));
}

#[test]
fn line_breaks_are_ordinary_characters() {
    assert!(glob_matches("a?b", "a\nb"));
    assert!(glob_matches("a*b", "a\r\nb"));
    assert!(glob_matches("?", "\n"));
}

/// Definition of the glob by plain recursion (exponential, fine for tiny inputs).
fn reference(pattern: &[char], input: &[char]) -> bool {
    match pattern.split_first() {
        None => input.is_empty(),
        Some(('*', rest)) => (0..=input.len()).any(|taken| reference(rest, &input[taken..])),
        Some((&head, rest)) => match input.split_first() {
            Some((&first, tail)) => (head == '?' || head == first) && reference(rest, tail),
            None => false,
        },
    }
}

fn all_strings(alphabet: &[char], max_len: usize) -> Vec<String> {
    let mut all = vec![String::new()];
    let mut layer = vec![String::new()];
    for _ in 0..max_len {
        layer = layer
            .iter()
            .flat_map(|prefix| alphabet.iter().map(move |c| format!("{prefix}{c}")))
            .collect();
        all.extend(layer.iter().cloned());
    }
    all
}

#[test]
fn agrees_with_the_plain_definition_over_every_short_pattern_and_input() {
    let patterns = all_strings(&['a', 'b', '*', '?'], 4);
    let inputs = all_strings(&['a', 'b'], 4);
    assert!(patterns.len() > 300);
    for pattern in &patterns {
        let pattern_chars: Vec<char> = pattern.chars().collect();
        for input in &inputs {
            let input_chars: Vec<char> = input.chars().collect();
            assert_eq!(
                glob_matches(pattern, input),
                reference(&pattern_chars, &input_chars),
                "pattern [{pattern}] input [{input}]"
            );
        }
    }
}

#[test]
fn chained_stars_do_not_backtrack_explosively() {
    let started = Instant::now();
    assert!(!glob_matches(
        &format!("{}*b", "*a".repeat(12)),
        &"a".repeat(36)
    ));
    assert!(!glob_matches(
        &format!("{}*b", "*a".repeat(20)),
        &"a".repeat(2000)
    ));
    assert!(started.elapsed() < Duration::from_secs(3));
}
