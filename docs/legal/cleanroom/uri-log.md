# Clean-room log: crates/rakka-actor/src/actor/uri.rs

## 1. Files read (all inside the workspace)
- docs/legal/cleanroom/uri-spec.md
- crates/rakka-actor/src/actor/uri.rs (the original stub)
- crates/rakka-actor/src/actor/errors.rs (only the `NumberFormatError` definition region, via grep), jstring.rs (first ~30 lines, for style), address.rs (lines ~70-110, via sed) and path.rs (lines 1-30, via sed): consumers of `uri::parse`.
- cases/path-from-string.jsonl (first lines), cases/oracle-path-address.jsonl (the UTF-* case lines), cases/oracle-case-ids.tsv (grep).
- oracle/run.sh, tools/parity/replay.py (head), tools/parity/fuzz_diff.py (head, token lists).
- grep output listing crates/rakka-actor/examples/classic-parity paths (file names only).
- Probe scripts I wrote myself, kept outside the crate: scratchpad/probe/{dec.py,host.py,diff2.py,diff3.py,diff4.py}.

## 2. URLs fetched
- https://www.unicode.org/Public/13.0.0/ucd/UnicodeData.txt (one curl, to list categories Zs/Zl/Zp and Cc; used to hardcode the "other character" exclusions).
- No web search. I did not open the Javadoc, RFCs or any other page (I relied on the spec and on general knowledge of RFC 2396/2732 grammar).

## 3. Oracle / fuzzer commands run
- `oracle/run.sh path-from-string-lines`: about 30 batch invocations from my own probe scripts (hostname rules, IPv6 forms, scope ids, embedded IPv4 overflow, UTF-8 escape decoding exhaustive 1-2 byte and selected 3-5 byte sequences, ~970 structural URI shapes, ~54k random strings) plus one single-line manual check. Each batch compared against the Rust `classic-parity` binary.
- `python3.12 tools/parity/replay.py`: about 6 times.
- `python3.12 tools/parity/fuzz_diff.py --count 20000 --seed N`: seeds 1 and 2 several times, seeds 3 and 4 once; final: 0 divergences on seeds 1 and 2.
- `cargo build/fmt/clippy/test`.

## 4. RED / GREEN story
- RED: stub with `todo!()`.
- First implementation written from the spec plus general RFC knowledge; replay was already 1044/1044. The UTF-8 rule fitted in the first draft (invalid lead or bad continuation: one U+FFFD per byte; truncated valid prefix: one U+FFFD for the prefix; a surrogate-encoding ED A0..BF swallows its continuation into one U+FFFD) matched the oracle on all exhaustive probes.
- Fuzz seed 1/2 found 39/32 divergences: a single-label host starting with a digit (`1h`, `0x1c`, `256`) is a valid host; the "last label starts with a letter" rule applies only to multi-label names. Fixed -> 0.
- Own differential probes then found: (a) embedded IPv4 stops after the fourth octet, (b) the IPv6 address is validated before the scope id (so an overflow wins over a bad scope), (c) scope ids allow only alphanumerics, `.` and `_`, (d) the server-based form is only attempted when all authority characters are in the server character set (userinfo chars, `.`, `-`, `@`, `[`, `]`, `%`, other), which explains why a space after an overflowing octet gives syntax error instead of NumberFormat, (e) an embedded IPv4 tail is only recognised after a `:`. All fixed; differential probes 0 divergences.
- Unit tests (10) added in uri.rs; one initial test of mine was wrong (a valid user-info/host pair assumed to be registry-based) and was corrected.

## 5. Not read
I did not read, open or search any OpenJDK or other GPL/LGPL source, the `url`/`http` crates, any file outside the workspace (nothing under /Users/pablogore/Projects/rakka, no git history), nor any licence-audit file. No such code was seen by accident. The Javadoc and RFC documents were not fetched.
