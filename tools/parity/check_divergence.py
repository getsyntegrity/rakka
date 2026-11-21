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
import collections, difflib, json, os, re, shutil, signal, subprocess, sys, tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import replay_lib

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
CRATE = os.path.join(ROOT, "crates", "rakka-actor")
TOOLS = os.path.dirname(os.path.abspath(__file__))
PARITY = os.path.join("release", "examples", "classic-parity")
REPORTS = os.path.join(ROOT, "rust", "divergence-reports")

# name -> (file, old, new, why it is a realistic bug)
MUTANTS = {
    "mailbox-suspension-does-not-stop-user-messages": (
        "src/actor/cell.rs",
        "        while budget > 0 && self.status.should_process_message() {",
        "        while budget > 0 && !self.status.is_closed() {",
        "a suspended actor keeps processing user messages (the turn only checks `Closed`, not the suspend count)",
    ),
    "mailbox-throughput-off-by-one": (
        "src/actor/cell.rs",
        "self.system.settings.throughput.max(1)",
        "(self.system.settings.throughput.max(1) + 1)",
        "a turn processes one message more than `throughput`, breaking the fairness between actors on one thread",
    ),
    "mailbox-dead-user-messages-not-published": (
        "src/actor/cell.rs",
        "        for env in dead {",
        "        for env in dead.into_iter().take(0) {",
        "a user message sent to a closed mailbox is dropped instead of published as a dead letter",
    ),
    "mailbox-dead-system-message-not-answered": (
        "src/actor/cell.rs",
        "                Self::dead_system_message(msg);\n                return;",
        "                drop(msg);\n                return;",
        "a system message sent to a terminated actor is dropped, so a `Watch` is never answered with `Terminated` (DeathWatch hole)",
    ),
    "mailbox-queue-is-lifo": (
        "src/dispatch/message_queue.rs",
        "        self.queue.lock().unwrap().pop_front()\n",
        "        self.queue.lock().unwrap().pop_back()\n",
        "the default queue hands out the newest message first, reversing every actor's message order (MBX-MQ-01, MBX-MQ-02)",
    ),
    "mailbox-clean-up-reversed": (
        "src/dispatch/message_queue.rs",
        "        while let Some(envelope) = self.dequeue() {",
        "        while let Some(envelope) = { let newest = self.queue.lock().unwrap().pop_back(); newest } {",
        "cleanUp ships the pending messages to dead letters newest first (MBX-MQ-03)",
    ),
    "mailbox-queue-count-off-by-one": (
        "src/dispatch/message_queue.rs",
        "        self.queue.lock().unwrap().len()\n",
        "        self.queue.lock().unwrap().len() + 1\n",
        "numberOfMessages reports one message too many (MBX-MQ-01, MBX-MQ-03)",
    ),
    "mailbox-close-forgets-queued-messages": (
        "src/actor/cell.rs",
        "        self.queue.clean_up(&*leftovers); // `cleanUp`; a producer that raced with the close drains for itself (Q-MBX-8)\n",
        "",
        "closing the mailbox forgets to move the queued user messages to dead letters, so they vanish silently (found by the dead-letter scenarios)",
    ),
    "mailbox-close-keeps-status-bits": (
        "src/dispatch/mailbox_status.rs",
        "            if self.update(status, Self::CLOSED) {",
        "            if self.update(status, status | Self::CLOSED) {",
        "closing ORs the Closed bit into the word instead of replacing it, so a closed mailbox keeps its Scheduled bit and suspend "
        "count and `isClosed` (== Closed) stops being true (MBX-ST-05)",
    ),
    "mailbox-suspended-cannot-be-scheduled": (
        "src/dispatch/mailbox_status.rs",
        "            if status & Self::SHOULD_SCHEDULE_MASK != Self::OPEN {",
        "            if status != Self::OPEN {",
        "a suspended mailbox refuses to be scheduled, so system messages sent to a suspended actor are never processed (MBX-ST-04)",
    ),
    "mailbox-suspended-ignores-system-hint": (
        "src/dispatch/mailbox_status.rs",
        "            _ => has_system_message_hint || has_system_messages(),",
        "            _ => has_system_messages(),",
        "a suspended mailbox ignores the system-message hint of the sender and waits for the queue to show the message (MBX-ST-04, ST-08)",
    ),
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
        "let mut budget = self.system.settings.throughput.max(1);",
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
        'match decision {',
        'match { let _ = decision; Directive::Stop } {',
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
        '        self.status.suspend(); // suspendNonRecursive\n        for k in self\n',
        '        for k in self\n',
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
    "bounded-queue-poll-deadline-overflows": (
        "src/util/bounded_blocking_queue.rs",
        "        let deadline = Instant::now().checked_add(timeout);\n        let mut guard = self.lock();\n        loop {\n            if let Some(element) = guard.backing.poll() {",
        "        let deadline = Some(Instant::now() + timeout);\n        let mut guard = self.lock();\n        loop {\n            if let Some(element) = guard.backing.poll() {",
        "poll(Long.MAX_VALUE, ...) adds the timeout to Instant::now() unchecked and panics instead of waiting without a deadline (BQ-C9, BQ-C10)",
    ),
    "bounded-queue-offer-deadline-overflows": (
        "src/util/bounded_blocking_queue.rs",
        "        let deadline = Instant::now().checked_add(timeout);\n        let mut guard = self.lock();\n        loop {\n            if guard.backing.len() < self.max_capacity {",
        "        let deadline = Some(Instant::now() + timeout);\n        let mut guard = self.lock();\n        loop {\n            if guard.backing.len() < self.max_capacity {",
        "offer(e, Long.MAX_VALUE, ...) on a full queue panics on the deadline arithmetic instead of waiting for room (BQ-C11)",
    ),
    "bounded-queue-no-deadline-means-expired": (
        "src/util/bounded_blocking_queue.rs",
        "        None => Some(None),\n",
        "        None => None,\n",
        "a timeout too large for an Instant is treated as already expired: poll returns null at once instead of waiting for the element (BQ-C9)",
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
        "Ok(total_ns / target.nanos())",
        "Ok(total_ns.div_euclid(target.nanos()))",
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
    # --- refinement 01, A-BUG-03: an exception escaping the actor factory or the decider ---
    "factory-failure-not-reported": (
        "src/actor/cell.rs",
        "let cause = self.creation_failure(&throwable_of(payload));",
        "let cause = self.creation_failure(&throwable_of(payload));\n                return drop(cause);",
        "an exception thrown by the actor factory is swallowed: the supervisor never hears about the failed child",
    ),
    "creation-failure-without-root-cause": (
        "src/actor/cell.rs",
        '"{}: exception during creation, root cause message: [{root_message}]",',
        '"{}: exception during creation [{root_message}]",',
        "the ActorInitializationException text loses the `root cause message: [...]` part",
    ),
    "decider-failure-ignored": (
        "src/actor/cell.rs",
        "self.fail_while_deciding(child, cause, throwable_of(payload));",
        "drop(payload);",
        "a decider that throws leaves the failed child suspended forever: its supervisor does not fail",
    ),
    "decider-failure-without-failed-message": (
        "src/actor/cell.rs",
        "self.fail(thrown, Some(failed), Some(child));",
        "self.fail(thrown, None, None);",
        "a supervisor that failed while deciding forgets the Failed message (preRestart sees none) and suspends the child again",
    ),
    "restart-factory-failure-drops-old-instance": (
        "src/actor/cell.rs",
        "self.st.lock().unwrap().actor = replaced;",
        "drop(replaced);",
        "when the factory throws during a restart the old instance is dropped, so the final postStop is lost",
    ),
    "post-restart-failure-names-wrong-class": (
        "src/actor/cell.rs",
        "self.path, original_cause.class",
        'self.path, "java.lang.Exception"',
        "the PostRestartException text names a fixed class instead of the failure that caused the restart",
    ),
    "hocon-render-shows-number-text": (
        "src/config/mod.rs",
        "Kind::Num(n) => n.render(),",
        "Kind::Num(n) => n.text().to_owned(),",
        "render prints the original text of a number (1.50, 010, 1e-7) instead of its value (1.5, 10, 1.0E-7)",
    ),
    "hocon-render-whole-double-stays-double": (
        "src/config/model.rs",
        "if whole as f64 == f {",
        "if false {",
        "render keeps a whole double as a double (1.0, 5.0E18) instead of the long Typesafe shows (1, 5000000000000000000)",
    ),
    "hocon-render-single-digit-double-not-two-digit": (
        "src/config/double_text.rs",
        "if !sci.contains('.') {",
        "if false {",
        "Double.toString shows Double.MIN_VALUE as 4.9E-324 (exact value rounded to two digits), not the shortest 5.0E-324",
    ),
    "hocon-number-lexer-accepts-plus-text": (
        "src/config/parse.rs",
        "None if text.contains('+') => {",
        "None if false => {",
        "a number-like run with a '+' that is not a number (1+2, 1e+) is accepted as text; Typesafe Config rejects the reserved character",
    ),
    "hocon-integer-overflow-is-number": (
        "src/config/model.rs",
        "NumRepr::Int(text.parse::<i64>().ok()?)",
        "text.parse::<i64>().map(NumRepr::Int).unwrap_or_else(|_| NumRepr::Float(text.parse().unwrap_or(0.0)))",
        "an integer literal that does not fit a long (99999999999999999999) becomes a number; Typesafe Config keeps it as text",
    ),
    "hocon-number-needs-fraction-digits": (
        "src/config/model.rs",
        "if int_digits == 0 && fraction_digits == 0 {",
        "if int_digits == 0 || fraction_digits == 0 {",
        "1. and 1.e5 are text; Double.parseDouble accepts a bare trailing point so Typesafe Config makes them numbers",
    ),
    "hocon-duration-converts-before-saturating": (
        "src/config/units.rs",
        "DurationNumber::Whole(n) => clamp(i128::from(n) * i128::from(unit_ns)),",
        "DurationNumber::Whole(n) => return Ok(clamp(i128::from(n) * i128::from(unit_ns) / i128::from(target.nanos()))),",
        "a duration saturates after converting to the target unit (106752 d in ms is 9223372800000) instead of at Long.MAX nanoseconds (9223372036854)",
    ),
    "hocon-duration-nan-is-an-error": (
        "src/config/units.rs",
        "\"NaN\" => return Some(f64::NAN),",
        "\"NaN\" => return None,",
        "a duration of NaN is rejected; Typesafe Config parses it as a double and the cast to long makes it 0",
    ),
    "hocon-duration-trailing-point-is-an-error": (
        "src/config/units.rs",
        "if mantissa_digits == 0 {",
        "if mantissa_digits < 2 {",
        "5 s written as `5.` or `.5` is rejected; Double.parseDouble accepts both",
    ),
    "hocon-size-negative-zero-is-an-error": (
        "src/config/units.rs",
        "if negative && !bytes.is_zero() {",
        "if negative {",
        "-0.5 B (BigDecimal truncates to 0) is rejected as a negative memory size",
    ),
    "hocon-size-range-message-shows-input": (
        "src/config/units.rs",
        "for a 64-bit long: '{exact}'",
        "for a 64-bit long: '{num}'",
        "the out-of-range message prints the input text instead of the exact product BigInteger (11150372599265311569558933316709551578284032)",
    ),
    "hocon-size-fraction-rounds-instead-of-truncating": (
        "src/config/exact_decimal.rs",
        "product.divide_by_ten();",
        "product.times_ten();",
        "the decimal places of a size are not dropped by truncation (8.2 MB is 8200000, 0.9 B is 0)",
    ),
    "hocon-size-exponent-sign-flipped": (
        "src/config/exact_decimal.rs",
        "let scale = i64::try_from(fraction.len()).ok()? - exponent;",
        "let scale = i64::try_from(fraction.len()).ok()? + exponent;",
        "1e3 B and 1e-3 kB scale the wrong way",
    ),
    "hocon-render-small-double-in-decimal-form": (
        "src/config/double_text.rs",
        "if (-3..7).contains(&exponent) {",
        "if (-30..70).contains(&exponent) {",
        "Double.toString threshold: 1e-7 and 1.23456789E7 must use the E notation",
    ),
    "settings-log-dead-letters-on-is-zero": (
        "src/config/settings.rs",
        '"on" | "true" => i32::MAX,',
        '"on" | "true" => 0,',
        "`pekko.log-dead-letters = on` logs nothing instead of every dead letter",
    ),
    "settings-log-dead-letters-case-sensitive": (
        "src/config/settings.rs",
        'match to_root_lower_case(&config.get_string("pekko.log-dead-letters")?).as_str() {',
        'match config.get_string("pekko.log-dead-letters")?.as_str() {',
        "`ON`, `Off` and `TRUE` are no longer read as the switch positions of log-dead-letters (Pekko lower-cases first)",
    ),
    "settings-suspend-infinite-case-sensitive": (
        "src/config/settings.rs",
        'match to_root_lower_case(&config.get_string(key)?).as_str() {',
        'match config.get_string(key)?.as_str() {',
        "`log-dead-letters-suspend-duration = Infinite` is read as a duration and fails to start",
    ),
    "settings-home-empty-is-some": (
        "src/config/settings.rs",
        '            "" => None,',
        '            "" => Some(String::new()),',
        "an empty pekko.home becomes Some(\"\") instead of None",
    ),
    "settings-coordinated-shutdown-combination-accepted": (
        "src/config/settings.rs",
        "if run_by_terminate && !terminate {",
        "if false && run_by_terminate && !terminate {",
        "run-by-actor-system-terminate=on with terminate-actor-system=off starts instead of failing fast",
    ),
    "settings-version-check-skipped": (
        "src/config/settings.rs",
        "        super::check_pekko_version(&config)?;",
        "",
        "a configuration written for another Pekko version starts without the ConfigurationException",
    ),
    "settings-check-valid-skipped": (
        "src/config/settings.rs",
        '        config.check_valid(&reference, &["pekko"])?;',
        "        let _ = &reference;",
        "a mistyped key (`pekko.log-dead-letters = true`) is no longer reported by the validation of the whole configuration",
    ),
    "settings-debug-lifecycle-reads-fsm": (
        "src/config/settings.rs",
        'let debug_lifecycle = config.get_boolean("pekko.actor.debug.lifecycle")?;',
        'let debug_lifecycle = config.get_boolean("pekko.actor.debug.fsm")?;',
        "the lifecycle debug flag follows the fsm key",
    ),
    "amend-ignores-the-registry": (
        "src/config/settings.rs",
        "&& is_registered(SLF4J_LOGGER)\n        {",
        "&& (is_registered(SLF4J_LOGGER) || true)\n        {",
        "use-slf4j adds the Slf4j logger even when no factory of that name is registered, so the system cannot start its loggers",
    ),
    "amend-keeps-the-default-logger": (
        "src/config/settings.rs",
        "filter(|l| l != DEFAULT_LOGGER)",
        "filter(|l| l != SLF4J_LOGGER)",
        "the default logger is not removed when the Slf4j logger takes over, so every event is logged twice",
    ),
    "amend-always-replaces-the-filter": (
        "src/config/settings.rs",
        "        configured_filter == SLF4J_LOGGING_FILTER || configured_filter != DEFAULT_LOGGING_FILTER;",
        "        configured_filter == SLF4J_LOGGING_FILTER;",
        "a custom logging filter is overwritten by the Slf4j one",
    ),
    "duration-does-not-saturate-before-converting": (
        "src/config/units.rs",
        "DurationNumber::Whole(n) => clamp(i128::from(n) * i128::from(unit_ns)),",
        "DurationNumber::Whole(n) => return Ok(clamp(i128::from(n) * i128::from(unit_ns) / i128::from(target.nanos()))),",
        "a duration larger than the Long nanoseconds is converted without saturating first, so `106752 d` is an error as milliseconds instead of 9223372036854",
    ),
    # --- refinement 01, A-BUG-01/02: text of ActorInitializationException (preStart) and PostRestartException (postRestart) ---
    "prestart-failure-fixed-text": (
        "src/actor/cell.rs",
        "let cause = self.creation_failure(&root_cause);",
        'let cause = Throwable::new("org.apache.pekko.actor.ActorInitializationException", Some("exception during creation"));\n            let _ = root_cause;',
        "a preStart failure reports a fixed 'exception during creation' text without the path and the root cause message",
    ),
    "post-restart-failure-drops-new-instance": (
        "src/actor/cell.rs",
        "// the new instance stays in place (Pekko keeps `_actor`): it gets its `postStop` when the actor stops",
        "self.st.lock().unwrap().actor = None;",
        "after a postRestart failure the new instance is dropped, so its final postStop is lost",
    ),
    # --- Q-CFG-30: shortest digits of a rendered double, exact ties go to the even digit ---
    "double-tie-breaks-to-odd": (
        "src/config/double_text.rs",
        "let even = if lower % 2 == 0 { lower } else { upper };",
        "let even = if lower % 2 == 1 { lower } else { upper };",
        "an exact tie between two shortest candidates takes the odd last digit (what JDK 17 never does and JDK 19+ forbids)",
    ),
    "double-tie-breaks-to-larger": (
        "src/config/double_text.rs",
        "let even = if lower % 2 == 0 { lower } else { upper };",
        "let even = upper;",
        "an exact tie takes the larger candidate, which is what Display of Rust does (357 of 380 849 random doubles)",
    ),
    "double-tie-ignored": (
        "src/config/double_text.rs",
        "if let Some(even) = even_digits_of_exact_tie(d.abs(), &digits, exponent) {",
        "if let Some(even) = even_digits_of_exact_tie(d.abs(), &digits, exponent).filter(|_| false) {",
        "the tie rule is never applied: the digits are exactly what the standard library gives",
    ),
    "double-tie-needs-no-equal-distance": (
        "src/config/double_text.rs",
        "    if below != above {\n        return None;\n    }\n",
        "",
        "any value whose exact expansion has one digit more than the shortest is treated as a tie, so a closest candidate with an odd digit is replaced by a farther even one",
    ),
    "double-tie-result-not-checked": (
        "src/config/double_text.rs",
        "same_power && identifies_value)",
        "same_power)",
        "the even candidate replaces the digits even when it no longer identifies the double (the lower side of a power of two is narrower)",
    ),
    # --- slice P09 (AC-CFG-016): FactoryRegistry, the by-name lookup of DynamicAccess ---
    "registry-lookup-ignores-case": (
        "src/config/dynamic_access.rs",
        "self.factories.get(name).ok_or_else(|| ClassNotFoundError {",
        "self.factories.iter().find(|(key, _)| key.eq_ignore_ascii_case(name)).map(|(_, factory)| factory).ok_or_else(|| ClassNotFoundError {",
        "a class name is matched without regard to case, so `unboundedmailbox` finds `UnboundedMailbox` (Class.forName is exact)",
    ),
    "registry-lookup-trims-the-name": (
        "src/config/dynamic_access.rs",
        "self.factories.get(name).ok_or_else(|| ClassNotFoundError {",
        "self.factories.get(name.trim()).ok_or_else(|| ClassNotFoundError {",
        "a name with surrounding spaces finds the class it spells (Class.forName does not trim)",
    ),
    "registry-contains-ignores-case": (
        "src/config/dynamic_access.rs",
        "self.factories.contains_key(name)",
        "self.factories.keys().any(|key| key.eq_ignore_ascii_case(name))",
        "classIsOnClasspath answers true for a name that differs only in case",
    ),
    "registry-contains-trims-the-name": (
        "src/config/dynamic_access.rs",
        "self.factories.contains_key(name)",
        "self.factories.contains_key(name.trim())",
        "classIsOnClasspath answers true for a name with a trailing space",
    ),
    "class-not-found-drops-the-name": (
        "src/config/dynamic_access.rs",
        "            name: name.to_owned(),\n",
        "            name: String::new(),\n",
        "the ClassNotFoundException does not carry the class name that was asked for",
    ),
    "class-not-found-is-another-java-class": (
        "src/config/dynamic_access.rs",
        '        "java.lang.ClassNotFoundException"\n    }',
        '        "java.lang.NoClassDefFoundError"\n    }',
        "the failure for an unknown name is reported as a NoClassDefFoundError instead of a ClassNotFoundException",
    ),
    # --- slice P09 (AC-CFG-008): ProviderSelection and pekko.actor.provider ---
    "provider-remote-short-name-ignored": (
        "src/config/provider_selection.rs",
        '            "remote" | REMOTE_ACTOR_REF_PROVIDER => ProviderSelection::Remote,\n',
        '            REMOTE_ACTOR_REF_PROVIDER => ProviderSelection::Remote,\n',
        "`remote` is not recognised as the short name of the remote provider and becomes a custom class name",
    ),
    "provider-cluster-class-name-ignored": (
        "src/config/provider_selection.rs",
        '"cluster" | CLUSTER_ACTOR_REF_PROVIDER => ProviderSelection::Cluster,',
        '"cluster" => ProviderSelection::Cluster,',
        "the class name of the cluster provider is a custom provider instead of the same selection as `cluster`",
    ),
    "provider-remote-has-cluster": (
        "src/config/provider_selection.rs",
        'matches!(self, ProviderSelection::Cluster)',
        'matches!(self, ProviderSelection::Cluster | ProviderSelection::Remote)',
        "the remote provider reports a cluster",
    ),
    "provider-local-class-name-is-local": (
        "src/config/provider_selection.rs",
        '"local" => ProviderSelection::Local,',
        '"local" | LOCAL_ACTOR_REF_PROVIDER => ProviderSelection::Local,',
        "the class name of the local provider selects Local, but Pekko 2.0.0 takes it as a custom class name",
    ),
    "provider-selection-trims-the-value": (
        "src/config/provider_selection.rs",
        '        match provider {\n',
        '        match provider.trim() {\n',
        "` local` or `cluster ` select Local and Cluster (Pekko compares the exact string)",
    ),
    "provider-selection-ignores-case": (
        "src/config/provider_selection.rs",
        '        match provider {\n',
        '        match provider.to_ascii_lowercase().as_str() {\n',
        "`LOCAL` and `Remote` select Local and Remote (Pekko compares the exact string)",
    ),
    "provider-custom-identifier-changed": (
        "src/config/provider_selection.rs",
        'ProviderSelection::Custom(_) => "custom",',
        'ProviderSelection::Custom(_) => "other",',
        "a custom provider has another identifier than `custom`",
    ),
    "settings-provider-class-is-the-identifier": (
        "src/config/settings.rs",
        '        self.provider_selection.fqcn()\n',
        '        self.provider_selection.identifier()\n',
        "`Settings.ProviderClass` is the short identifier instead of the class name",
    ),
    "settings-has-cluster-always-false": (
        "src/config/settings.rs",
        '        self.provider_selection.has_cluster()\n',
        '        false\n',
        "`Settings.HasCluster` is false for the cluster provider",
    ),
    # --- slice P09 (AC-CFG-015): Setup, ActorSystemSetup and BootstrapSetup ---
    "setup-replacement-is-appended-as-a-second-entry": (
        "src/config/setup.rs",
        'Some(existing) => *existing = entry,',
        'Some(_) => self.entries.push(entry),',
        "a second setup of the same type is added next to the first one, so `get` keeps returning the first",
    ),
    "setup-replacement-moves-to-the-end": (
        "src/config/setup.rs",
        'Some(existing) => *existing = entry,',
        'Some(_) => {\n                self.entries.retain(|e| e.type_id != entry.type_id);\n                self.entries.push(entry);\n            }',
        "a replaced setup loses its position in the list of names",
    ),
    "setup-display-sorts-the-names": (
        "src/config/setup.rs",
        '        let names: Vec<&str> = self.entries.iter().map(|e| e.class_name).collect();\n',
        '        let mut names: Vec<&str> = self.entries.iter().map(|e| e.class_name).collect();\n        names.sort_unstable();\n',
        "the class names are listed in alphabetical order instead of insertion order",
    ),
    "setup-display-separator-has-a-space": (
        "src/config/setup.rs",
        'names.join(",")',
        'names.join(", ")',
        "the class names are separated by a comma and a space",
    ),
    "setup-and-keeps-the-first-of-a-type": (
        "src/config/setup.rs",
        'ActorSystemSetup::empty().with_setup(self).with_setup(other)',
        'ActorSystemSetup::empty().with_setup(other).with_setup(self)',
        "`a and b` keeps `a` when both have the same type (Pekko keeps the last)",
    ),
    "bootstrap-with-config-drops-the-provider": (
        "src/config/setup.rs",
        '            config: Some(config),\n            ..self',
        '            config: Some(config),\n            actor_ref_provider: None',
        "`withConfig` forgets the actor-ref provider",
    ),
    "bootstrap-with-provider-drops-the-config": (
        "src/config/setup.rs",
        '            actor_ref_provider: Some(provider),\n            ..self',
        '            actor_ref_provider: Some(provider),\n            config: None',
        "`withActorRefProvider` forgets the configuration",
    ),
    "bootstrap-class-name-is-not-pekkos": (
        "src/config/setup.rs",
        '"org.apache.pekko.actor.BootstrapSetup"',
        '"BootstrapSetup"',
        "the BootstrapSetup is listed in `toString` under another name than its Java class",
    ),
    "settings-ignore-the-provider-of-the-setup": (
        "src/config/settings.rs",
        'Some(selection) => selection.identifier().to_owned(),',
        'Some(_) => config.get_string("pekko.actor.provider")?,',
        "a BootstrapSetup.actorRefProvider does not take priority over pekko.actor.provider",
    ),
    "settings-custom-provider-of-the-setup-keeps-its-class": (
        "src/config/settings.rs",
        'Some(selection) => selection.identifier().to_owned(),',
        'Some(selection) => selection.fqcn().to_owned(),',
        "a custom provider of the setup keeps its class name (Pekko 2.0.0 reads it through its identifier, so it becomes `custom`)",
    ),
    # --- slice P10 (AC-CFG-013): Deploy, Scope and the router slot ---
    "deploy-fallback-prefers-the-other-dispatcher": (
        "src/actor/deploy.rs",
        "dispatcher: if self.dispatcher == NO_DISPATCHER_GIVEN {\n                other.dispatcher.clone()\n            } else {\n                self.dispatcher.clone()\n            },",
        "dispatcher: other.dispatcher.clone(),",
        "withFallback takes the dispatcher of the other deploy even when the receiver names one",
    ),
    "deploy-fallback-never-takes-the-other-mailbox": (
        "src/actor/deploy.rs",
        "mailbox: if self.mailbox == NO_MAILBOX_GIVEN {\n                other.mailbox.clone()\n            } else {\n                self.mailbox.clone()\n            },",
        "mailbox: self.mailbox.clone(),",
        "withFallback keeps an empty mailbox instead of taking the one of the other deploy",
    ),
    "deploy-fallback-takes-the-path-of-the-other": (
        "src/actor/deploy.rs",
        "path: self.path.clone(),\n            config: self.config.with_fallback(&other.config),",
        "path: other.path.clone(),\n            config: self.config.with_fallback(&other.config),",
        "withFallback takes the path of the other deploy instead of keeping its own",
    ),
    "deploy-fallback-keeps-the-tags": (
        "src/actor/deploy.rs",
        "            },\n            tags: BTreeSet::new(),\n        }",
        "            },\n            tags: self.tags.clone(),\n        }",
        "withFallback carries the tags of the receiver, which Pekko drops",
    ),
    "local-scope-lets-the-other-win": (
        "src/actor/deploy.rs",
        "Scope::Local => Scope::Local,",
        "Scope::Local => other.clone(),",
        "LocalScope.withFallback returns the other scope instead of itself",
    ),
    "no-scope-given-keeps-itself": (
        "src/actor/deploy.rs",
        "Scope::NoScopeGiven => other.clone(),",
        "Scope::NoScopeGiven => Scope::NoScopeGiven,",
        "NoScopeGiven.withFallback returns itself instead of the other scope",
    ),
    "no-router-keeps-itself": (
        "src/actor/deploy.rs",
        "RouterConfig::NoRouter => other.clone(),",
        "RouterConfig::NoRouter => RouterConfig::NoRouter,",
        "NoRouter.withFallback returns itself instead of the other router",
    ),
    "router-takes-the-other-router": (
        "src/actor/deploy.rs",
        "RouterConfig::Opaque(_) => self.clone(),",
        "RouterConfig::Opaque(_) => other.clone(),",
        "a router.withFallback returns the other router instead of itself",
    ),
    "deploy-equality-compares-the-text-of-the-configuration": (
        "src/actor/deploy.rs",
        "self.config.same_value(&other.config)",
        "self.config == other.config",
        "two deploys whose configuration was read from different text are not equal",
    ),
    "config-equality-compares-numbers-by-their-text": (
        "src/config/mod.rs",
        "whole(*f) && *f as i64 == *x",
        "false",
        "the whole number 1 and 1.0 are not the same configuration value",
    ),
    "config-equality-ignores-the-extra-keys-of-the-other": (
        "src/config/mod.rs",
        "    a.len() == b.len()\n        && a.iter()",
        "    a.iter()",
        "a configuration equals another one that has more keys",
    ),
    # --- slice P10 (AC-CFG-012.2): Deployer.deploy and Deployer.lookup ---
    "deployer-lookup-keeps-the-guardian": (
        "src/actor/deployer.rs",
        "self.lookup(path.elements().into_iter().skip(1))",
        "self.lookup(path.elements())",
        "lookup of an actor path does not drop the guardian (user, system), so /user/a is looked up as user/a",
    ),
    "deployer-deploy-keeps-the-first-element": (
        "src/actor/deployer.rs",
        "parts.get(1..).unwrap_or_default()",
        "parts.get(0..).unwrap_or_default()",
        "deploy does not drop the first element of the path, so /a/b is stored as (empty)/a/b",
    ),
    "deployer-accepts-an-empty-element": (
        "src/actor/deployer.rs",
        "            if element.is_empty() {\n                return Err(InvalidActorNameError::new(",
        "            if element.is_empty() && false {\n                return Err(InvalidActorNameError::new(",
        "an empty element of a deployment path is not rejected",
    ),
    "deployer-skips-the-validation-of-the-elements": (
        "src/actor/deployer.rs",
        "            validate_path_element(element, Some(&path))?;\n",
        "",
        "an element with an illegal character is accepted",
    ),
    "deployer-validation-message-omits-the-path": (
        "src/actor/deployer.rs",
        "validate_path_element(element, Some(&path))?;",
        "validate_path_element(element, None)?;",
        "the message of an invalid element does not name the path of the deployment",
    ),
    "deployer-split-keeps-the-trailing-empty-parts": (
        "src/actor/deployer.rs",
        "while parts.last().is_some_and(|part| part.is_empty()) {",
        "while false && parts.last().is_some_and(|part| part.is_empty()) {",
        "split keeps the empty parts at the end, so a path ending in / has an empty element",
    ),
    "deployer-deploy-forgets-the-previous-deployments": (
        "src/actor/deployer.rs",
        "let updated = Arc::new(snapshot.insert(elements, deploy.clone())?);",
        "let updated = Arc::new(WildcardIndex::new().insert(elements, deploy.clone())?);",
        "every deploy starts from an empty index",
    ),
    "deployer-deploy-swaps-a-stale-snapshot": (
        "src/actor/deployer.rs",
        "if Arc::ptr_eq(&current, &snapshot) {",
        "if true {",
        "the new index replaces the current one even when another deploy swapped in meanwhile, so that deploy is lost (killed by a deterministic unit test, see UNIT_MUTANTS)",
    ),
    "deployer-deploy-stores-nothing": (
        "src/actor/deployer.rs",
        "                *current = updated;\n",
        "                drop(updated);\n",
        "a valid deployment is reported as done but never stored",
    ),
    "deployer-empty-element-message-is-wrong": (
        "src/actor/deployer.rs",
        '"Actor name in deployment [{path}] must not be empty"',
        '"Actor name in deployment [{path}] must be empty"',
        "the message of an empty element of a deployment path is wrong",
    ),
    # --- slice P10 (AC-CFG-012.1): the Deployer built from pekko.actor.deployment ---
    "deployer-entry-ignores-the-default-section": (
        "src/actor/deployer.rs",
        "let deployment = entry.with_fallback(default);",
        "let deployment = entry.clone();",
        "an entry is read without the default section as its fallback",
    ),
    "deployer-default-wins-over-the-entry": (
        "src/actor/deployer.rs",
        "let deployment = entry.with_fallback(default);",
        "let deployment = default.with_fallback(entry);",
        "the default section overrides the values of the entry",
    ),
    "deployer-router-is-never-named": (
        "src/actor/deployer.rs",
        "name => RouterConfig::opaque(name),",
        "_ => RouterConfig::NoRouter,",
        "a router that is not from-code is dropped instead of kept by its configured name",
    ),
    "deployer-from-code-is-a-named-router": (
        "src/actor/deployer.rs",
        '"from-code" => RouterConfig::NoRouter,',
        '"from-code" => RouterConfig::opaque("from-code"),',
        "from-code is kept as a router instead of meaning no router",
    ),
    "deployer-dispatcher-ignores-the-default-section": (
        "src/actor/deployer.rs",
        'let dispatcher = deployment.get_string("dispatcher")?;',
        'let dispatcher = entry.get_string("dispatcher")?;',
        "the dispatcher of an entry is read without the default section",
    ),
    "deployer-mailbox-ignores-the-default-section": (
        "src/actor/deployer.rs",
        'let mailbox = deployment.get_string("mailbox")?;',
        'let mailbox = entry.get_string("mailbox")?;',
        "the mailbox of an entry is read without the default section",
    ),
    "deployer-dispatcher-and-mailbox-are-swapped": (
        "src/actor/deployer.rs",
        "            &dispatcher,\n            &mailbox,\n",
        "            &mailbox,\n            &dispatcher,\n",
        "the dispatcher is stored as the mailbox and the mailbox as the dispatcher",
    ),
    "deployer-entry-scope-is-local": (
        "src/actor/deployer.rs",
        "            Scope::NoScopeGiven,\n            &dispatcher,",
        "            Scope::Local,\n            &dispatcher,",
        "the deployments read from the configuration have the local scope",
    ),
    "deployer-configuration-errors-are-swallowed": (
        "src/actor/deployer.rs",
        "            .collect::<Result<Vec<_>, _>>()?;",
        "            .filter_map(Result::ok)\n            .collect::<Vec<_>>();",
        "an entry with a value of the wrong type is skipped instead of failing the construction",
    ),
    "deployer-rejected-paths-are-swallowed": (
        "src/actor/deployer.rs",
        "            deployer.deploy(deploy)?;",
        "            let _ = deployer.deploy(deploy);",
        "an entry with an invalid path is skipped instead of failing the construction",
    ),
    "deployer-entries-go-in-reverse-alphabetical-order": (
        "src/actor/deployer.rs",
        "            .object_entries()\n            .into_iter()\n",
        "            .object_entries()\n            .into_iter()\n            .rev()\n",
        "of several keys that give one path the first in alphabetical order wins instead of the last (the temporary policy)",
    ),
}

# Mutants of the Python tools (tools/parity): killed by the unit tests of the tool, no replay needed.
# name -> (file in tools/parity, old, new, why)
TOOL_MUTANTS = {
    "na-q-cfg-30-ignores-the-double": (
        "na_q_cfg_30.py",
        "same_double = oracle.double() is not None and oracle.double() == rakka.double()  # (a)",
        "same_double = True  # (a)",
        "condition a: a different double one ulp away is accepted as the same value",
    ),
    "na-q-cfg-30-rakka-need-not-be-the-jdk-19-text": (
        "na_q_cfg_30.py",
        "same_double and _is_jdk19_result(rakka) and not _is_jdk19_result(oracle) and same_layout  # (b) (c)",
        "same_double and not _is_jdk19_result(oracle) and same_layout  # (b) (c)",
        "condition b: a tie that rakka resolves to the odd digit is hidden as a JDK 17 difference",
    ),
    "na-q-cfg-30-oracle-may-be-the-jdk-19-text": (
        "na_q_cfg_30.py",
        "same_double and _is_jdk19_result(rakka) and not _is_jdk19_result(oracle) and same_layout  # (b) (c)",
        "same_double and _is_jdk19_result(rakka) and same_layout  # (b) (c)",
        "condition c: an oracle that already prints the JDK 19+ text is accepted (any difference elsewhere in the number passes)",
    ),
    "na-q-cfg-30-ignores-the-notation": (
        "na_q_cfg_30.py",
        "same_layout = oracle.scientific == rakka.scientific and oracle.exponent == rakka.exponent  # (d)",
        "same_layout = oracle.exponent == rakka.exponent  # (d)",
        "condition d: decimal notation against E notation is accepted",
    ),
    "na-q-cfg-30-ignores-the-exponent": (
        "na_q_cfg_30.py",
        "same_layout = oracle.scientific == rakka.scientific and oracle.exponent == rakka.exponent  # (d)",
        "same_layout = oracle.scientific == rakka.scientific  # (d)",
        "condition d: digits on another power of ten are accepted",
    ),
    "na-q-cfg-30-ignores-the-rest-of-the-document": (
        "na_q_cfg_30.py",
        "if oracle_pieces != rakka_pieces or len(oracle_numbers) != len(rakka_numbers):  # (e) the rest of the output",
        "if len(oracle_numbers) != len(rakka_numbers):  # (e) the rest of the output",
        "condition e: a difference outside the numbers is accepted",
    ),
    "na-q-cfg-30-accepts-several-different-numbers": (
        "na_q_cfg_30.py",
        "    if len(different) != 1:\n        return False\n",
        "    if not different:\n        return False\n",
        "condition e: two numbers that differ are looked at only for the first",
    ),
}


def sh(cmd, timeout=None, **kw):
    return subprocess.run(cmd, stdin=subprocess.DEVNULL, capture_output=True, text=True, timeout=timeout, **kw)


REPLAY_TIMEOUT = 900   # seconds; a hung replay is reported as TIMEOUT, never as "detected"
BUILD_TIMEOUT = 900
SUMMARY = re.compile(r"(\d+)/(\d+) cases passed")

# Outcome classes. Only DETECTED counts as a caught mutant; everything else is reported separately.
BUDGET_EXHAUSTED = "BUDGET_EXHAUSTED"    # the run's budget ended while waiting for the replay lock: no replay ran
BUDGET_EXHAUSTED_LINE = "BUDGET_EXHAUSTED: the run budget ended while waiting for the replay lock"
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


# Mutants that the replay cannot see because they need an interleaving of two threads: they are killed by a deterministic
# unit test (no threads, no sleeps) that the harness runs instead of the replay.
UNIT_MUTANTS = {"deployer-deploy-swaps-a-stale-snapshot": ("actor::deployer", "a_deployment_that_loses_the_swap_is_retried_on_the_newer_index_and_nothing_is_lost")}


def run_unit_mutant(scratch, name, work):
    """Runs the unit tests of the module on the mutated copy. DETECTED needs the expected test to fail (a compile error or
    some other failing test is not enough) and the same tests to pass on the unmutated tree (the control)."""
    module, expected_test = UNIT_MUTANTS[name]
    def cargo_test(manifest, target):
        return sh(["cargo", "test", "--manifest-path", manifest, "--target-dir", target, "--lib", module], timeout=BUILD_TIMEOUT)
    try:
        control = cargo_test(os.path.join(ROOT, "crates", "rakka-actor", "Cargo.toml"), os.path.join(scratch, "target-control-" + name))
        r = cargo_test(os.path.join(work, "Cargo.toml"), os.path.join(scratch, "target-" + name))
    except subprocess.TimeoutExpired:
        return TIMEOUT, None, None, "cargo test"
    if control.returncode != 0:
        return HARNESS_ERROR, None, None, "the unit tests fail on the unmutated tree"
    summary = next((l for l in r.stdout.splitlines() if l.startswith("test result:")), "(no test result)")
    if "could not compile" in r.stderr:
        return BUILD_ERROR, None, None, r.stderr[-600:]
    if r.returncode == 0:
        return UNDETECTED, None, None, summary
    if "test result: FAILED" in r.stdout and f"{expected_test} ... FAILED" in r.stdout:
        return DETECTED, None, None, summary
    return HARNESS_ERROR, None, None, "failed, but not the expected test: " + summary


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
    if name in UNIT_MUTANTS:
        return run_unit_mutant(scratch, name, work)
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
    if FAST_KILL:
        try:
            kill = fast_kill(binary, name)
        except replay_lib.ReplayLockTimeout:
            return BUDGET_EXHAUSTED, None, None, BUDGET_EXHAUSTED_LINE
        if kill:   # a divergence the full replay would also classify as applicable, demonstrated on the real tool for one case
            publish_fast_kill_report(name, kill)
            return DETECTED, None, None, (f"fast-kill: case {kill.case} confirmed by a single-case replay; "
                                          "the unmutated build matches Pekko on it, the mutant does not")
    return full_replay_verdict(binary, name)


def full_replay_verdict(binary, name):
    """The verdict of a mutant from the full recorded replay: -> (outcome, passed, total, line)."""
    try:
        r = replay(binary)
    except subprocess.TimeoutExpired:
        return TIMEOUT, None, None, "rust-anything replay"
    except replay_lib.ReplayLockTimeout:
        return BUDGET_EXHAUSTED, None, None, BUDGET_EXHAUSTED_LINE
    if r.report_text is not None:   # the report of THIS replay (read under the checkout lock), never the shared file as it is now
        publish_mutant_report(name, f"full replay ({r.total} cases)", r.report_text)
    return classify(r, binary)


FAST_KILL = True
FAST_KILL_CANDIDATES = 8


def known_killers(name):
    """Cases the committed report of an earlier full replay of this mutant listed as divergent, minus the NOT_APPLICABLE ones."""
    try:
        with open(os.path.join(REPORTS, f"replay-report-{name}.md"), encoding="utf-8") as fh:
            listed = re.findall(r"(?m)^### (\S+)\s*$", fh.read())
    except OSError:
        return []
    not_applicable = replay_lib.load_not_applicable()
    return [c for c in listed if c not in not_applicable][:FAST_KILL_CANDIDATES]


_control_binary = None


def control_binary_for_fast_kill():
    """The unmutated parity binary of the current tree (built once per process), or None when it cannot be built."""
    global _control_binary
    if _control_binary is None:
        path = os.path.join(ROOT, "target", PARITY)
        try:
            with replay_lib.build_lock():   # build and publish are one step: no other build can swap target/ in between
                built = sh(["cargo", "build", "--release", "-p", "rakka-actor", "--example", "classic-parity"], timeout=BUILD_TIMEOUT, cwd=ROOT)
                # the control replays an immutable copy, never the shared target/ path that the next build overwrites
                _control_binary = replay_lib.publish_binary(path) if built.returncode == 0 and os.path.exists(path) else ""
        except subprocess.TimeoutExpired:
            _control_binary = ""
    return _control_binary or None


FastKill = collections.namedtuple("FastKill", "case report_text reason")


def fast_kill(binary, name):
    """The first known killer case whose divergence is DEMONSTRATED: the unmutated build agrees with the recorded Pekko output on it, the
    mutant does not (replay_lib.diverges_only_in_mutant), AND the real tool, replaying that single case, fails it as an applicable case
    (replay_lib.confirm_applicable_divergence), i.e. the full replay would classify it the same way. Else None: the full replay decides.
    Never a verdict of 'undetected'."""
    control = control_binary_for_fast_kill()
    if control is None:
        return None
    for case in known_killers(name):
        if not replay_lib.diverges_only_in_mutant(binary, control, case):
            continue
        confirmation = replay_lib.confirm_applicable_divergence(binary, case)
        if confirmation.confirmed:
            return FastKill(case, confirmation.report_text, "the unmutated build matches Pekko on it")
    return None


def publish_mutant_report(name, verdict_path, report_text):
    """The per-mutant report always starts with the path that produced the verdict, so that the evidence cannot be mistaken for another one."""
    with open(os.path.join(REPORTS, f"replay-report-{name}.md"), "w", encoding="utf-8") as report_file:
        report_file.write(f"Verdict path: {verdict_path}\n\n{report_text}")


def publish_fast_kill_report(name, kill):
    publish_mutant_report(name, f"fast-kill: case {kill.case} confirmed by a single-case replay (`rust-anything app replay --case`); "
                                f"{kill.reason}; no full replay ran", kill.report_text)


def run_tool_mutant(scratch, name, rel, old, new, why):
    """A mutant of tools/parity itself: a scratch copy is patched and the unit tests of the tools must fail."""
    work = os.path.join(scratch, name)
    shutil.copytree(TOOLS, work, ignore=shutil.ignore_patterns("__pycache__"))
    path = os.path.join(work, rel)
    src = open(path).read()
    if src.count(old) != 1:
        return PATCH_ERROR, None, None, f"pattern occurs {src.count(old)} times in {rel}"
    patched = src.replace(old, new)
    open(path, "w").write(patched)
    print("".join(difflib.unified_diff(src.splitlines(True), patched.splitlines(True), f"a/tools/parity/{rel}", f"b/tools/parity/{rel}")))
    try:
        r = sh([sys.executable, "-I", "-m", "unittest", "discover", "-s", work, "-p", "test_*.py"], timeout=300)
    except subprocess.TimeoutExpired:
        return TIMEOUT, None, None, "unit tests of the tools"
    tail = (r.stderr.strip().splitlines() or ["(no output)"])[-1]
    if r.returncode == 0:
        return UNDETECTED, None, None, tail
    if "Ran 0 tests" in r.stderr or "ModuleNotFoundError" in r.stderr or "SyntaxError" in r.stderr:
        return HARNESS_ERROR, None, None, r.stderr[-300:]
    return DETECTED, None, None, tail


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
    for name, (rel, old, new, _why) in TOOL_MUTANTS.items():
        n = open(os.path.join(TOOLS, rel)).read().count(old)
        if n != 1 or old == new:
            bad += 1
            print(f"STALE {name}: {n} occurrences in tools/parity/{rel}")
    print(f"{len(MUTANTS) + len(SELFTEST) + len(TOOL_MUTANTS)} mutant patterns checked, {bad} stale")
    sys.exit(2 if bad else 0)


def control_exit_code(control_ok, replayed_cases):
    """0 the control is clean; 1 it replayed and diverged from Pekko (a real regression); 2 it could not run (build failure, timeout,
    harness error: `replayed_cases` is None), which says nothing about the code."""
    if control_ok:
        return 0
    return 1 if replayed_cases is not None else 2


def final_exit_code(counts, sizes, control_ok, replayed_cases, budget_exhausted):
    """The exit code of a whole run: the budget code when the run's budget ended while waiting for the replay lock (nothing can be concluded),
    2 for an invalid experiment, 1 for a surviving mutant or a diverging control, 0 otherwise."""
    if budget_exhausted:
        return replay_lib.EXIT_BUDGET_EXHAUSTED
    errors = counts[PATCH_ERROR] + counts[BUILD_ERROR] + counts[HARNESS_ERROR] + counts[TIMEOUT]
    if errors or len(sizes) > 1 or control_exit_code(control_ok, replayed_cases) == 2:
        return 2
    return 0 if counts[UNDETECTED] == 0 and control_ok else 1


SCRATCH_OWNER_MARKER = ".owner-pid"   # the same name as in tools/gates/mutant_runner.py: a later run removes the scratch of a dead owner


def make_scratch_directory(parent=None):
    """A scratch workspace that records its owner, so that a later run can tell a directory left behind by a killed process (SIGKILL cannot
    be handled) from one a live process is using."""
    path = tempfile.mkdtemp(prefix="pekko-mutants-", dir=parent)
    with open(os.path.join(path, SCRATCH_OWNER_MARKER), "w") as owner_file:
        owner_file.write(str(os.getpid()))
    return path


def run_control_replay():
    """Replays the unmodified crate: builds the parity binary of the CURRENT tree first (a leftover binary would certify old code).
    -> (control_ok, passed, total, line, replay result or None)."""
    control_binary = control_binary_for_fast_kill()
    if control_binary is None:
        return False, None, None, "could not build the unmodified parity binary", None
    try:
        control_replay = replay(control_binary)
    except subprocess.TimeoutExpired:
        return False, None, None, "TIMEOUT", None
    except replay_lib.ReplayLockTimeout:
        return False, None, None, BUDGET_EXHAUSTED_LINE, None
    control_outcome, control_passed, control_total, control_line = classify(control_replay, control_binary)
    return control_outcome == UNDETECTED, control_passed, control_total, control_line, control_replay


def main():
    args = sys.argv[1:]
    if "--check-patterns" in args:
        check_patterns()
    global FAST_KILL
    selftest = "--selftest" in args
    skip_control, control_only = "--skip-control" in args, "--control-only" in args
    FAST_KILL = "--no-fast-kill" not in args
    result_json = next((a.split("=", 1)[1] for a in args if a.startswith("--result-json=")), None)
    only = [a for a in args if not a.startswith("--")]  # optional: run just these mutants (the control still runs unless --skip-control)
    mutants = dict(SELFTEST) if selftest else {**MUTANTS, **TOOL_MUTANTS}
    signal.signal(signal.SIGTERM, lambda *_: sys.exit(143))   # cancellation runs the `finally` below: scratch removed, children killed
    c_pass = c_total = None
    for n in only:
        assert n in mutants, f"unknown mutant {n}"
    tool_only = bool(only) and all(n in TOOL_MUTANTS for n in only)
    os.makedirs(REPORTS, exist_ok=True)
    scratch = make_scratch_directory()
    results = {}
    try:
        for name, (rel, old, new, why) in mutants.items():
            if control_only or (only and name not in only):
                continue
            print(f"=== mutant {name}: {why}", flush=True)
            res = (run_tool_mutant if name in TOOL_MUTANTS else run_mutant)(scratch, name, rel, old, new, why)
            results[name] = res
            print(f"{res[0]}  {res[3]}\n", flush=True)
        c = None
        if skip_control:   # the caller (tools/gates) runs the control once for a whole batch
            control_ok, c_pass, c_total, c_line = True, None, None, "skipped (--skip-control)"
        elif tool_only:   # the tool mutants need no replay: the control would only repeat the last full run
            control_ok, c_pass, c_total, c_line = True, None, None, "skipped (tool mutants only)"
        else:
            control_ok, c_pass, c_total, c_line, c = run_control_replay()
        print(f"=== control (unmodified rakka-actor): exit={c.returncode if c is not None and c_pass is not None else '?'}  {c_line}", flush=True)
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
    if result_json:
        with open(result_json, "w") as fh:
            json.dump({"mutants": {n: list(r) for n, r in results.items()}, "control": None if skip_control else control_ok}, fh)
    if not selftest and not only and not skip_control and not control_only:
        with open(os.path.join(REPORTS, "SUMMARY.md"), "w") as fh:
            fh.write(table + f"\n\noutcomes: {counts}\n")
    budget_exhausted = any(result[0] == BUDGET_EXHAUSTED for result in results.values()) or c_line == BUDGET_EXHAUSTED_LINE
    sys.exit(final_exit_code(counts, sizes, control_ok, c_pass, budget_exhausted))


if __name__ == "__main__":
    main()
