#!/usr/bin/env python3
"""Proves the recorded replay can fail: replays deliberately broken copies of the Rust port.

For each mutant it copies crates/rakka-actor/ into a scratch workspace (the real sources are never touched),
applies one source edit, builds it, runs `rust-anything app replay --build <mutant binary>`, and
requires a non-zero exit with at least one failing case. The per-mutant replay reports are kept in
rust/divergence-reports/. Finally the unmodified port is replayed again as the control.

  tools/check_divergence.py [mutant ...] [--selftest]

Exit code: 0 every mutant DETECTED and the control clean; 1 a mutant went UNDETECTED or the control failed;
2 the experiment is invalid (patch, build, harness error or timeout). `--selftest` runs an equivalent mutant that must exit 1.
"""
import difflib, os, re, shutil, subprocess, sys, tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import replay_lib

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
CRATE = os.path.join(ROOT, "crates", "rakka-actor")
PARITY = os.path.join("release", "examples", "classic-parity")
REPORTS = os.path.join(ROOT, "rust", "divergence-reports")
CASES_REPORT = os.path.join(ROOT, "rust", "app-cases", "replay-report.md")

# name -> (file, old, new, why it is a realistic bug)
MUTANTS = {
    "utf8-lossy-decoder": (
        "src/actor/uri.rs",
        "decode_utf8_lossy_into(&octets, &mut out);",
        "out.push_str(&String::from_utf8_lossy(&octets));",
        "decode %XX user info with Rust's maximal-subpart UTF-8 replacement instead of the JDK's "
        "(the real bug the differential fuzzer found while porting)",
    ),
    "no-set-terminated-without-children": (
        "src/actor/cell.rs",
        "            *self.children.lock().unwrap() = Children::Terminated; // setTerminated()\n",
        "",
        "an actor without children forgets `setTerminated()`, so its guardian keeps accepting actorOf after "
        "termination (found by the oracle replay while building slice 2a)",
    ),
    "mailbox-is-lifo": (
        "src/actor/cell.rs",
        "                } else {\n                    mb.user.pop_front()\n",
        "                } else {\n                    mb.user.pop_back()\n",
        "the mailbox hands out the newest message first (breaks per-sender ordering)",
    ),
    "receiver-never-sees-sender": (
        "src/actor/cell.rs",
        "self.st.lock().unwrap().sender = env.sender.clone();",
        "self.st.lock().unwrap().sender = None;",
        "`context.sender()` ignores the explicit sender of a message",
    ),
    "forward-drops-sender": (
        "src/actor/props.rs",
        "target.tell_any(msg, self.cell.current_sender());",
        "target.tell_any(msg, None);",
        "`forward` forgets the original sender",
    ),
    "throughput-ignored": (
        "src/actor/cell.rs",
        "let mut budget = cell.system.settings.throughput.max(1);",
        "let mut budget = usize::MAX;",
        "an actor drains its whole mailbox in one turn instead of `throughput` messages",
    ),
    "pill-not-suppressed-from-dead-letters": (
        "src/actor/system.rs",
        "msg.is::<Terminated>() || msg.is::<PoisonPill>()",
        "msg.is::<Terminated>()",
        "a PoisonPill sent to a dead actor is published as DeadLetter (it is DeadLetterSuppression in Pekko)",
    ),
    "drop-uid-in-serialization-format": (
        "src/actor/path.rs",
        "if uid != UNDEFINED_UID {\n            s.push('#');",
        "if false {\n            s.push('#');",
        "toSerializationFormat forgets the #uid suffix",
    ),
    # --- slice 3a: hierarchy identity and DeathWatch ---
    "ref-identity-is-path": (
        "src/actor/actor_ref.rs",
        "a.uid() == b.uid() && a == b",
        "a == b",
        "two references are equal when their paths are equal (the uid is ignored), so a new actor with the old path "
        "is taken for the old one",
    ),
    "children-match-by-path": (
        "src/actor/children.rs",
        "Some(ChildStats::Child(c)) if c == actor => Some(c.clone()),",
        "Some(ChildStats::Child(c)) if c.path() == actor.path() => Some(c.clone()),",
        "the parent recognises a child by path only: a stale reference is treated as the current child",
    ),
    "watch-self-check-by-path": (
        "src/actor/cell.rs",
        "        if *subject == me {\n            return;",
        "        if subject.path() == me.path() {\n            return;",
        "`watch` ignores a reference with the same path as the watcher (a new actor cannot watch its dead predecessor)",
    ),
    "unwatch-keeps-queued-terminated": (
        "src/actor/cell.rs",
        "&& !self.st.lock().unwrap().terminated_queued.remove(&t.actor)\n        {",
        "&& !self.st.lock().unwrap().terminated_queued.remove(&t.actor) && false\n        {",
        "a Terminated already queued is delivered after `unwatch` (Pekko's terminatedQueued is ignored)",
    ),
    "unwatch-is-noop": (
        "src/actor/cell.rs",
        "                st.watching.retain(|w| w != subject);\n                st.watching.len() != before",
        "                st.watching.len() != before",
        "`unwatch` does not remove the watch, so the watcher still gets Terminated",
    ),
    "watch-dead-confirms-existence": (
        "src/actor/cell.rs",
        "                actor: watchee,\n                existence_confirmed: false,",
        "                actor: watchee,\n                existence_confirmed: true,",
        "watching an already terminated actor reports `existenceConfirmed = true`",
    ),
    # --- slice 3b, part 1: failure handling (default strategy) ---
    "kill-restarts-instead-of-stopping": (
        "src/actor/supervision.rs",
        '        | "org.apache.pekko.actor.ActorKilledException"\n        | "org.apache.pekko.actor.DeathPactException" => Directive::Stop,',
        '        | "org.apache.pekko.actor.DeathPactException" => Directive::Stop,',
        "the default decider restarts on `ActorKilledException`, so `Kill` no longer stops the actor",
    ),
    "deathpact-restarts-instead-of-stopping": (
        "src/actor/supervision.rs",
        '        | "org.apache.pekko.actor.ActorKilledException"\n        | "org.apache.pekko.actor.DeathPactException" => Directive::Stop,',
        '        | "org.apache.pekko.actor.ActorKilledException" => Directive::Stop,',
        "an unhandled `Terminated` restarts the watcher instead of stopping it",
    ),
    "init-failure-restarts-instead-of-stopping": (
        "src/actor/supervision.rs",
        '        "org.apache.pekko.actor.ActorInitializationException"\n        | "org.apache.pekko.actor.PostRestartException"',
        '        "org.apache.pekko.actor.PostRestartException"',
        "a `preStart` failure is restarted instead of stopping the actor",
    ),
    "exception-stops-instead-of-restarting": (
        'src/actor/cell.rs',
        'match strategy.decide(&cause) {',
        'match { let _ = strategy.decide(&cause); Directive::Stop } {',
        "every failure stops the child (the old pre-supervision behaviour)",
    ),
    "post-restart-not-called": (
        "src/actor/cell.rs",
        "a.post_restart(ctx, &cause)",
        "a.pre_start(ctx)",
        "the fresh instance skips `postRestart` and goes straight to `preStart`",
    ),
    "pre-restart-without-message": (
        "src/actor/cell.rs",
        "a.pre_restart(ctx, &cause, message.as_ref())",
        "a.pre_restart(ctx, &cause, None)",
        "`preRestart` is not told the message that failed",
    ),
    "failure-does-not-suspend-mailbox": (
        'src/actor/cell.rs',
        '        self.mb.lock().unwrap().suspend += 1; // suspendNonRecursive\n        for k in',
        '        for k in',
        "the failed actor keeps processing its mailbox while its supervisor decides",
    ),
    # --- slice 3b, part 2: OneForOneStrategy, directives, restart limit ---
    "resume-confused-with-restart": (
        "src/actor/cell.rs",
        "Directive::Resume => child.send_system(SystemMessage::Resume { cause: Some(cause) }),",
        "Directive::Resume => child.send_system(SystemMessage::Recreate { cause }),",
        "the `Resume` directive restarts the child (new instance, state lost) instead of keeping it",
    ),
    "restart-confused-with-resume": (
        'src/actor/cell.rs',
        '                k.send_system(SystemMessage::Recreate {\n                    cause: cause.clone(),\n                });',
        '                k.send_system(SystemMessage::Resume {\n                    cause: Some(cause.clone()),\n                });',
        "the `Restart` directive resumes the child (no restart hooks, state kept) instead of replacing it",
    ),
    "restart-limit-allows-one-more": (
        "src/actor/supervision.rs",
        "self.count <= max_nr_of_retries",
        "self.count <= max_nr_of_retries + 1",
        "the restart limit is shifted up by one: the first restart beyond `maxNrOfRetries` is still granted",
    ),
    "restart-limit-allows-one-fewer": (
        "src/actor/supervision.rs",
        "self.count <= max_nr_of_retries",
        "self.count < max_nr_of_retries",
        "the restart limit is shifted down by one: the last allowed restart is refused",
    ),
    "minus-one-is-not-unlimited": (
        "src/actor/supervision.rs",
        "if max_nr_of_retries < 0 {",
        "if max_nr_of_retries < -1 {",
        "`maxNrOfRetries = -1` (unlimited) is treated as a zero limit",
    ),
    "restart-counter-shared-by-siblings": (
        'src/actor/cell.rs',
        '.entry(k.clone())\n                    .or_default()\n                    .request_restart_permission(max)',
        '.entry(self.self_ref())\n                    .or_default()\n                    .request_restart_permission(max)',
        "all children share one restart counter instead of having one each",
    ),
    # --- slice 3b-3: Escalate ---
    "escalate-resolved-locally": (
        "src/actor/cell.rs",
        "Directive::Escalate => self.escalate(child, cause),",
        "Directive::Escalate => child.send_system(SystemMessage::Resume { cause: Some(cause) }),",
        "the parent resolves locally (resumes the child) a failure its decider escalated: the grandparent never hears of it",
    ),
    "escalate-alters-cause": (
        'src/actor/cell.rs',
        '        self.fail(cause, Some(message), Some(child));',
        '        let cause = Throwable::new("org.apache.pekko.actor.ActorInitializationException", cause.message.as_deref());\n        self.fail(cause, Some(message), Some(child));',
        "the escalation rewraps the cause: the grandparent's decider sees another exception class (observable in its `decide` event and in the default decision)",
    ),
    "escalate-never-resumes-children": (
        'src/actor/cell.rs',
        'let own = if Some(&k) == perp.as_ref() {\n                cause.clone()\n            } else {\n                None\n            };',
        'let own: Option<Throwable> = None;',
        "after the grandparent resumes the parent, the child that failed is never resumed (stays suspended)",
    ),
    # --- slice 3b-4: the subtree of an actor that failed on its own is suspended ---
    "failure-does-not-suspend-children": (
        "src/actor/cell.rs",
        "            k.send_system(SystemMessage::Suspend); // suspendChildren\n",
        "",
        "an actor that failed keeps its children running while its supervisor decides",
    ),
    "suspension-not-recursive": (
        "src/actor/cell.rs",
        "            k.send_system(SystemMessage::Suspend);\n        }\n",
        "        }\n",
        "a suspended child does not suspend its own children: grandchildren keep processing while the failed ancestor waits",
    ),
    # --- slice 3b-5: AllForOneStrategy ---
    "all-for-one-degrades-to-one-for-one": (
        "src/actor/cell.rs",
        "let affected = if strategy.is_all_for_one() {",
        "let affected = if false {",
        "AllForOne behaves like OneForOne: only the failed child is restarted or stopped",
    ),
    "all-for-one-stop-only-failed": (
        "src/actor/cell.rs",
        "            for k in &affected {\n                self.stop_actor(k);\n            }\n",
        "            self.stop_actor(&child);\n",
        "under AllForOne a Stop (or an exhausted restart limit) stops only the failed child, the healthy sibling survives",
    ),
    # --- consolidation: JDK 17 oracle and the &str input domain (decisions Q-X-16, Q-X-17) ---
    "compare-by-code-point": (
        "src/actor/jstring.rs",
        "a.encode_utf16().cmp(b.encode_utf16())",
        "a.cmp(b)",
        "string order by code point (Rust `str` order) instead of UTF-16 code units: supplementary characters sort after U+FFFF",
    ),
    "ascii-digits-only": (
        "src/actor/jstring.rs",
        "let digit = decimal_digit_value(unit).ok_or_else(fail)?;",
        "let digit = (if unit < 128 { decimal_digit_value(unit) } else { None }).ok_or_else(fail)?;",
        "a uid written with non-ASCII decimal digits (Arabic-Indic, fullwidth) is rejected instead of parsed",
    ),
    # --- slice P02: Timeout, Helpers, Done/NotUsed, base exceptions ---
    "lower-case-ascii-only": (
        "src/util/helpers.rs",
        "    s.to_lowercase()\n",
        "    s.to_ascii_lowercase()\n",
        "toRootLowerCase only lower-cases ASCII: non-ASCII letters (Cyrillic, Georgian, final sigma) stay upper-case",
    ),
    "timeout-max-rejected": (
        "src/util/timeout.rs",
        "if nanos == i64::MIN {",
        "if nanos == i64::MIN || nanos == i64::MAX {",
        "the largest representable Timeout (2^63-1 ns) is rejected as out of range (off-by-one at the upper bound)",
    ),
    "requirement-prefix-missing": (
        "src/util/helpers.rs",
        '"requirement failed: {msg}"',
        '"{msg}"',
        "`requiring` fails with the bare message: the `requirement failed: ` prefix of Predef.require is lost",
    ),
    # --- slice P06: UTIL-PLATAFORMA ---
    "utc-string-milliseconds-unpadded": (
        "src/util/helpers.rs",
        "        zero_if(ms < 100),\n",
        "        zero_if(false),\n",
        "currentTimeMillisToUTCString does not zero-pad the hundreds of milliseconds: 5 ms prints as `.05UTC`",
    ),
    "scheduled-clock-increments-by-two": (
        "src/util/clock.rs",
        "self.time.fetch_add(1, Ordering::AcqRel).wrapping_add(1)",
        "self.time.fetch_add(2, Ordering::AcqRel).wrapping_add(2)",
        "ScheduledClock.currentTime advances the time by two between scheduler updates instead of one",
    ),
    "swar-last-index-counts-from-front": (
        "src/util/swar.rs",
        "((63 - word.trailing_zeros()) >> 3) as i32",
        "(word.leading_zeros() >> 3) as i32",
        "SWARUtil.getLastIndex returns the first match (counting from the most significant byte) instead of the last",
    ),
    # --- slice P65: HashCode ---
    "hash-code-long-no-fold": (
        "src/util/hash_code.rs",
        "(*self ^ ((*self as u64) >> 32) as i64) as i32",
        "*self as i32",
        "HashCode.hash(seed, Long) drops the high word instead of folding `v ^ (v >>> 32)`",
    ),
    "hash-code-nan-payload-kept": (
        "src/util/hash_code.rs",
        "        let bits = if self.is_nan() {\n            0x7fc0_0000_u32\n        } else {\n            self.to_bits()\n        };",
        "        let bits = self.to_bits();",
        "Float hashing keeps the NaN payload instead of collapsing to the canonical NaN (floatToIntBits vs floatToRawIntBits)",
    ),
    "hash-code-string-utf8-units": (
        "src/util/hash_code.rs",
        "self.encode_utf16().fold(0_i32, |h, unit| {",
        "self.bytes().fold(0_i32, |h, unit| {",
        "a string contributes its UTF-8 bytes instead of UTF-16 units: non-ASCII text hashes differently",
    ),
    # --- slice P66: PekkoVersion.require ---
    "pekko-version-patch-compared-as-text": (
        "src/pekko_version.rs",
        "to_int(required_parsed.patch)? > to_int(current_parsed.patch)?",
        "required_parsed.patch > current_parsed.patch",
        "patch numbers are compared as text instead of as ints once their text differs: `2.5.7` against `2.5.0007` is refused",
    ),
    "pekko-version-rc-accepts-milestone": (
        "src/pekko_version.rs",
        "(Some((PreRelease::ReleaseCandidate, _)), Some((PreRelease::Milestone, _))) => {\n            unsupported()\n        }",
        "(Some((PreRelease::ReleaseCandidate, _)), Some((PreRelease::Milestone, _))) => {\n            Ok(())\n        }",
        "a requirement on an RC is satisfied by a milestone of the same patch (a milestone is older than an RC)",
    ),
    "pekko-version-final-accepts-pre-release": (
        "src/pekko_version.rs",
        "(None, Some(_)) => unsupported(),",
        "(None, Some(_)) => Ok(()),",
        "a requirement on a final version is satisfied by an M/RC of the same patch",
    ),
    # --- slice P67: Version ---
    "version-final-below-qualified": (
        "src/util/version.rs",
        "(true, false) => 1,",
        "(true, false) => compare_to_raw(&self.rest, &other.rest),",
        "the 1.7.1 defect (Q-UTIL-6 rejected by the Owner): a version without qualifier compares below the same version with one",
    ),
    "version-dynver-count-ignored": (
        "src/util/version.rs",
        "for (a, b) in self.numbers.iter().zip(&other.numbers) {",
        "for (a, b) in self.numbers.iter().zip(&other.numbers).take(3) {",
        "the dynver commit count (fourth number) is not compared: 1.0.10+3-hash equals 1.0.10+10-hash apart from the text of the rest",
    ),
    "version-difference-saturates": (
        "src/util/version.rs",
        "let diff = a.wrapping_sub(*b);",
        "let diff = a.saturating_sub(*b);",
        "the difference of two numbers saturates instead of wrapping like Int subtraction: 2147483647.0.0 against -1.0.0 is positive",
    ),
    # --- slice P68: PrettyDuration ---
    "pretty-duration-rounds-half-down": (
        "src/util/pretty_duration.rs",
        "let round_up = digits[precision] >= 5;",
        "let round_up = digits[precision] > 5;",
        "a 5 right after the last kept digit is not rounded up (12.345 us prints 12.34): Java rounds HALF_UP",
    ),
    "pretty-duration-no-scientific-at-power": (
        "src/util/pretty_duration.rs",
        "if exponent < precision as i32 {",
        "if exponent <= precision as i32 {",
        "the rounded value 10^precision stays in decimal notation (10000 days prints 10000.0 d, not 1.000e+04 d)",
    ),
    "pretty-duration-shortest-digits-of-big-integers": (
        "src/util/pretty_duration.rs",
        "if (9_007_199_254_740_992.0..=9_223_372_036_854_775_808.0).contains(&value) {",
        "if false {",
        "a negative duration of 2^53 ns or more uses the shortest digits of the double (what C prints) instead of the integer digits Java prints",
    ),
    # --- slice P69: UUIDComparator ---
    "uuid-order-signed-halves": (
        "src/util/uuid_order.rs",
        "fn compare_uints(i1: i32, i2: i32) -> i32 {",
        "fn compare_uints(i1: i32, i2: i32) -> i32 {\n    if true {\n        return i1.wrapping_sub(i2);\n    }",
        "the 32-bit halves are compared as signed numbers (like java.util.UUID.compareTo) instead of unsigned: a set top bit sorts first",
    ),
    "uuid-order-version-one-by-msb": (
        "src/util/uuid_order.rs",
        "    if version == 1 {\n        // `timestamp()`",
        "    if version == 255 {\n        // `timestamp()`",
        "two version-1 UUIDs are ordered by their raw most significant bits instead of by timestamp()",
    ),
    "uuid-order-timestamp-keeps-version": (
        "src/util/uuid_order.rs",
        "let time_hi = msb & 0x0FFF;",
        "let time_hi = msb & 0xFFFF;",
        "timestamp() keeps the four version bits in time_hi: a 64-bit value instead of the 60-bit time",
    ),
    # --- slice P70: AlwaysIncreasingClock ---
    "wall-clock-stall-repeats": (
        "src/util/wall_clock.rs",
        "            let next = if current_system_time <= last {\n                last.wrapping_add(1)",
        "            let next = if current_system_time <= last {\n                last",
        "a clock that is not behind the system time repeats the previous value instead of adding one",
    ),
    "wall-clock-follows-system-time": (
        "src/util/wall_clock.rs",
        "            let next = if current_system_time <= last {",
        "            let next = if current_system_time <= last && false {",
        "the clock always returns the system time, so a clock ahead of it (or a system time that goes backwards) moves back",
    ),
    "wall-clock-saturates": (
        "src/util/wall_clock.rs",
        "                last.wrapping_add(1)",
        "                last.saturating_add(1)",
        "the increment saturates at Long.MAX_VALUE instead of wrapping around to Long.MIN_VALUE like Java",
    ),
    # --- slice P75: LineNumbers (closure origin) ---
    "line-numbers-range-always": (
        "src/util/line_numbers.rs",
        "Self::SourceFileLines { filename, from, to } if from != to =>",
        "Self::SourceFileLines { filename, from, to } if true =>",
        "a one-line origin is shown as a range (`file:22-22`) instead of `file:22`",
    ),
    "line-numbers-keeps-directory": (
        "src/util/line_numbers.rs",
        "let filename = path.rsplit(['/', '\\\\']).next().unwrap_or(path);",
        "let filename = path;",
        "the origin keeps the directory of the source file instead of its bare file name",
    ),
    "line-numbers-pretty-without-package": (
        "src/util/line_numbers.rs",
        'format!("{package}/{lines}")',
        'format!("{lines}")',
        "prettyName omits the `<package>/` prefix of a located closure",
    ),
    # --- slice P03: data structures (Index, Switch, TokenBucket, WildcardIndex, sketches, queues, ImmutableIntMap) ---
    "index-put-duplicate-is-new": (
        "src/util/index.rs",
        "Ok(_) => false,",
        "Ok(_) => true,",
        "`put` reports a value that was already associated with the key as new",
    ),
    "token-bucket-no-tick-at-exact-period": (
        "src/util/token_bucket.rs",
        "if time_elapsed >= period {",
        "if time_elapsed > period {",
        "a token that arrives exactly one period after the last update is not counted until the next period",
    ),
    "switch-no-revert-on-panic": (
        "src/util/switch.rs",
        "self.compare_and_set(!from, from); // revert status",
        "// the switch keeps its new state even though the action panicked",
        "a panicking action no longer reverts the switch (SwitchSpec `revert when exception`)",
    ),
    "wildcard-skips-double-fallback": (
        "src/util/wildcard_index.rs",
        "if with_single_wildcard.is_none_or(WildcardTree::is_empty) {",
        "if false {",
        "a path with no single-wildcard match never falls back to the terminal `**` entries",
    ),
    "frequency-sketch-reset-keeps-neighbour-bits": (
        "src/util/frequency_sketch.rs",
        "(slot >> 1) & self.reset_mask",
        "slot >> 1",
        "the TinyLFU reset shifts the counters without masking, so the low bit of each counter leaks into its neighbour",
    ),
    "murmur3-mix-last-rotation": (
        "src/util/frequency_sketch.rs",
        "k = k.rotate_left(15);",
        "k = k.rotate_left(14);",
        "MurmurHash3.mixLast rotates by 14 instead of 15, so every sketch hash (and the exact counters of FrequencySketchSpec) changes",
    ),
    "stable-queue-ties-last-in-first-out": (
        "src/util/stable_priority_queue.rs",
        "(a.seq_num.wrapping_sub(b.seq_num) as i64) < 0,",
        "(a.seq_num.wrapping_sub(b.seq_num) as i64) > 0,",
        "elements of equal priority come out last in first out instead of first in first out",
    ),
    "int-map-hash-starts-at-zero": (
        "src/util/immutable_int_map.rs",
        ".fold(1_i32,",
        ".fold(0_i32,",
        "ImmutableIntMap.hashCode starts from 0 instead of 1 like Arrays.hashCode (1281 for {10 -> 10})",
    ),
    # --- slice P05: Glob ---
    "glob-trailing-star-unmatched": (
        "src/util/glob.rs",
        "pattern[p..].iter().all(|&unit| unit == STAR)",
        "p == pattern.len()",
        "trailing `*`s are not allowed to match nothing: `a*` no longer matches `a`",
    ),
    "glob-question-rejects-surrogate-unit": (
        "src/util/glob.rs",
        "Some(&unit) if unit == ANY_ONE || unit == input[i] => {",
        "Some(&unit) if (unit == ANY_ONE && !(0xD800..0xE000).contains(&input[i])) || unit == input[i] => {",
        "`?` refuses a UTF-16 surrogate unit, so a code point outside the BMP can no longer be matched by `??`",
    ),
    "glob-star-in-input-not-literal": (
        "src/util/glob.rs",
        "Some(&unit) if unit == ANY_ONE || unit == input[i] => {",
        "Some(&unit) if unit != STAR && (unit == ANY_ONE || unit == input[i]) => {",
        "a `*` in the pattern never matches a literal `*` of the input first (it always starts a run), unlike Glob.scala:51",
    ),
    "lossy-stdin-decoder": (
        "examples/classic-parity/json.rs",
        "String::from_utf16(&out).map_err(|_| OUT_OF_DOMAIN.to_string())",
        "Ok(String::from_utf16_lossy(&out))",
        "the harness replaces a lone UTF-16 surrogate with U+FFFD instead of rejecting the out-of-domain input explicitly",
    ),
    "ipv4-octet-overflow-swallowed": (
        "src/actor/uri.rs",
        ".ok_or_else(|| {\n                    UriError::NumberFormat(NumberFormatError::new(format!(\n                        \"Error at index {index} in: \\\"{part}\\\"\"\n                    )))\n                })?;",
        ".unwrap_or(256);",
        "an overflowing IPv4 octet in an IPv6 host no longer escapes as NumberFormatException (the JDK 17 behaviour Pekko 2.0.0 shows)",
    ),

    # --- slice P04: ByteString, ByteStringBuilder, ByteIterator and the java.io adapters ---
    "byte-string-take-right-saturates": (
        "src/util/byte_string/mod.rs",
        "self.slice(length.wrapping_sub(n), length)",
        "self.slice(length.saturating_sub(n), length)",
        "`take_right(Int.MinValue)` stops overflowing to the whole value (Scala computes `length - n` with Int wrap-around)",
    ),
    "utf8-surrogate-replaced-byte-by-byte": (
        "src/util/byte_string/charset.rs",
        "let consumed = if bytes[position] == 0xED",
        "let consumed = if bytes[position] == 0xEE",
        "an encoded surrogate (ED A0..BF xx) yields one U+FFFD per byte instead of one for the whole sequence, as the JVM decoder does",
    ),
    "long-part-little-endian-shifts-as-long": (
        "src/util/byte_string/iterator.rs",
        "value |= i64::from((byte as i32).wrapping_shl(i.wrapping_mul(8) as u32)) as u64;",
        "value |= u64::from(byte).wrapping_shl(i.wrapping_mul(8) as u32);",
        "`getLongPart(n, LITTLE_ENDIAN)` shifts a Long instead of Pekko's Int (shift modulo 32 and sign extension past 4 bytes)",
    ),
    # --- slice P77: Timeout.create(java.time.Duration) through the Java<->Scala duration converter ---
    "duration-nanos-sum-wraps": (
        "src/util/timeout.rs",
        ".and_then(|seconds_as_nanos| seconds_as_nanos.checked_add(i64::from(nanos)))",
        ".map(|seconds_as_nanos| seconds_as_nanos.wrapping_add(i64::from(nanos)))",
        "converting a java.time.Duration of seconds and nanos that passes 2^63-1 ns wraps to a negative Timeout instead of failing",
    ),
    "duration-seconds-multiply-wraps": (
        "src/util/timeout.rs",
        ".and_then(|seconds| seconds.checked_mul(1_000_000_000))",
        ".map(|seconds| seconds.wrapping_mul(1_000_000_000))",
        "seconds-to-nanoseconds is not checked: a duration of more than 292 years with a nanosecond part becomes a small positive Timeout",
    ),
    "duration-text-minutes-unreduced": (
        "src/util/timeout.rs",
        "seconds % 3600 / 60",
        "seconds / 60",
        "the overflow message prints the minutes of the duration without taking them modulo the hour (PT2562047H153722867M...)",
    ),
    # --- slice P07: HOCON parser, resolver, units, checkValid and render ---
    "hocon-self-reference-forgets-lower-layer": (
        "src/config/resolve.rs",
        "                Some(v) => Ok(Some(v.clone())),",
        "                Some(_) => Ok(None),",
        "a key that refers to itself (`x = ${?x} [..]`, `x += ..`) ignores what the layers below set, so library-extensions of the application replace the defaults",
    ),
    "hocon-duration-floors-negative": (
        "src/config/units.rs",
        "clamp(total / i128::from(target.nanos()))",
        "clamp(total.div_euclid(i128::from(target.nanos())))",
        "a negative duration is converted by flooring instead of truncating toward zero (-1500 us is -2 ms, not -1 ms)",
    ),
    "hocon-size-accepts-uppercase-kb": (
        "src/config/units.rs",
        '&["kB", "kilobyte", "kilobytes"],',
        '&["kB", "KB", "kilobyte", "kilobytes"],',
        "`1 KB` is accepted as a size although the HOCON library rejects it (only kB is the decimal kilobyte)",
    ),
    "hocon-boolean-on-ignores-case": (
        "src/config/mod.rs",
        'Kind::Str(s) if matches!(s.as_str(), "true" | "yes" | "on") => Ok(true),',
        'Kind::Str(s) if matches!(s.to_lowercase().as_str(), "true" | "yes" | "on") => Ok(true),',
        "the quoted string \"ON\" is read as a boolean although only the lowercase spellings are",
    ),
    "hocon-check-valid-string-reference-needs-string": (
        "src/config/validate.rs",
        "        Kind::Str(_) => true,",
        "        Kind::Str(_) => matches!(value.kind, Kind::Str(_)),",
        "checkValid rejects a number where the reference has a string, so a valid application config (`pekko.log-dead-letters = 10`) fails to start",
    ),
    "hocon-check-valid-list-accepts-number": (
        "src/config/validate.rs",
        "Kind::List(_) => matches!(value.kind, Kind::List(_) | Kind::Obj(_)),",
        "Kind::List(_) => matches!(value.kind, Kind::List(_) | Kind::Obj(_) | Kind::Num(_)),",
        "checkValid accepts a number where the reference has a list, hiding a mistyped key until it is read",
    ),
    "hocon-render-shows-environment-values": (
        "src/config/validate.rs",
        "Kind::Str(_) if v.line == 0 =>",
        "Kind::Str(_) if v.line == u32::MAX =>",
        "renderWithRedactions prints the value of an environment variable (a secret) in Settings.toString and error messages",
    ),
}


