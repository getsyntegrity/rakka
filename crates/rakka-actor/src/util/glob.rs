// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/Glob.scala (Apache-2.0).

/// True if `input` matches the glob `pattern` (AC-UTIL-016.6).
///
/// `?` matches exactly one UTF-16 code unit (Q-UTIL-8: Pekko works on `Char`, so a code point
/// outside the BMP needs `??`), `*` matches any run including the empty one, everything else
/// is literal; line breaks are ordinary characters. Runs in time proportional to the product of
/// the two lengths at worst: it remembers only the most recent `*` and never revisits older ones.
#[must_use]
pub fn glob_matches(pattern: &str, input: &str) -> bool {
    const STAR: u16 = b'*' as u16;
    const ANY_ONE: u16 = b'?' as u16;
    let pattern: Vec<u16> = pattern.encode_utf16().collect();
    let input: Vec<u16> = input.encode_utf16().collect();
    let (mut p, mut i) = (0, 0);
    // Where to resume if the run consumed by the most recent `*` turns out to be too short.
    let mut resume: Option<(usize, usize)> = None;

    while i < input.len() {
        match pattern.get(p) {
            Some(&unit) if unit == ANY_ONE || unit == input[i] => {
                p += 1;
                i += 1;
            }
            Some(&STAR) => {
                resume = Some((p, i));
                p += 1;
            }
            _ => match resume {
                Some((star_p, star_i)) => {
                    resume = Some((star_p, star_i + 1));
                    i = star_i + 1;
                    p = star_p + 1;
                }
                None => return false,
            },
        }
    }
    pattern[p..].iter().all(|&unit| unit == STAR)
}
