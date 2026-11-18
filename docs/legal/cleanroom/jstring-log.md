# Log of the clean-room implementation of jstring.rs

## 1. Files read
- docs/legal/cleanroom/jstring-spec.md
- crates/rakka-actor/src/actor/jstring.rs (the stub; later my own code)
- crates/rakka-actor/src/actor/errors.rs (only `head`/`grep` of the NumberFormatError definition and constructor shape, lines ~1-15 and ~55-130)
- oracle/run.sh (first 20 lines, to learn how to invoke it)
- `ls` of cases/, rust/app-cases/, oracle/ (names only). I did not open the contents of any case file.
- Downloaded data file (outside the repo, in the session scratchpad ucd/): UnicodeData.txt 13.0.0.

## 2. URLs fetched or searched
- https://www.unicode.org/Public/13.0.0/ucd/UnicodeData.txt (curl). Nothing else. No web search. The Javadoc was not fetched (I relied on the spec and general knowledge of the documented API contract).

## 3. Oracle commands
- `echo '"pekko://sys/a#٥"' | oracle/run.sh path-from-string` -> uid 5
- `echo '"pekko://sys/a#-2147483649"' | oracle/run.sh path-from-string` -> NumberFormatException, message `For input string: "-2147483649"`
- A python3.12 loop running `oracle/run.sh path-from-string` with `"pekko://sys/a#<c>"` for, in each of the 37 Nd series generated from UnicodeData.txt, the first digit (expected uid 0) and the last digit (expected uid 9), plus the code points just before and just after each series (expected rejection unless they belong to another series). Result: 0 mismatches.
- `cargo test -p rakka-actor --lib jstring`, `cargo fmt --all`, `cargo clippy -p rakka-actor --all-targets -- -D warnings` (clippy fails only on dead-code errors in uri.rs, a stub outside my file; none in jstring.rs).
- I did not run tools/parity/replay.py (it cannot pass with the uri.rs stubs).

## 4. RED/GREEN
1. Wrote 8 unit tests (spec 2.1/2.2/2.3 examples, oracle values uid 5 for U+0665 and U+FF15, limits, rejects, supplementary digit, series boundaries) against todo!() stubs: RED 8/8 failed.
2. compare_utf16 = UTF-16 code unit iterator comparison: 2 compare tests GREEN, 6 parse tests still RED.
3. parse_int (sign, at least one digit, negative accumulation with checked arithmetic, per-UTF-16-unit digit lookup in a table of 37 series starts generated from UnicodeData.txt 13.0.0, category Nd, <= U+FFFF): 8/8 GREEN.
4. Table verified against the oracle for every series (see section 3).

## 5. What I did NOT read
- Nothing under /Users/pablogore/Projects/rakka, no git history, no OpenJDK or other GPL/LGPL source code, no licence-audit file, no file outside this workspace except the downloaded UnicodeData.txt in the session scratchpad. I did not open address.rs, path.rs, cases/*.jsonl or rust/app-cases contents. No such code was seen by accident.