def sh(cmd, timeout=None, **kw):
    return subprocess.run(cmd, stdin=subprocess.DEVNULL, capture_output=True, text=True, timeout=timeout, **kw)


REPLAY_TIMEOUT = 900   # seconds; a hung replay is reported as TIMEOUT, never as "detected"
BUILD_TIMEOUT = 900
SUMMARY = re.compile(r"(\d+)/(\d+) cases passed")

# Outcome classes. Only DETECTED counts as a caught mutant; everything else is reported separately.
DETECTED, UNDETECTED, PATCH_ERROR, BUILD_ERROR, HARNESS_ERROR, TIMEOUT = (
    "DETECTED", "UNDETECTED", "PATCH_ERROR", "BUILD_ERROR", "HARNESS_ERROR", "TIMEOUT")


def replay(binary):
    return replay_lib.run_replay(binary, REPLAY_TIMEOUT)


def classify(r, binary):
    """-> (outcome, passed, total, line), counting only applicable cases (NOT_APPLICABLE ones must be rejected
    explicitly, see replay_lib). A mutant is detected only when the replay ran to completion and either an
    applicable case diverged or a NOT_APPLICABLE input was no longer rejected explicitly."""
    ev = replay_lib.evaluate(binary, r)
    if ev["ok"] is None:
        return HARNESS_ERROR, None, None, ev["line"]
    detail = ev["line"] + "".join(f"; {p}" for p in ev["na_problems"])
    if ev["ok"]:
        return UNDETECTED, ev["passed"], ev["total"], detail
    return DETECTED, ev["passed"], ev["total"], detail


def run_mutant(scratch, name, rel, old, new, why):
    # a scratch workspace holds the copied crate (it inherits version/edition from the workspace)
    ws = os.path.join(scratch, name)
    work = os.path.join(ws, "rakka-actor")
    shutil.copytree(CRATE, work, ignore=shutil.ignore_patterns("target"))
    shutil.copy(os.path.join(ROOT, "Cargo.toml"), os.path.join(ws, "Cargo.toml"))
    rootm = open(os.path.join(ws, "Cargo.toml")).read()
    rootm = re.sub(r'(?s)^.*?\[workspace\]', '[workspace]', rootm)
    rootm = rootm.replace('members = [".", "crates/rakka-actor", "crates/rakka-actor-typed"]', 'members = ["rakka-actor"]')
    open(os.path.join(ws, "Cargo.toml"), "w").write(rootm)
    path = os.path.join(work, rel)
    src = open(path).read()
    if src.count(old) != 1:
        return PATCH_ERROR, None, None, f"pattern occurs {src.count(old)} times in {rel}"
    patched = src.replace(old, new)
    open(path, "w").write(patched)
    if patched == src and name not in EQUIVALENT:
        return PATCH_ERROR, None, None, "patch did not change the file"
    print("".join(difflib.unified_diff(src.splitlines(True), patched.splitlines(True), f"a/rakka-actor/{rel}", f"b/rakka-actor/{rel}")))
    try:
        b = sh(["cargo", "build", "--release", "--manifest-path", os.path.join(work, "Cargo.toml"),
                "--target-dir", os.path.join(scratch, "target-" + name), "--example", "classic-parity"], timeout=BUILD_TIMEOUT)
    except subprocess.TimeoutExpired:
        return TIMEOUT, None, None, "cargo build"
    if b.returncode != 0:
        return BUILD_ERROR, None, None, b.stderr[-600:]
    binary = os.path.join(scratch, "target-" + name, PARITY)
    if not os.path.exists(binary):
        return BUILD_ERROR, None, None, "mutated binary missing"
    try:
        r = replay(binary)
    except subprocess.TimeoutExpired:
        return TIMEOUT, None, None, "rust-anything replay"
    shutil.copy(CASES_REPORT, os.path.join(REPORTS, f"replay-report-{name}.md"))
    return classify(r, binary)


# `--selftest` adds a mutant that changes nothing observable (a whitespace edit): it MUST come out UNDETECTED, so the run must exit non-zero.
EQUIVALENT = {"selftest-equivalent-whitespace"}
SELFTEST = {"selftest-equivalent-whitespace": (
    "src/actor/supervision.rs", "self.count <= max_nr_of_retries", "self.count <= max_nr_of_retries /* same */",
    "self-test of the launcher: an equivalent edit that must NOT be detected")}


def check_patterns():
    """`--check-patterns`: every mutant's pattern must occur exactly once (stale patterns after a refactor exit 2)."""
    bad = 0
    for name, (rel, old, new, _why) in {**MUTANTS, **SELFTEST}.items():
        n = open(os.path.join(CRATE, rel)).read().count(old)
        if n != 1:
            bad += 1
            print(f"STALE {name}: {n} occurrences in {rel}")
    print(f"{len(MUTANTS) + len(SELFTEST)} mutant patterns checked, {bad} stale")
    sys.exit(2 if bad else 0)


def main():
    args = sys.argv[1:]
    if "--check-patterns" in args:
        check_patterns()
    selftest = "--selftest" in args
    only = [a for a in args if not a.startswith("--")]  # optional: run just these mutants (the control still runs)
    mutants = dict(SELFTEST) if selftest else dict(MUTANTS)
    for n in only:
        assert n in mutants, f"unknown mutant {n}"
    os.makedirs(REPORTS, exist_ok=True)
    scratch = tempfile.mkdtemp(prefix="pekko-mutants-")
    results = {}
    try:
        for name, (rel, old, new, why) in mutants.items():
            if only and name not in only:
                continue
            print(f"=== mutant {name}: {why}", flush=True)
            res = run_mutant(scratch, name, rel, old, new, why)
            results[name] = res
            print(f"{res[0]}  {res[3]}\n", flush=True)
        try:
            control_binary = os.path.join(ROOT, "target", PARITY)
            c = replay(control_binary)
            c_out, c_pass, c_total, c_line = classify(c, control_binary)
            control_ok = c_out == UNDETECTED
        except subprocess.TimeoutExpired:
            control_ok, c_pass, c_total, c_line = False, None, None, "TIMEOUT"
        print(f"=== control (unmodified rakka-actor): exit={c.returncode if c_pass is not None else '?'}  {c_line}", flush=True)
    finally:
        shutil.rmtree(scratch, ignore_errors=True)
    # a replay of a different corpus size would make the pass counts incomparable
    sizes = {t for (_o, _p, t, _l) in results.values() if t is not None} | ({c_total} if c_total else set())
    rows = ["| mutant | outcome | passed/total |", "|---|---|---|"]
    rows += [f"| {n} | {o} | {f'{p}/{t}' if p is not None else '-'} |" for n, (o, p, t, _l) in results.items()]
    rows.append(f"| control (unmodified) | {'CLEAN' if control_ok else 'FAILED'} | {f'{c_pass}/{c_total}' if c_pass is not None else '-'} |")
    table = "\n".join(rows)
    counts = {k: sum(1 for r in results.values() if r[0] == k) for k in (DETECTED, UNDETECTED, PATCH_ERROR, BUILD_ERROR, HARNESS_ERROR, TIMEOUT)}
    print("\n" + table + f"\n\ncorpus sizes seen: {sorted(sizes)}  outcomes: {counts}  detected {counts[DETECTED]}/{len(results)}")
    if not selftest and not only:
        with open(os.path.join(REPORTS, "SUMMARY.md"), "w") as fh:
            fh.write(table + f"\n\noutcomes: {counts}\n")
    errors = counts[PATCH_ERROR] + counts[BUILD_ERROR] + counts[HARNESS_ERROR] + counts[TIMEOUT]
    if errors or len(sizes) > 1:
        sys.exit(2)          # the experiment itself is invalid: not a verdict on the harness
    sys.exit(0 if counts[UNDETECTED] == 0 and control_ok else 1)


if __name__ == "__main__":
    main()
