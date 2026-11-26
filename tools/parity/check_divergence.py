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
import atexit, collections, difflib, hashlib, json, os, re, shutil, signal, subprocess, sys, tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import replay_lib
import clean_replay_evidence

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
CRATE = os.path.join(ROOT, "crates", "rakka-actor")
TOOLS = os.path.dirname(os.path.abspath(__file__))
PARITY = os.path.join("release", "examples", "classic-parity")
REPORTS = os.path.join(ROOT, "rust", "divergence-reports")

# name -> (file, old, new, why it is a realistic bug)
MUTANTS = {
    "system-queue-drains-newest-first": (
        "src/dispatch/system_queue.rs",
        "            State::Open(messages) => std::mem::take(messages),",
        "            State::Open(messages) => {\n                let mut newest_first = std::mem::take(messages);\n                newest_first.reverse();\n                newest_first\n            }",
        "the system messages are handed out newest first, so a Resume can overtake the Suspend sent before it (MBX-SQ-01)",
    ),
    "system-queue-reopens-after-clean-up": (
        "src/dispatch/system_queue.rs",
        "        match std::mem::replace(&mut *self.state.lock().unwrap(), State::Closed) {",
        "        match std::mem::replace(&mut *self.state.lock().unwrap(), State::Open(Vec::new())) {",
        "cleanUp forgets the closing sentinel: a system message sent after the actor terminated is stored in a mailbox nobody runs (MBX-SQ-02)",
    ),
    "system-queue-swallows-messages-after-close": (
        "src/dispatch/system_queue.rs",
        "            State::Closed => Err(message),",
        "            State::Closed => Ok(()),",
        "a system message sent after termination is accepted and lost instead of handed back for dead letters, so a Watch is never answered",
    ),
    "system-queue-clean-up-drops-pending": (
        "src/dispatch/system_queue.rs",
        "            State::Open(messages) => messages,",
        "            State::Open(_) => Vec::new(),",
        "closing the mailbox forgets the system messages that were still pending, so none of them reaches dead letters (MBX-SQ-02)",
    ),
    "mailbox-suspension-does-not-stop-user-messages": (
        'src/actor/cell.rs',
        '        while self.status.should_process_message() {\n            let Some(env) = self.queue.dequeue() else {',
        '        while !self.status.is_closed() {\n            let Some(env) = self.queue.dequeue() else {',
        'a suspended actor keeps processing user messages (the turn only checks `Closed`, not the suspend count)',
    ),
    "mailbox-throughput-off-by-one": (
        'src/dispatch/turn.rs',
        'if self.left <= 1 {',
        'if self.left == 0 {',
        'a turn processes one message more than `throughput`, breaking the fairness between actors on one thread',
    ),
    "mailbox-dead-user-messages-not-published": (
        "src/actor/cell.rs",
        "        for env in enqueued.dead {",
        "        for env in enqueued.dead.into_iter().take(0) {",
        "a user message sent to a closed mailbox is dropped instead of published as a dead letter",
    ),
    "mailbox-dead-system-message-not-answered": (
        'src/actor/cell.rs',
        '        self.system\n            .dead_letter_mailbox\n            .system_enqueue(&self.self_ref(), msg);',
        '        drop(msg);',
        'a system message sent to a terminated actor is dropped, so a `Watch` is never answered with `Terminated` (DeathWatch hole)',
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
        "        self.queue.clean_up(&receiver, &*leftovers); // `cleanUp`; a producer that raced with the close drains for itself (Q-MBX-8)\n",
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
        'src/dispatch/turn.rs',
        'left: self.throughput.max(1),',
        'left: usize::MAX,',
        'an actor drains its whole mailbox in one turn instead of `throughput` messages',
    ),
    # --- Identify / ActorIdentity (AC-REF-013; killed by the identify-* cases) ---
    "identify-reaches-receive": (
        "src/actor/cell.rs",
        "if let Some(identify) = env.message.downcast_ref::<Identify>() {",
        "if let Some(identify) = env.message.downcast_ref::<Identify>().filter(|_| false) {",
        "the cell does not auto-receive `Identify`: it reaches `receive` and nobody gets an `ActorIdentity`",
    ),
    "identify-of-a-dead-actor-also-published": (
        "src/actor/system.rs",
        "tell_no_such_actor(identify, &d.sender, None);\n                true",
        "tell_no_such_actor(identify, &d.sender, None);\n                false",
        "an `Identify` sent to a stopped actor is answered and also published as a dead letter",
    ),
    "identify-of-dead-letters-loses-reply-sender": (
        "src/actor/system.rs",
        "tell_no_such_actor(identify, asker, Some(asker.clone()));",
        "tell_no_such_actor(identify, asker, None);",
        "the reply to an `Identify` told to `deadLetters` has no sender instead of the asker",
    ),
    # --- restart of the surviving children (AC-SUP-015; killed by the sv-* cases) ---
    "restart-leaves-survivors-suspended": (
        "src/actor/cell.rs",
        "for survivor in survivors {",
        "for survivor in survivors.into_iter().filter(|_| false) {",
        "a restarted parent does not restart the children its `preRestart` kept: they stay suspended",
    ),
    "failed-restart-still-restarts-survivors": (
        "src/actor/cell.rs",
        "self.fail(failure, None, None, &survivors);\n            return;\n        }\n        for",
        "self.fail(failure, None, None, &survivors);\n        }\n        for",
        "the survivors are restarted although the parent's `postRestart` failed",
    ),
    "failed-restart-suspends-survivors-again": (
        "src/actor/cell.rs",
        "&& !still_suspended.contains(k)",
        "|| still_suspended.contains(k)",
        "a failing restart suspends the survivors a second time, so the next restart does not resume them",
    ),
    "pill-not-suppressed-from-dead-letters": (
        "src/actor/system.rs",
        "msg.is::<Terminated>() || msg.is::<PoisonPill>()",
        "msg.is::<Terminated>()",
        "a PoisonPill sent to a dead actor is published as DeadLetter (it is DeadLetterSuppression in Pekko)",
    ),
    "dead-letters-answers-a-watch-of-itself": (
        "src/actor/system.rs",
        "if watchee != me && watcher != me {",
        "if true {",
        "watching deadLetters is answered with a Terminated, although DeadLetterActorRef ignores a Watch whose watchee is itself (MBX-DL-01)",
    ),
    "throughput-deadline-ignored": (
        "src/dispatch/turn.rs",
        "&& clock.current_time().wrapping_sub(deadline) >= 0",
        "&& clock.current_time().wrapping_sub(deadline) >= 0 && false",
        "throughput-deadline-time never ends a turn, so one actor keeps the thread for its whole throughput (MBX-SC-02)",
    ),
    "drop-uid-in-serialization-format": (
        "src/actor/path.rs",
        "if uid != UNDEFINED_UID {\n            s.push('#');",
        "if false {\n            s.push('#');",
        "toSerializationFormat forgets the #uid suffix",
    ),
    "actor-name-ending-in-an-escape-rejected": (
        "src/actor/path.rs",
        "&& pos + 2 < units.len()",
        "&& pos + 3 < units.len()",
        "an actor name that ends with a percent escape (\"%41\", \"a%4A\") is rejected: an off-by-one in the bounds check of the hex digits (#315, name-ok-28)",
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
        'src/actor/system.rs',
        '                        actor: watchee.clone(),\n                        existence_confirmed: false,',
        '                        actor: watchee.clone(),\n                        existence_confirmed: true,',
        'watching an already terminated actor reports `existenceConfirmed = true`',
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
        '        self.fail(cause, Some(message), Some(child), &[]);',
        '        let cause = Throwable::new("org.apache.pekko.actor.ActorInitializationException", cause.message.as_deref());\n        self.fail(cause, Some(message), Some(child), &[]);',
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
        "self.fail(thrown, Some(failed), Some(child), &[]);",
        "self.fail(thrown, None, None, &[]);",
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
    # --- Q-MBX-11 slice C1: a cell that cannot be made (killed by the in-crate twins of MBX-IF-01..04, see UNIT_MUTANTS) ---
    "init-with-failure-forgets-the-failure": (
        "src/actor/cell.rs",
        "            Some(initialization_failure),",
        "            None,",
        "the replacement cell of an actor whose cell could not be made starts the actor normally: the failure is lost and nobody is told",
    ),
    "init-with-failure-text-differs": (
        "src/actor/cell.rs",
        'Some(&format!("{path}: failure while creating ActorCell")),',
        'Some(&format!("{path}: exception during creation")),',
        "the ActorInitializationException of a cell that could not be made loses its `failure while creating ActorCell` text",
    ),
    "recreate-of-a-failed-creation-stays-a-recreate": (
        "src/actor/cell.rs",
        "if self.has_no_instance() {\n            // there is no instance to restart",
        "if false && self.has_no_instance() {\n            // there is no instance to restart",
        "a Recreate that reaches an actor with no instance is not turned into a Create: no `changing Recreate into Create` error",
    ),
    "resume-of-a-failed-creation-stays-a-resume": (
        "src/actor/cell.rs",
        'self.change_into_create("Resume", cause.as_ref());',
        "self.fault_create();",
        "a Resume that reaches an actor with no instance creates it without the `changing Resume into Create` error",
    ),
    "fault-create-does-not-wait-for-the-children": (
        "src/actor/cell.rs",
        "if !self.children.lock().unwrap().set_creation_reason() {",
        "if true {",
        "a failed creation is retried before its children are gone, so the new instance finds their names still taken",
    ),
    # --- mailbox of the deployment, looked up by path (killed by MBX-CFG-04) ---
    "deployment-looked-up-by-the-path-of-the-parent": (
        "src/actor/cell.rs",
        "lookup_path(&path)",
        "lookup_path(&self.path)",
        "the deployment is looked up with the path of the parent instead of the path of the new actor (MBX-CFG-04)",
    ),
    "deployment-looked-up-by-the-name-only": (
        "src/actor/cell.rs",
        "lookup_path(&path)",
        "lookup(std::iter::once(name.as_str()))",
        "the deployment is looked up by the name of the child alone, so a deployment of a top-level actor with that name applies to a child (MBX-CFG-04)",
    ),
    # --- mailbox of the deployment (killed by the unit test of Props::deployed_over, see UNIT_MUTANTS) ---
    "deployment-mailbox-ignored": (
        "src/actor/props.rs",
        "            Some(mailbox) if mailbox != NO_MAILBOX_GIVEN => self.with_mailbox_id(mailbox),",
        "            Some(mailbox) if mailbox != NO_MAILBOX_GIVEN => {\n                let _ = mailbox;\n                self\n            }",
        "the mailbox of the deployment found for the path is ignored: the props keep theirs",
    ),
    "deployment-empty-mailbox-resets-the-props": (
        "src/actor/props.rs",
        "            Some(mailbox) if mailbox != NO_MAILBOX_GIVEN => self.with_mailbox_id(mailbox),",
        "            Some(mailbox) => self.with_mailbox_id(mailbox),",
        "a deployment without a mailbox (empty) replaces the mailbox of the props with the default one",
    ),
    "props-mailbox-beats-the-deployment": (
        "src/actor/props.rs",
        "            Some(mailbox) if mailbox != NO_MAILBOX_GIVEN => self.with_mailbox_id(mailbox),",
        "            Some(mailbox) if mailbox != NO_MAILBOX_GIVEN && self.mailbox_id.is_none() => {\n                self.with_mailbox_id(mailbox)\n            }",
        "the precedence is inverted: the deployment only applies when the props selected nothing",
    ),
    # --- mailbox selection by id (killed by MBX-ID-01..03) ---
    "mailbox-id-selection-ignored": (
        "src/actor/props.rs",
        "                    (Some(_), _, _) => mailboxes.lookup(self.mailbox_id())?,",
        "                    (Some(_), _, _) => {\n                        mailboxes.lookup(self.mailbox_id())?;\n                        mailboxes.lookup(Mailboxes::DEFAULT_MAILBOX_ID)?\n                    }",
        "the id is looked up (so an unknown id still fails) but the actor keeps the default mailbox instead of the selected one (MBX-ID-01)",
    ),
    # --- mailbox registered as the default (killed by MBX-CFG-02) ---
    "mailbox-registered-default-ignored-without-a-selection": (
        "src/actor/props.rs",
        "        if self.mailbox.is_none() {",
        "        if self.mailbox.is_none() && self.mailbox_id.is_some() {",
        "an actor without a selection keeps the built-in unbounded mailbox even when a type is registered as pekko.actor.default-mailbox (MBX-CFG-02)",
    ),
    "mailbox-id-always-resolves-to-the-default": (
        "src/dispatch/mailboxes.rs",
        "        self.registered.get(id).cloned().ok_or_else(|| {",
        "        Some(Arc::new(UnboundedMailbox) as Arc<dyn MailboxType>).ok_or_else(|| {",
        "every registered id resolves to the default unbounded mailbox (MBX-ID-01)",
    ),
    "mailbox-id-resolves-to-the-wrong-type": (
        "src/dispatch/mailboxes.rs",
        "        self.registered.get(id).cloned().ok_or_else(|| {",
        "        self.registered.values().next().cloned().ok_or_else(|| {",
        "a registered id resolves to the first registered type instead of its own (MBX-ID-01)",
    ),
    "mailbox-unknown-id-is-accepted": (
        "src/dispatch/mailboxes.rs",
        "        self.registered.get(id).cloned().ok_or_else(|| {\n            ConfigurationException::new(format!(\"Mailbox Type [{id}] not configured\"))\n        })",
        "        Ok(self.registered.get(id).cloned().unwrap_or_else(|| Arc::new(UnboundedMailbox)))",
        "an id nobody registered silently resolves to the default mailbox, so a typo in the id is never reported (MBX-ID-02)",
    ),
    "mailbox-invalid-id-falls-back-to-the-default-and-creates-the-actor": (
        "src/actor/cell.rs",
        "        let props = match props.resolve_mailbox(\n            &self.system.settings.mailboxes,\n            self.system.settings.mailbox_requirement,\n        ) {\n            Ok(resolved) => resolved,\n            Err(cause) => {",
        "        let props = match props.clone().resolve_mailbox(\n            &self.system.settings.mailboxes,\n            self.system.settings.mailbox_requirement,\n        ) {\n            Ok(resolved) => resolved,\n            Err(_) if true => props,\n            Err(cause) => {",
        "the creation goes on with the default mailbox after the selected id could not be resolved: the actor exists although the selection was invalid (MBX-ID-02)",
    ),
    "mailbox-invalid-id-keeps-the-name-reserved": (
        "src/actor/cell.rs",
        "            Err(cause) => {\n                self.children.lock().unwrap().unreserve(&name);\n                return Err(ConfigurationError::caused_by(",
        "            Err(cause) => {\n                return Err(ConfigurationError::caused_by(",
        "a failed creation leaves its name reserved, so the same name cannot be used again (MBX-ID-02)",
    ),
    "mailbox-id-error-message-names-the-default-id": (
        "src/actor/cell.rs",
        "let mailbox_id = props.mailbox_id().to_owned();",
        "let mailbox_id = crate::dispatch::mailboxes::Mailboxes::DEFAULT_MAILBOX_ID.to_owned();",
        "the error always names the default mailbox instead of the id that failed (MBX-ID-02)",
    ),
    # --- mailbox selected by the queue semantics an actor requires (killed by MBX-REQ-01..03, or by the unit test where noted) ---
    "mailbox-actor-requirement-ignored": (
        "src/actor/props.rs",
        "                    (None, Some(semantics), _) => mailboxes\n                        .lookup_by_queue_type(semantics)\n                        .or_else(dispatcher_requirement_as_fallback)?,",
        "                    (None, Some(_), _) => mailboxes.lookup(self.mailbox_id())?,",
        "the queue semantics the actor requires is ignored: the actor gets the default mailbox (MBX-REQ-01)",
    ),
    "mailbox-requirement-beats-the-selected-id": (
        "src/actor/props.rs",
        "                    (Some(_), _, _) => mailboxes.lookup(self.mailbox_id())?,\n                    (None, Some(semantics), _) => mailboxes\n                        .lookup_by_queue_type(semantics)\n                        .or_else(dispatcher_requirement_as_fallback)?,",
        "                    (_, Some(semantics), _) => mailboxes\n                        .lookup_by_queue_type(semantics)\n                        .or_else(dispatcher_requirement_as_fallback)?,\n                    (Some(_), None, _) => mailboxes.lookup(self.mailbox_id())?,",
        "the precedence is inverted: the requirement of the actor decides the mailbox even when the props or the deployment selected one (MBX-REQ-03)",
    ),
    "mailbox-requirement-binding-ignored": (
        "src/dispatch/mailboxes.rs",
        "            Some(mailbox_id) => self.lookup(mailbox_id),",
        "            Some(_) => self.lookup(Self::UNBOUNDED_QUEUE_BASED_ID),",
        "every requirement resolves to the mailbox bound to the unbounded semantics, whatever the configuration binds (MBX-REQ-01)",
    ),
    "mailbox-requirement-applied-to-every-actor": (
        "src/actor/props.rs",
        "            required_queue: A::required_message_queue(),",
        "            required_queue: Some(MessageQueueSemantics::UNBOUNDED),",
        "every actor is treated as requiring the unbounded semantics, so one that requires nothing loses the default mailbox (MBX-REQ-01)",
    ),
    "mailbox-requirement-without-binding-uses-the-default": (
        "src/dispatch/mailboxes.rs",
        "            None => Err(ConfigurationException::new(format!(\n                \"Mailbox Mapping for [{semantics}] not configured\"\n            ))),",
        "            None => self.lookup(Self::DEFAULT_MAILBOX_ID),",
        "a requirement nobody bound is served by the default mailbox, so the missing configuration is never reported (MBX-REQ-02)",
    ),
    "mailbox-requirement-error-names-the-class-without-its-kind": (
        "src/dispatch/queue_semantics.rs",
        "        write!(f, \"interface {}\", self.class_name)",
        "        write!(f, \"{}\", self.class_name)",
        "the error of an unbound requirement names the class without the `interface` the JVM prints (MBX-REQ-02)",
    ),
    "mailbox-dispatcher-requirement-ignored": (
        "src/actor/props.rs",
        "                    (None, None, Some(semantics)) => mailboxes.lookup_by_queue_type(semantics)?,",
        "                    (None, None, Some(_)) => mailboxes.lookup(self.mailbox_id())?,",
        "the semantics the dispatcher requires is ignored: an actor that requires nothing gets the default mailbox (MBX-REQ-04)",
    ),
    "mailbox-dispatcher-requirement-beats-the-actor": (
        "src/actor/props.rs",
        "                    (None, Some(semantics), _) => mailboxes\n                        .lookup_by_queue_type(semantics)\n                        .or_else(dispatcher_requirement_as_fallback)?,",
        "                    (None, Some(semantics), _) => mailboxes\n                        .lookup_by_queue_type(dispatcher_requirement.unwrap_or(semantics))?,",
        "the requirement of the dispatcher is consulted before the one of the actor (MBX-REQ-04)",
    ),
    "mailbox-dispatcher-requirement-is-not-the-fallback": (
        "src/actor/props.rs",
        ".or_else(dispatcher_requirement_as_fallback)?",
        "?",
        "an actor whose own requirement has no binding fails instead of using the requirement of the dispatcher (MBX-REQ-04)",
    ),
    "mailbox-dispatcher-requirement-beats-the-selected-id": (
        "src/actor/props.rs",
        "                    (Some(_), _, _) => mailboxes.lookup(self.mailbox_id())?,",
        "                    (Some(_), _, Some(semantics)) => mailboxes.lookup_by_queue_type(semantics)?,\n                    (Some(_), _, None) => mailboxes.lookup(self.mailbox_id())?,",
        "the requirement of the dispatcher decides the mailbox even when the props or the deployment selected one (MBX-REQ-04)",
    ),
    "mailbox-dispatcher-any-queue-counts-as-a-requirement": (
        "src/actor/props.rs",
        "            let dispatcher_requirement = dispatcher_requirement\n                .filter(|semantics| *semantics != MessageQueueSemantics::MESSAGE_QUEUE);",
        "            let dispatcher_requirement = dispatcher_requirement;",
        "a dispatcher that requires `MessageQueue` (no requirement in Pekko) makes every actor without a binding for it fail (MBX-REQ-04)",
    ),
    # --- Q-MBX-11 slice C2: a rejected Dispatcher.attach (killed by the in-crate twins of MBX-RJ-05..07, see UNIT_MUTANTS) ---
    "attach-rejection-swallowed-by-start": (
        "src/actor/cell.rs",
        "self.register_for_execution(false, false).map(drop)",
        "{ let _ = self.register_for_execution(false, false); Ok(()) }",
        "`ActorCell::start` swallows the executor's rejection: neither the creator nor the supervisor ever learns of it",
    ),
    "point-swallows-the-attach-rejection": (
        "src/actor/repointable.rs",
        "        cell.start()?;",
        "        let _ = cell.start();",
        "`point` ignores a rejected attach and installs the cell that never started, as if `start` were inside `catchFailures`",
    ),
    "supervisor-ignores-a-rejected-point": (
        "src/actor/cell.rs",
        "if repointable.point().is_err() {\n                self.handle_invoke_failure(rejected_execution(), None);\n            }",
        "let _ = repointable.point();",
        "the supervisor that processes `Supervise` does not fail when attaching the new cell is rejected",
    ),
    "sync-actor-of-swallows-the-attach-rejection": (
        "src/actor/cell.rs",
        "if child.start().is_err() {\n                throw(rejected_execution());\n            }",
        "let _ = child.start();",
        "`context.actorOf` returns the reference of a child whose attach was rejected instead of throwing to the caller",
    ),
    "sync-actor-of-rejection-releases-the-name": (
        "src/actor/cell.rs",
        "            if child.start().is_err() {\n                throw(rejected_execution());",
        "            if child.start().is_err() {\n                self.children.lock().unwrap().remove(&child_ref);\n                throw(rejected_execution());",
        "after a rejected attach `context.actorOf` forgets the child, so its name can be reused (Pekko keeps it taken)",
    ),
    # --- Q-MBX-11 slice C4: actors that never started (killed by the in-crate twins of MBX-UT-01..09, see UNIT_MUTANTS) ---
    "stop-marks-a-child-that-has-not-started": (
        "src/actor/cell.rs",
        "let may_wait_for_death = actor.0.as_repointable().is_none_or(|r| r.is_started());",
        "let may_wait_for_death = true;",
        "`stop` marks a child that never started as about to die, so its parent waits for a termination that cannot happen until the child is pointed (MBX-UT-03, MBX-UT-06)",
    ),
    "swap-lookup-is-omitted": (
        "src/actor/repointable.rs",
        "        self.swap_lookup(cell.clone());\n",
        "",
        "the lookup of children is never pointed to the real cell: a child created in `preStart` is not found through a reference that is still unstarted (MBX-UT-01)",
    ),
    "lookup-children-are-always-empty": (
        "src/actor/repointable.rs",
        "            .map(|cell| cell.children())\n            .unwrap_or_default()",
        "            .map(|_| Vec::new())\n            .unwrap_or_default()",
        "a started reference answers no children (MBX-UT-01)",
    ),
    "is-started-while-handing-over": (
        "src/actor/repointable.rs",
        "matches!(&*self.lock(), Underlying::Started(_))",
        "matches!(&*self.lock(), Underlying::Started(_) | Underlying::HandingOver { .. })",
        "`isStarted` is true while the real cell is still being handed the queues (MBX-UT-09)",
    ),
    "system-message-after-close-is-not-dead-lettered": (
        "src/dispatch/dead_letter_mailbox.rs",
        "pub(crate) fn system_enqueue(&self, receiver: &ActorRef, handle: SystemMessage) {\n",
        "pub(crate) fn system_enqueue(&self, receiver: &ActorRef, handle: SystemMessage) {\n        if receiver.0.is_terminated() {\n            return;\n        }\n",
        "a system message (here the `Supervise` of a child whose parent already terminated) is silently dropped instead of becoming a dead letter (MBX-UT-03)",
    ),
    # --- Q-MBX-11 slice C3: UnstartedPushTimeout (killed by the in-crate twins of MBX-PT-01..06, see UNIT_MUTANTS) ---
    "push-timeout-not-applied-to-user-messages": (
        "src/actor/repointable.rs",
        "let limit = self.push_policy.as_ref().map(|policy| policy.timeout);",
        "let limit: Option<Duration> = None;",
        "a user message sent while the queue is handed over waits for as long as it takes instead of at most `unstarted-push-timeout`",
    ),
    "push-timeout-limit-ignored": (
        "src/actor/repointable.rs",
        "let deadline = limit.and_then(|limit| Instant::now().checked_add(limit));",
        "let deadline = limit.map(|_| Instant::now());",
        "the wait for the lock of an unstarted reference ends at once, whatever the timeout",
    ),
    "push-timeout-overflow-panics": (
        "src/actor/repointable.rs",
        "let deadline = limit.and_then(|limit| Instant::now().checked_add(limit));",
        "let deadline = limit.map(|limit| Instant::now() + limit);",
        "a timeout too long to add to the clock panics the sender instead of never expiring",
    ),
    "config-push-timeout-not-read": (
        "src/actor/system.rs",
        "            unstarted_push_timeout,\n            deployer: Arc::new(Deployer::from_settings(config)?),\n            ..Settings::default()",
        "            unstarted_push_timeout: Duration::from_secs(10),\n            deployer: Arc::new(Deployer::from_settings(config)?),\n            ..Settings::default()",
        "the configured `unstarted-push-timeout` never reaches the runtime settings: every system uses the default",
    ),
    "config-negative-push-timeout-is-the-default": (
        "src/actor/system.rs",
        ".map_or(Duration::ZERO, Duration::from_nanos);",
        ".map_or(Duration::from_secs(10), Duration::from_nanos);",
        "a negative configured `unstarted-push-timeout` becomes the default instead of zero (the JVM's `tryLock` treats it as zero)",
    ),
    "actor-of-reference-ignores-the-configured-push-timeout": (
        "src/actor/cell.rs",
        "timeout: self.system.settings.unstarted_push_timeout,",
        "timeout: std::time::Duration::from_secs(10),",
        "the references made by `actorOf` use the default `unstarted-push-timeout` whatever the system was configured with",
    ),
    "unknown-rust-type-is-given-a-jvm-class": (
        "src/actor/repointable.rs",
        '"class <unknown Rust type>"',
        '"class java.lang.Object"',
        "the Warning names a JVM class (`java.lang.Object`) for a Rust type that has none",
    ),
    "hand-over-thread-waits-for-itself": (
        "src/actor/repointable.rs",
        "if *owner != this_thread)",
        "if true)",
        "the thread that is handing the queue over waits for its own lock when it sends to the reference (Pekko's lock is reentrant)",
    ),
    "lock-timeout-drop-without-warning": (
        "src/actor/repointable.rs",
        "policy.events.publish_warning(WarningEvent {",
        "drop(WarningEvent {",
        "a message dropped after the lock timeout publishes no Warning",
    ),
    "lock-timeout-drop-without-dead-letter": (
        "src/actor/repointable.rs",
        "if let Some(recipient) = self.me.upgrade() {",
        "if let Some(recipient) = self.me.upgrade().filter(|_| false) {",
        "a message dropped after the lock timeout is not sent to the dead letters",
    ),
    "hand-over-end-does-not-wake-the-senders": (
        "src/actor/repointable.rs",
        "        self.reference.handed_over.notify_all();\n",
        "",
        "the senders that waited for the hand-over are not told when it ends",
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
        "            },\n            tags: Vec::new(),\n        }\n    }\n}\n\n#[cfg(test)]",
        "            },\n            tags: self.tags.clone(),\n        }\n    }\n}\n\n#[cfg(test)]",
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
    # --- P10 PR-C fixes D2, D3, D4: concatenations and substitution paths in the concise render ---
    "concatenation-drops-the-text-of-null": (
        "src/config/mod.rs",
        '}) => Some("null".to_owned()),',
        '}) => Some(String::new()),',
        "a null inside a concatenation of plain values contributes no text",
    ),
    "concatenation-number-loses-trailing-zeros": (
        "src/config/mod.rs",
        "}) => Some(number.text().to_owned()),",
        "}) => Some(number.text().trim_end_matches('0').to_owned()),",
        "a number inside a concatenation of plain values is shown without its trailing zeros",
    ),
    "concatenation-keeps-white-space-next-to-containers": (
        "src/config/mod.rs",
        "        let touches_container = matches!(piece, Piece::Ws(_))\n",
        "        let touches_container = false && matches!(piece, Piece::Ws(_))\n",
        "white space next to a list or an object is kept as a string",
    ),
    "concatenation-does-not-fuse-adjacent-lists": (
        "src/config/mod.rs",
        "                    items.extend(next.iter());\n",
        "                    let _ = next;\n",
        "the second of two adjacent lists is lost",
    ),
    "concatenation-fuses-objects-in-the-wrong-order": (
        "src/config/mod.rs",
        "fused = merge_obj(next, &fused);",
        "fused = merge_obj(&fused, next);",
        "of two adjacent objects the first wins on a repeated key",
    ),
    "concatenation-quotes-a-lone-plain-value": (
        "src/config/mod.rs",
        "if let Piece::V(value) = kept[start] {",
        "if let (Piece::V(value), false) = (kept[start], true) {",
        "a lone number between substitutions is printed as a string",
    ),
    "substitution-path-quotes-digit-elements": (
        "src/config/model.rs",
        "(c.is_alphabetic() || c.is_numeric())",
        "(c.is_alphabetic())",
        "a path element made of digits is printed with quotes",
    ),
    "substitution-path-leaves-astral-elements-bare": (
        "src/config/model.rs",
        "(c as u32) <= 0xFFFF && ",
        "",
        "a path element with an astral letter is printed without quotes",
    ),
    # --- P10 PR-C: Deploy.toString and the concise render of a configuration ---
    "render-keys-sort-as-plain-strings": (
        "src/config/mod.rs",
        "        (Some(_), None) => std::cmp::Ordering::Less,\n        (None, Some(_)) => std::cmp::Ordering::Greater,\n",
        "        (Some(_), None) => std::cmp::Ordering::Greater,\n        (None, Some(_)) => std::cmp::Ordering::Less,\n",
        "keys made of digits are listed after the other keys instead of before",
    ),
    "render-numeric-keys-compare-as-text": (
        "src/config/mod.rs",
        "a.len().cmp(&b.len()).then_with(|| a.cmp(&b))",
        "a.cmp(&b)",
        "the digit keys 9 and 10 are ordered as text",
    ),
    "render-shows-only-the-top-layer-of-a-key": (
        "src/config/mod.rs",
        ".flat_map(|(k, stack)| stack.iter().rev().map(move |layer| (k, layer)))",
        ".filter_map(|(k, stack)| stack.first().map(|layer| (k, layer)))",
        "an unresolved key shows only its highest-priority layer",
    ),
    "render-concatenation-keeps-white-space-as-a-separate-string": (
        "src/config/mod.rs",
        "        Piece::Ws(white) => Some(white.clone()),\n",
        "        Piece::Ws(_) => None,\n",
        "the white space between a substitution and a string is dropped from the concatenation",
    ),
    "tags-are-listed-sorted": (
        "src/actor/deploy.rs",
        "return format!(\"Set({})\", tags.join(\", \"));",
        "let mut sorted = tags.to_vec();\n        sorted.sort();\n        return format!(\"Set({})\", sorted.join(\", \"));",
        "up to four tags are listed in alphabetical order instead of insertion order",
    ),
    "five-tags-print-as-a-small-set": (
        "src/actor/deploy.rs",
        "if tags.len() <= 4 {",
        "if tags.len() <= 5 {",
        "five tags are printed in insertion order as a Set instead of a HashSet in hash order",
    ),
    "surrogate-escape-pair-becomes-two-replacement-characters": (
        "src/config/parse.rs",
        "if (0xD800..0xDC00).contains(&code) {",
        "if (0xD800..0xDC00).contains(&code) && false {",
        "a pair of surrogate escapes is read as two replacement characters",
    ),
    # --- Q-UTIL-21 / Q-X-21: the choice among overlapping `prefix*` entries follows the iteration order of Scala's HashMap ---
    "wildcard-suffix-children-keep-insertion-order": (
        "src/util/wildcard_index.rs",
        "            updated.sort_suffix_children_in_hash_map_order();\n",
        "",
        "overlapping prefix* entries are tried in insertion order instead of the HashMap order",
    ),
    "hash-map-order-skips-the-improve-scrambling": (
        "src/util/scala_hash_order.rs",
        ".map(|(position, key)| (position, improve(string_hash_code(key))))",
        ".map(|(position, key)| (position, string_hash_code(key) as u32))",
        "the trie consumes the raw String.hashCode instead of the scrambled one",
    ),
    "hash-map-order-reads-the-bits-from-the-top": (
        "src/util/scala_hash_order.rs",
        "groups[((entry.1 >> shift) & 0x1f) as usize].push(entry);",
        "groups[(((entry.1.reverse_bits()) >> shift) & 0x1f) as usize].push(entry);",
        "the five-bit chunks are taken from the most significant end",
    ),
    "hash-map-order-puts-child-nodes-before-lone-entries": (
        "src/util/scala_hash_order.rs",
        "    for group in groups.iter().filter(|group| group.len() == 1) {\n        out.push(group[0].0);\n    }\n    for group in groups.into_iter().filter(|group| group.len() > 1) {\n        node_order(group, shift + BITS_PER_LEVEL, out);\n    }\n",
        "    for group in groups.iter().filter(|group| group.len() > 1) {\n        node_order(group.clone(), shift + BITS_PER_LEVEL, out);\n    }\n    for group in groups.iter().filter(|group| group.len() == 1) {\n        out.push(group[0].0);\n    }\n",
        "a node iterates its children before its own entries",
    ),
    "hash-map-order-reverses-colliding-keys": (
        "src/util/scala_hash_order.rs",
        "out.extend(entries.into_iter().map(|(position, _)| position));",
        "out.extend(entries.into_iter().rev().map(|(position, _)| position));",
        "keys with the same scrambled hash iterate in reverse insertion order",
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
    "nonblocking-queue-keeps-one-over-capacity": (
        "src/dispatch/bounded_node_queue.rs",
        "        if queue.len() < self.capacity {",
        "        if queue.len() <= self.capacity {",
        "an off-by-one lets a full NonBlockingBoundedMailbox keep one envelope more than its capacity (NB-QU-01)",
    ),
    "nonblocking-queue-drops-the-overflow-silently": (
        "src/dispatch/bounded_node_queue.rs",
        "        tell_dead_letter(&self.dead_letters, receiver, envelope);\n        false\n    }",
        "        drop(envelope);\n        false\n    }",
        "a refused envelope vanishes instead of reaching deadLetters (NB-QU-01)",
    ),
    "nonblocking-queue-dead-letters-to-the-wrong-receiver": (
        "src/dispatch/bounded_node_queue.rs",
        "        tell_dead_letter(&self.dead_letters, receiver, envelope);",
        "        tell_dead_letter(&self.dead_letters, &self.dead_letters, envelope);",
        "the DeadLetter names deadLetters, not the mailbox's owner, as the receiver (NB-QU-04)",
    ),
    "nonblocking-queue-holds-its-lock-while-dead-lettering": (
        "src/dispatch/bounded_node_queue.rs",
        "        drop(queue); // `deadLetters` may call back into this queue\n",
        "",
        "the producer keeps the queue locked while it tells deadLetters, which cannot then read the queue",
    ),
    "nonblocking-queue-dequeues-the-newest": (
        "src/dispatch/bounded_node_queue.rs",
        "        self.locked().pop_front()",
        "        self.locked().pop_back()",
        "the consumer takes the newest envelope first (NB-QU-02)",
    ),
    "nonblocking-queue-accepts-a-negative-capacity": (
        "src/dispatch/bounded_node_queue.rs",
        "        usize::try_from(capacity)\n            .map(|capacity| NonBlockingBoundedMailbox { capacity })\n            .map_err(|_| NegativeCapacity)",
        "        Ok(NonBlockingBoundedMailbox {\n            capacity: capacity as usize,\n        })",
        "a negative capacity becomes an enormous one instead of an IllegalArgumentException (NB-CF-01)",
    ),
    "nonblocking-queue-panics-on-a-poisoned-lock": (
        "src/dispatch/bounded_node_queue.rs",
        "        self.queue.lock().unwrap_or_else(PoisonError::into_inner)",
        "        self.queue.lock().unwrap()",
        "one producer that panics under the lock makes every later send and receive of the mailbox panic",
    ),
    "supervise-is-processed-while-terminating": (
        "src/actor/children.rs",
        "            Children::Terminating { reason, .. } => *reason == SuspendReason::Termination,",
        "            Children::Terminating { .. } => false,",
        "a parent that waits for a started child to die before it terminates still points the child whose Supervise arrives meanwhile (MBX-UT-05)",
    ),
    "supervise-is-ignored-while-restarting": (
        "src/actor/children.rs",
        "            Children::Terminating { reason, .. } => *reason == SuspendReason::Termination,",
        "            Children::Terminating { .. } => true,",
        "a parent that waits for a started child to die before it restarts ignores the Supervise of a new child, which then never starts (MBX-UT-07)",
    ),
    "nonblocking-cell-schedules-a-turn-for-a-refused-message": (
        "src/actor/cell.rs",
        "        if enqueued.awaits_consumption\n",
        "        if (enqueued.awaits_consumption || true)\n",
        "a message the bounded queue refused still starts a consumption turn that finds nothing (Owner contract of the wiring)",
    ),
    "nonblocking-cell-addresses-a-refusal-to-dead-letters": (
        "src/actor/cell.rs",
        "        let receiver = self.self_ref();\n        let enqueued =",
        "        let receiver = self.system.dead_letters.clone();\n        let enqueued =",
        "the DeadLetter of a message refused by the actor's bounded mailbox names deadLetters, not the actor, as receiver",
    ),
    "nonblocking-cell-addresses-the-cleanup-to-dead-letters": (
        "src/actor/cell.rs",
        "        let receiver = self.self_ref();\n        self.queue.clean_up(",
        "        let receiver = self.system.dead_letters.clone();\n        self.queue.clean_up(",
        "what is still queued when the actor stops is dead-lettered as addressed to deadLetters instead of the actor",
    ),
    "nonblocking-queue-reports-a-refused-message-as-stored": (
        "src/dispatch/bounded_node_queue.rs",
        "        tell_dead_letter(&self.dead_letters, receiver, envelope);\n        false\n    }",
        "        tell_dead_letter(&self.dead_letters, receiver, envelope);\n        true\n    }",
        "enqueue answers true for an envelope it dead-lettered, so the cell schedules a turn for nothing",
    ),
    "enqueue-unless-closed-ignores-the-queue-answer": (
        "src/dispatch/mailbox_close.rs",
        "            awaits_consumption: stored,",
        "            awaits_consumption: true,",
        "a message the queue refused is reported as waiting for consumption",
    ),
    "replacement-cell-honours-the-selected-mailbox": (
        "src/actor/cell.rs",
        "        let queue = if creation_failure.is_some() {",
        "        let queue = if creation_failure.is_some() && false {",
        "the replacement of a cell that could not be made uses the mailbox the props selected instead of `new UnboundedMailbox` (Dispatch.scala:117-125)",
    ),
    "props-ignores-the-selected-mailbox": (
        "src/actor/props.rs",
        "            Some(mailbox) => mailbox.create(dead_letters),",
        "            Some(_) => UnboundedMailbox.create(),",
        "Props::with_mailbox has no effect: the actor keeps the unbounded mailbox",
    ),
    "nonblocking-queue-clean-up-moves-only-the-first": (
        "src/dispatch/bounded_node_queue.rs",
        "        while let Some(envelope) = self.dequeue() {\n            dead_letters.enqueue(owner, envelope);",
        "        if let Some(envelope) = self.dequeue() {\n            dead_letters.enqueue(owner, envelope);",
        "cleanUp moves one envelope and leaves the rest (NB-QU-05)",
    ),
    # --- the queue a mailbox type promises (ProducesMessageQueue); killed by unit tests (UNIT_MUTANTS) ---
    "produced-queue-ignores-the-listed-semantics": (
        "src/dispatch/queue_semantics.rs",
        "requirement == MessageQueueSemantics::MESSAGE_QUEUE || self.satisfies.contains(&requirement)",
        "requirement == MessageQueueSemantics::MESSAGE_QUEUE",
        "a queue satisfies only MessageQueue: the semantics its type lists are ignored",
    ),
    "produced-queue-satisfies-any-requirement": (
        "src/dispatch/queue_semantics.rs",
        "requirement == MessageQueueSemantics::MESSAGE_QUEUE || self.satisfies.contains(&requirement)",
        "true",
        "a queue satisfies every requirement, listed or not",
    ),
    "produced-queue-does-not-satisfy-message-queue": (
        "src/dispatch/queue_semantics.rs",
        "requirement == MessageQueueSemantics::MESSAGE_QUEUE || self.satisfies.contains(&requirement)",
        "self.satisfies.contains(&requirement)",
        "MessageQueue is satisfied only when the type lists it",
    ),
    "unbounded-mailbox-promises-no-semantics": (
        "src/dispatch/message_queue.rs",
        "            \"org.apache.pekko.dispatch.UnboundedMailbox$MessageQueue\",\n            &[\n                MessageQueueSemantics::UNBOUNDED,\n                MessageQueueSemantics::MULTIPLE_CONSUMER,\n            ],",
        "            \"org.apache.pekko.dispatch.UnboundedMailbox$MessageQueue\",\n            &[],",
        "UnboundedMailbox promises a queue that satisfies nothing",
    ),
    "bounded-mailbox-promises-an-unbounded-queue": (
        "src/dispatch/bounded_node_queue.rs",
        "                MessageQueueSemantics::BOUNDED,\n                MessageQueueSemantics::MULTIPLE_CONSUMER,",
        "                MessageQueueSemantics::UNBOUNDED,\n                MessageQueueSemantics::MULTIPLE_CONSUMER,",
        "NonBlockingBoundedMailbox promises the unbounded semantics instead of the bounded one",
    ),
    "type-without-promise-claims-an-unbounded-queue": (
        "src/dispatch/message_queue.rs",
        "    fn produced_queue(&self) -> ProducedQueue {\n        ProducedQueue::PLAIN\n    }",
        "    fn produced_queue(&self) -> ProducedQueue {\n        ProducedQueue::new(\"claimed\", &[MessageQueueSemantics::UNBOUNDED])\n    }",
        "a type that makes no promise is taken to produce an unbounded queue",
    ),
    # --- verifyRequirements: the selected type must fulfill the requirement of the actor and of the dispatcher (unit tests) ---
    "mailbox-actor-requirement-not-verified": (
        "src/actor/props.rs",
        "        check(\n            self.required_queue,\n            format!(\"actor class [class {}]\", self.actor_class),\n        )",
        "        Ok(())",
        "the requirement of the actor is not verified against the queue the selected type promises",
    ),
    "mailbox-dispatcher-requirement-not-verified": (
        "src/actor/props.rs",
        "        check(\n            dispatcher_requirement,\n            format!(\"dispatcher [{DEFAULT_DISPATCHER_ID}]\"),\n        )?;\n",
        "",
        "the requirement of the dispatcher is not verified against the queue the selected type promises",
    ),
    "mailbox-requirement-verified-actor-first": (
        "src/actor/props.rs",
        "        check(\n            dispatcher_requirement,\n            format!(\"dispatcher [{DEFAULT_DISPATCHER_ID}]\"),\n        )?;\n        check(\n            self.required_queue,\n            format!(\"actor class [class {}]\", self.actor_class),\n        )",
        "        check(\n            self.required_queue,\n            format!(\"actor class [class {}]\", self.actor_class),\n        )?;\n        check(\n            dispatcher_requirement,\n            format!(\"dispatcher [{DEFAULT_DISPATCHER_ID}]\"),\n        )",
        "the requirement of the actor is reported before that of the dispatcher",
    ),
    "mailbox-failed-binding-has-no-dispatcher-fallback": (
        "src/actor/props.rs",
        "            (None, Some(_), Some(semantics)) => {",
        "            (None, Some(_), Some(semantics)) if false => {",
        "an actor binding that does not fit never gives way to the binding of the dispatcher",
    ),
    "mailbox-fallback-binding-is-not-verified": (
        "src/actor/props.rs",
        "                self.verify_requirements(&*fallback, dispatcher_requirement)?;\n",
        "",
        "the binding of the dispatcher used as a fallback is accepted without verification",
    ),
    "mailbox-selected-id-is-not-verified": (
        "src/actor/props.rs",
        "            if let Some(selected) = self.mailbox.take() {",
        "            if let Some(selected) = self.mailbox.take().filter(|_| self.mailbox_id.is_none()) {",
        "a mailbox selected by id is not verified",
    ),
    # --- killed by MBX-CMP-01 (the oracle's text of the verification failure) ---
    "mailbox-incompatible-queue-is-a-configuration-cause": (
        "src/dispatch/mailboxes.rs",
        "            Self::IncompatibleQueue(_) => ErrorClass::IllegalArgument,",
        "            Self::IncompatibleQueue(_) => ErrorClass::Configuration,",
        "the cause of the failure is reported as a ConfigurationException instead of an IllegalArgumentException (MBX-CMP-01)",
    ),
    "mailbox-plain-queue-is-printed-as-a-class": (
        "src/dispatch/queue_semantics.rs",
        "        is_interface: true,",
        "        is_interface: false,",
        "the queue of a type without promise is printed as `class` instead of `interface` (MBX-CMP-01)",
    ),
    "mailbox-actor-class-is-named-without-its-kind": (
        "src/actor/props.rs",
        "\"actor class [class {}]\"",
        "\"actor class [{}]\"",
        "the actor class of the verification failure is printed without `class` (MBX-CMP-01)",
    ),
    "mailbox-fallback-failure-reports-the-first-error": (
        "src/actor/props.rs",
        "                self.verify_requirements(&*fallback, dispatcher_requirement)?;\n",
        "                self.verify_requirements(&*fallback, dispatcher_requirement)\n                    .map_err(|_| unfit)?;\n",
        "a fallback binding that does not fit reports the error of the first binding instead of its own (MBX-CMP-02)",
    ),
    "mailbox-dispatcher-requirement-checked-only-on-bindings": (
        "src/actor/props.rs",
        "        let unfit = match self.verify_requirements(&*selected, dispatcher_requirement) {",
        "        let unfit = match self.verify_requirements(&*selected, dispatcher_requirement.filter(|_| self.mailbox_id.is_none())) {",
        "a mailbox selected by id is not checked against the requirement of the dispatcher (MBX-CMP-02)",
    ),
    # --- Dispatch.init: killed by MBX-CMP-03 (the cell re-checks what the type created) ---
    "mailbox-created-queue-is-not-rechecked": (
        "src/actor/props.rs",
        "            .filter(unmet)\n            .map(|required| (required, queue.class_name()))",
        "            .filter(|_| false)\n            .map(|required| (required, queue.class_name()))",
        "the queue a type really creates is never checked against the requirement of the actor (MBX-CMP-03)",
    ),
    "unbounded-queue-has-no-multiple-consumer-semantics": (
        "src/dispatch/message_queue.rs",
        "                || requirement == MessageQueueSemantics::UNBOUNDED\n                || requirement == MessageQueueSemantics::MULTIPLE_CONSUMER,",
        "                || requirement == MessageQueueSemantics::UNBOUNDED,",
        "the built-in unbounded queue does not have MultipleConsumerSemantics (Mailbox.scala:538)",
    ),
    "multiple-consumer-requirement-has-no-default-binding": (
        "src/dispatch/mailboxes.rs",
        "                    MessageQueueSemantics::UNBOUNDED,\n                    MessageQueueSemantics::MULTIPLE_CONSUMER,\n",
        "                    MessageQueueSemantics::UNBOUNDED,\n",
        "reference.conf's binding of MultipleConsumerSemantics to unbounded-queue-based is missing",
    ),
    "mailbox-undeclared-queue-does-not-keep-the-promise": (
        "src/actor/props.rs",
        "                    .unwrap_or_else(|| promise.satisfies(*required))",
        "                    .unwrap_or(false)",
        "a queue that does not declare its semantics is rejected even though its type promised them (the review's medium finding)",
    ),
    "mailbox-recheck-names-the-required-class-as-created": (
        "src/actor/props.rs",
        "            .map(|required| (required, queue.class_name()))",
        "            .map(|required| (required, required.class_name()))",
        "the failure of Dispatch.init names the requirement where it should name the class of the queue (MBX-CMP-03)",
    ),
    "mailbox-unbounded-queue-has-no-effective-semantics": (
        "src/dispatch/message_queue.rs",
        "        Some(\n            requirement == MessageQueueSemantics::MESSAGE_QUEUE\n                || requirement == MessageQueueSemantics::UNBOUNDED\n                || requirement == MessageQueueSemantics::MULTIPLE_CONSUMER,\n        )",
        "        Some(requirement == MessageQueueSemantics::MESSAGE_QUEUE || requirement == MessageQueueSemantics::MULTIPLE_CONSUMER)",
        "the default unbounded queue does not have the unbounded semantics, so the actors that require it fail to initialise (MBX-REQ, family fall)",
    ),
    # --- the unbounded deque based mailbox (AC-MBX-011) ---
    "deque-enqueue-first-goes-to-the-tail": (
        "src/dispatch/deque_mailbox.rs",
        "self.envelopes().push_front(envelope);",
        "self.envelopes().push_back(envelope);",
        "enqueueFirst appends at the tail, so a message put back is processed after the ones that arrived meanwhile (MBX-DQ-01)",
    ),
    "deque-dequeue-takes-the-tail": (
        "src/dispatch/deque_mailbox.rs",
        "self.envelopes().pop_front()",
        "self.envelopes().pop_back()",
        "the deque mailbox delivers the newest message first (MBX-DQ-01)",
    ),
    # --- the single consumer only unbounded mailbox (AC-MBX-006) ---
    "single-consumer-dequeue-takes-the-tail": (
        "src/dispatch/single_consumer_mailbox.rs",
        "self.envelopes().pop_front()",
        "self.envelopes().pop_back()",
        "the single consumer queue delivers the newest message first (MBX-SCQ-01)",
    ),
    "single-consumer-queue-says-it-has-no-messages": (
        "src/dispatch/single_consumer_mailbox.rs",
        "!self.envelopes().is_empty()",
        "false",
        "has_messages answers false with messages queued, so the actor is not scheduled again (MBX-SCQ-01)",
    ),
    "single-consumer-queue-is-for-several-consumers": (
        "src/dispatch/single_consumer_mailbox.rs",
        "&[MessageQueueSemantics::UNBOUNDED],",
        "&[MessageQueueSemantics::UNBOUNDED, MessageQueueSemantics::MULTIPLE_CONSUMER],",
        "the queue declares MultipleConsumerSemantics, which Pekko's NodeMessageQueue does not (MBX-SCQ-01)",
    ),
    # --- guardians: the default mailbox and the unbounded requirement of each (killed by tests/classic_guardian_mailbox.rs, see UNIT_MUTANTS) ---
    "guardian-root-ignores-the-default-mailbox": (
        "src/actor/system.rs",
        "            Props::new(|| Guardian).with_resolved_mailbox(default_mailbox.clone()),\n            walker,",
        "            Props::new(|| Guardian),\n            walker,",
        "the root guardian does not take the registered default mailbox",
    ),
    "guardian-user-ignores-the-default-mailbox": (
        "src/actor/system.rs",
        "            \"user\",\n            Props::new(|| Guardian).with_resolved_mailbox(default_mailbox.clone()),",
        "            \"user\",\n            Props::new(|| Guardian),",
        "the /user guardian does not take the registered default mailbox",
    ),
    "guardian-system-ignores-the-default-mailbox": (
        "src/actor/system.rs",
        "            })\n            .with_resolved_mailbox(default_mailbox.clone()),",
        "            }),",
        "the /system guardian does not take the registered default mailbox",
    ),
    "guardian-requires-nothing": (
        "src/actor/system.rs",
        "impl Actor for Guardian {\n    fn required_message_queue() -> Option<MessageQueueSemantics> {\n        Some(MessageQueueSemantics::UNBOUNDED)\n    }",
        "impl Actor for Guardian {",
        "the root and /user guardians do not declare the unbounded requirement",
    ),
    "system-guardian-requires-nothing": (
        "src/actor/system.rs",
        "impl Actor for SystemGuardian {\n    fn required_message_queue() -> Option<MessageQueueSemantics> {\n        Some(MessageQueueSemantics::UNBOUNDED)\n    }",
        "impl Actor for SystemGuardian {",
        "the /system guardian does not declare the unbounded requirement",
    ),
    "failed-root-guardian-does-not-end-the-system": (
        "src/actor/system.rs",
        "                self.cause_of_termination.complete(Err(cause));\n                child.stop();\n",
        "                self.cause_of_termination.complete(Err(cause));\n",
        "the walker records the failure of the root guardian but does not stop it: its death notification never comes",
    ),
    "walker-keeps-walking-after-a-guardian-failed": (
        "src/actor/system.rs",
        "        if !self.is_walking() {",
        "        if false {",
        "the walker does not stop walking after a failure: the death of the root reaches stop() and completes whenTerminated",
    ),
    "walker-stop-links-only-a-success-to-the-public-future": (
        "src/actor/system.rs",
        "            if let Some(outcome) = self.cause_of_termination.wait(Duration::ZERO) {",
        "            if let Some(outcome @ Ok(_)) = self.cause_of_termination.wait(Duration::ZERO) {",
        "stop() after a guardian failed does not link the failure to the public future (completeWith of a failed future)",
    ),
    "walker-signal-fires-for-any-discarded-message": (
        "src/actor/system.rs",
        "            if matches!(&msg, SystemMessage::DeathWatchNotification { actor, .. } if self.root.get() == Some(actor))\n            {",
        "            if true {",
        "the test signal is set by any message the walker discards, not only by the death notification of the root",
    ),
    "dispatchers-guard-does-not-shut-down": (
        "src/actor/guardian_termination_tests.rs",
        "        self.0.shut_down_dispatchers();\n    }\n}",
        "    }\n}",
        "the guard of the test does not shut the dispatchers down when dropped",
    ),
    "walker-stop-does-not-complete-the-public-future": (
        "src/actor/system.rs",
        "                self.termination.complete(outcome);",
        "                let _ = outcome;",
        "stop() completes the walker's own promise but not the provider's terminationPromise that whenTerminated exposes",
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


HERMETIC_FLAGS = ["--locked", "--offline", "--message-format=json"]   # the authoritative Cargo.lock, no network, compiler diagnostics as data


def hermetic(command):
    """`cargo <subcommand> ...` as a locked, offline invocation that reports its diagnostics as JSON (the sub-command stays first)."""
    return [command[0], command[1], *HERMETIC_FLAGS, *command[2:]]


def provision_dependencies():
    """The one step that may use the network: fetches the locked dependencies into the Cargo cache. Every build afterwards is offline. -> (ok, detail)."""
    fetched = sh(["cargo", "fetch", "--locked", "--manifest-path", os.path.join(ROOT, "Cargo.toml")], timeout=BUILD_TIMEOUT)
    return fetched.returncode == 0, fetched.stderr[-600:]


def classify_cargo_failure(failed):
    """A failed locked, offline cargo run -> (BUILD_ERROR, rendered compiler errors) only when the compiler itself reported an error, else (HARNESS_ERROR, stderr):
    a missing dependency, a lock that no longer fits or a broken toolchain is a failure of the infrastructure, never a verdict about the mutation."""
    errors = []
    for line in failed.stdout.splitlines():
        try:
            diagnostic = json.loads(line)
        except ValueError:
            continue
        message = diagnostic.get("message") if isinstance(diagnostic, dict) and diagnostic.get("reason") == "compiler-message" else None
        if isinstance(message, dict) and message.get("level") == "error":
            errors.append(message.get("rendered") or "")
    if errors:
        return BUILD_ERROR, "".join(errors)[-600:]
    return HARNESS_ERROR, failed.stderr[-600:]


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


def classify(replay_result, binary):
    """-> (outcome, passed, total, line), counting only applicable cases (NOT_APPLICABLE ones must be rejected
    explicitly, see replay_lib). A mutant is detected only when the replay ran to completion and either an
    applicable case diverged or a NOT_APPLICABLE input was no longer rejected explicitly."""
    return classify_evaluation(replay_lib.evaluate(binary, replay_result))


def classify_evaluation(ev):
    """The verdict of one evaluation of a replay (the evaluation is computed once, so that the history seeded from it is the very one that was classified)."""
    if ev["ok"] is None:
        return HARNESS_ERROR, None, None, ev["line"]
    detail = ev["line"] + "".join(f"; {p}" for p in ev["na_problems"])
    if ev["ok"]:
        return UNDETECTED, ev["passed"], ev["total"], detail
    return DETECTED, ev["passed"], ev["total"], detail


# Mutants that the replay cannot see because they need an interleaving of two threads: they are killed by a deterministic
# unit test (no threads, no sleeps) that the harness runs instead of the replay.
UNIT_MUTANTS = {
    "walker-stop-links-only-a-success-to-the-public-future": ("actor::guardian_termination_tests", "stop_called_directly_after_the_failure_fails_the_public_future_with_the_cause"),
    "walker-signal-fires-for-any-discarded-message": ("actor::guardian_termination_tests", "only_the_death_notification_of_the_failed_root_is_the_synchronisation_signal"),
    "dispatchers-guard-does-not-shut-down": ("actor::guardian_termination_tests", "the_guard_shuts_the_dispatchers_down_when_the_test_unwinds"),
    "guardian-root-ignores-the-default-mailbox": ("test:classic_guardian_mailbox", "the_three_guardians_take_the_registered_default_mailbox"),
    "guardian-user-ignores-the-default-mailbox": ("test:classic_guardian_mailbox", "the_three_guardians_take_the_registered_default_mailbox"),
    "guardian-system-ignores-the-default-mailbox": ("test:classic_guardian_mailbox", "the_three_guardians_take_the_registered_default_mailbox"),
    "guardian-requires-nothing": ("actor::guardian_termination_tests", "a_failed_root_guardian_leaves_when_terminated_pending"),
    "system-guardian-requires-nothing": ("test:classic_guardian_mailbox", "the_user_and_system_guardians_that_fail_to_initialise_end_the_system_normally"),
    "failed-root-guardian-does-not-end-the-system": ("actor::guardian_termination_tests", "a_failed_root_guardian_leaves_when_terminated_pending"),
    "walker-keeps-walking-after-a-guardian-failed": ("actor::guardian_termination_tests", "a_failed_root_guardian_leaves_when_terminated_pending"),
    "walker-stop-does-not-complete-the-public-future": ("test:classic_guardian_mailbox", "a_default_mailbox_that_promises_nothing_leaves_the_guardians_unchecked"),
    "unbounded-queue-has-no-multiple-consumer-semantics": ("dispatch::message_queue", "the_unbounded_mailbox_promises_an_unbounded_queue_and_a_type_without_promise_a_plain_one"),
    "multiple-consumer-requirement-has-no-default-binding": ("dispatch::mailboxes", "the_built_in_queues_have_the_multiple_consumer_semantics_and_reference_conf_binds_it"),
    "mailbox-undeclared-queue-does-not-keep-the-promise": ("dispatch::mailboxes", "a_queue_that_does_not_declare_its_semantics_keeps_the_promise_of_its_type"),
    "mailbox-actor-requirement-not-verified": ("dispatch::mailboxes", "the_queue_a_selected_type_promises_must_fulfill_the_requirement_of_the_actor"),
    "mailbox-dispatcher-requirement-not-verified": ("dispatch::mailboxes", "the_requirement_of_the_dispatcher_is_verified_and_reported_before_that_of_the_actor"),
    "mailbox-requirement-verified-actor-first": ("dispatch::mailboxes", "the_requirement_of_the_dispatcher_is_verified_and_reported_before_that_of_the_actor"),
    "mailbox-failed-binding-has-no-dispatcher-fallback": ("dispatch::mailboxes", "a_binding_that_fails_verification_gives_way_to_the_requirement_of_the_dispatcher"),
    "mailbox-fallback-binding-is-not-verified": ("dispatch::mailboxes", "a_binding_that_fails_verification_gives_way_to_the_requirement_of_the_dispatcher"),
    "mailbox-selected-id-is-not-verified": ("dispatch::mailboxes", "the_queue_a_selected_type_promises_must_fulfill_the_requirement_of_the_actor"),
    "produced-queue-ignores-the-listed-semantics": ("dispatch::queue_semantics", "a_produced_queue_satisfies_the_semantics_it_lists_and_every_queue_satisfies_message_queue"),
    "produced-queue-satisfies-any-requirement": ("dispatch::queue_semantics", "a_produced_queue_satisfies_the_semantics_it_lists_and_every_queue_satisfies_message_queue"),
    "produced-queue-does-not-satisfy-message-queue": ("dispatch::queue_semantics", "a_produced_queue_satisfies_the_semantics_it_lists_and_every_queue_satisfies_message_queue"),
    "unbounded-mailbox-promises-no-semantics": ("dispatch::message_queue", "the_unbounded_mailbox_promises_an_unbounded_queue_and_a_type_without_promise_a_plain_one"),
    "bounded-mailbox-promises-an-unbounded-queue": ("dispatch::bounded_node_queue", "the_non_blocking_bounded_mailbox_promises_a_bounded_queue"),
    "type-without-promise-claims-an-unbounded-queue": ("dispatch::message_queue", "the_unbounded_mailbox_promises_an_unbounded_queue_and_a_type_without_promise_a_plain_one"),
    "mailbox-dispatcher-any-queue-counts-as-a-requirement": ("dispatch::mailboxes", "a_dispatcher_requiring_any_queue_has_no_requirement"),
    "mailbox-dispatcher-requirement-beats-the-actor": ("dispatch::mailboxes", "the_requirement_of_the_actor_decides_before_the_requirement_of_the_dispatcher"),
    "mailbox-dispatcher-requirement-is-not-the-fallback": ("dispatch::mailboxes", "the_requirement_of_the_dispatcher_serves_an_actor_whose_requirement_has_no_binding"),
    "mailbox-dispatcher-requirement-beats-the-selected-id": ("dispatch::mailboxes", "a_selected_id_decides_before_the_requirement_of_the_dispatcher"),
    "mailbox-requirement-beats-the-selected-id": ("dispatch::mailboxes", "a_selected_id_decides_the_mailbox_over_the_requirement"),
    "deployment-mailbox-ignored": ("dispatch::mailboxes", "a_deployment_with_a_mailbox_overrides_the_props_and_an_empty_or_missing_one_does_not"),
    "deployment-empty-mailbox-resets-the-props": ("dispatch::mailboxes", "a_deployment_with_a_mailbox_overrides_the_props_and_an_empty_or_missing_one_does_not"),
    "props-mailbox-beats-the-deployment": ("dispatch::mailboxes", "a_deployment_with_a_mailbox_overrides_the_props_and_an_empty_or_missing_one_does_not"),
    "system-message-after-close-is-not-dead-lettered": ("actor::unstarted_termination_tests", "a_parent_that_terminates_before_the_supervise_of_its_child_does_not_wait_for_it"),
    "deployer-deploy-swaps-a-stale-snapshot": ("actor::deployer", "a_deployment_that_loses_the_swap_is_retried_on_the_newer_index_and_nothing_is_lost"),
    "init-with-failure-forgets-the-failure": ("actor::init_failure_tests", "a_cell_that_cannot_be_made_is_stopped_by_the_supervisor_and_the_actor_never_starts"),
    "init-with-failure-text-differs": ("actor::init_failure_tests", "a_supervisor_that_restarts_turns_the_failed_creation_into_a_create_and_the_actor_starts"),
    "recreate-of-a-failed-creation-stays-a-recreate": ("actor::init_failure_tests", "a_supervisor_that_restarts_turns_the_failed_creation_into_a_create_and_the_actor_starts"),
    "resume-of-a-failed-creation-stays-a-resume": ("actor::init_failure_tests", "a_supervisor_that_resumes_turns_the_failed_creation_into_a_create_and_the_actor_starts"),
    "fault-create-does-not-wait-for-the-children": ("actor::init_failure_tests", "an_actor_that_fails_to_start_after_creating_a_child_is_restarted_as_a_create_once_its_children_are_gone"),
    "attach-rejection-swallowed-by-start": ("dispatch::executor_rejection_tests", "a_synchronous_actor_of_whose_attach_is_rejected_twice_throws_to_the_caller_and_keeps_the_name"),
    "point-swallows-the-attach-rejection": ("dispatch::executor_rejection_tests", "an_attach_rejected_twice_fails_the_supervisor_and_the_new_actor_never_starts"),
    "supervisor-ignores-a-rejected-point": ("dispatch::executor_rejection_tests", "an_attach_rejected_twice_fails_the_supervisor_and_the_new_actor_never_starts"),
    "sync-actor-of-swallows-the-attach-rejection": ("dispatch::executor_rejection_tests", "a_synchronous_actor_of_whose_attach_is_rejected_twice_throws_to_the_caller_and_keeps_the_name"),
    "sync-actor-of-rejection-releases-the-name": ("dispatch::executor_rejection_tests", "a_synchronous_actor_of_whose_attach_is_rejected_twice_throws_to_the_caller_and_keeps_the_name"),
    "stop-marks-a-child-that-has-not-started": ("actor::unstarted_termination_tests", "a_parent_that_terminates_before_the_supervise_of_its_child_does_not_wait_for_it"),
    "swap-lookup-is-omitted": ("actor::unstarted_termination_tests", "a_child_created_in_pre_start_is_found_through_a_reference_that_is_still_unstarted"),
    "lookup-children-are-always-empty": ("actor::unstarted_termination_tests", "a_child_created_in_pre_start_is_found_through_a_reference_that_is_still_unstarted"),
    "is-started-while-handing-over": ("actor::unstarted_push_timeout_tests", "is_started_is_false_while_the_hand_over_is_in_progress_and_true_after"),
    "push-timeout-not-applied-to-user-messages": ("actor::unstarted_push_timeout_tests", "a_send_that_waits_longer_than_the_timeout_is_a_dead_letter_with_a_warning"),
    "push-timeout-limit-ignored": ("actor::unstarted_push_timeout_tests", "a_send_that_waits_while_the_hand_over_ends_before_the_timeout_is_delivered_after_the_queued_messages"),
    "push-timeout-overflow-panics": ("actor::unstarted_push_timeout_tests", "a_timeout_too_long_for_the_clock_does_not_panic_the_sender"),
    "config-push-timeout-not-read": ("actor::unstarted_push_timeout_tests", "the_configured_unstarted_push_timeout_reaches_the_runtime_settings"),
    "config-negative-push-timeout-is-the-default": ("actor::unstarted_push_timeout_tests", "the_configured_unstarted_push_timeout_reaches_the_runtime_settings"),
    "actor-of-reference-ignores-the-configured-push-timeout": ("actor::unstarted_push_timeout_tests", "an_actor_of_reference_uses_the_configured_unstarted_push_timeout"),
    "unknown-rust-type-is-given-a-jvm-class": ("actor::unstarted_push_timeout_tests", "a_dropped_message_of_a_type_without_a_jvm_class_reports_an_unknown_type_not_an_invented_class"),
    "hand-over-thread-waits-for-itself": ("actor::unstarted_push_timeout_tests", "the_thread_that_hands_the_queue_over_can_send_to_the_same_reference"),
    "lock-timeout-drop-without-warning": ("actor::unstarted_push_timeout_tests", "a_send_that_waits_longer_than_the_timeout_is_a_dead_letter_with_a_warning"),
    "lock-timeout-drop-without-dead-letter": ("actor::unstarted_push_timeout_tests", "a_send_that_waits_longer_than_the_timeout_is_a_dead_letter_with_a_warning"),
    "hand-over-end-does-not-wake-the-senders": ("actor::unstarted_push_timeout_tests", "a_system_message_waits_for_the_lock_without_a_timeout_and_is_delivered_once_the_hand_over_ends"),
    "nonblocking-queue-keeps-one-over-capacity": ("dispatch::bounded_node_queue", "a_full_queue_refuses_at_once_keeps_the_first_and_dead_letters_the_rest_in_order"),
    "nonblocking-queue-drops-the-overflow-silently": ("dispatch::bounded_node_queue", "a_full_queue_refuses_at_once_keeps_the_first_and_dead_letters_the_rest_in_order"),
    "nonblocking-queue-dead-letters-to-the-wrong-receiver": ("dispatch::bounded_node_queue", "a_dead_letter_carries_the_message_the_envelope_sender_and_the_owner_as_receiver"),
    "nonblocking-queue-holds-its-lock-while-dead-lettering": ("dispatch::bounded_node_queue", "a_dead_letters_that_calls_back_into_the_queue_does_not_deadlock_the_producer"),
    "nonblocking-queue-dequeues-the-newest": ("dispatch::bounded_node_queue", "dequeue_frees_a_slot_and_the_order_stays_first_in_first_out"),
    "nonblocking-queue-accepts-a-negative-capacity": ("dispatch::bounded_node_queue", "a_negative_capacity_is_rejected_with_the_pekko_message_and_zero_and_the_maximum_pass"),
    "nonblocking-queue-panics-on-a-poisoned-lock": ("dispatch::bounded_node_queue", "a_producer_that_panicked_under_the_lock_does_not_stop_the_other_threads"),
    "nonblocking-cell-schedules-a-turn-for-a-refused-message": ("actor::nonblocking_mailbox_tests", "a_refused_message_does_not_start_a_consumption_turn"),
    "nonblocking-cell-addresses-a-refusal-to-dead-letters": ("actor::nonblocking_mailbox_tests", "a_refused_message_does_not_start_a_consumption_turn"),
    "nonblocking-cell-addresses-the-cleanup-to-dead-letters": ("actor::nonblocking_mailbox_tests", "stopping_the_actor_dead_letters_what_is_still_queued_addressed_to_it"),
    "nonblocking-queue-reports-a-refused-message-as-stored": ("dispatch::mailbox_close", "a_message_the_queue_refuses_needs_no_consumption_and_is_not_handed_back"),
    "enqueue-unless-closed-ignores-the-queue-answer": ("dispatch::mailbox_close", "a_message_the_queue_refuses_needs_no_consumption_and_is_not_handed_back"),
    "replacement-cell-honours-the-selected-mailbox": ("actor::init_failure_tests", "the_replacement_of_a_cell_that_cannot_be_made_ignores_the_selected_bounded_mailbox"),
    "props-ignores-the-selected-mailbox": ("actor::nonblocking_mailbox_tests", "a_refused_message_does_not_start_a_consumption_turn"),
    "nonblocking-queue-clean-up-moves-only-the-first": ("dispatch::bounded_node_queue", "clean_up_moves_a_full_queue_to_the_target_in_order_and_frees_every_slot"),
    "supervise-is-processed-while-terminating": ("actor::unstarted_termination_tests", "a_parent_already_waiting_for_a_started_child_ignores_the_supervise_of_a_new_child_and_still_terminates"),
    "supervise-is-ignored-while-restarting": ("actor::unstarted_termination_tests", "a_restart_waiting_for_a_started_child_processes_the_supervise_of_an_unstarted_child_meanwhile"),
}


def run_unit_mutant(scratch, name, work):
    """Runs the unit tests of the module (or, as `test:<file>`, the integration test file) on the mutated copy. DETECTED needs the expected test to fail (a compile error or
    some other failing test is not enough) and the same tests to pass on the unmutated tree (the control)."""
    module, expected_test = UNIT_MUTANTS[name]
    def cargo_test(manifest, target):
        return sh(hermetic(["cargo", "test", "--manifest-path", manifest, "--target-dir", target, *(["--test", module[5:]] if module.startswith("test:") else ["--lib", module])]), timeout=BUILD_TIMEOUT)
    try:
        control = cargo_test(os.path.join(ROOT, "crates", "rakka-actor", "Cargo.toml"), os.path.join(scratch, "target-control-" + name))
        r = cargo_test(os.path.join(work, "Cargo.toml"), os.path.join(scratch, "target-" + name))
    except subprocess.TimeoutExpired:
        return TIMEOUT, None, None, "cargo test"
    if control.returncode != 0:
        return HARNESS_ERROR, None, None, "the unit tests fail on the unmutated tree"
    summary = next((l for l in r.stdout.splitlines() if l.startswith("test result:")), "(no test result)")
    kind, detail = classify_cargo_failure(r) if r.returncode != 0 and "test result:" not in r.stdout else (None, None)
    if kind is not None:
        return kind, None, None, detail
    if r.returncode == 0:
        return UNDETECTED, None, None, summary
    if "test result: FAILED" in r.stdout and f"{expected_test} ... FAILED" in r.stdout:
        return DETECTED, None, None, summary
    return HARNESS_ERROR, None, None, "failed, but not the expected test: " + summary


def prepare_scratch_workspace(ws):
    """A scratch workspace holds the copied crate (it inherits version/edition from the workspace) and the authoritative Cargo.lock. -> (crate directory, None) or (None, why)."""
    work = os.path.join(ws, "rakka-actor")
    authoritative_lock = os.path.join(ROOT, "Cargo.lock")
    if not os.path.exists(authoritative_lock):
        return None, "the authoritative Cargo.lock is missing: no mutant is built without it"
    shutil.copytree(CRATE, work, ignore=shutil.ignore_patterns("target"))
    shutil.copy(authoritative_lock, os.path.join(ws, "Cargo.lock"))      # the same resolution as the real tree, never a fresh one
    shutil.copy(os.path.join(ROOT, "Cargo.toml"), os.path.join(ws, "Cargo.toml"))
    rootm = open(os.path.join(ws, "Cargo.toml")).read()
    rootm = re.sub(r'(?s)^.*?\[workspace\]', '[workspace]', rootm)
    rootm = rootm.replace('members = [".", "crates/rakka-actor", "crates/rakka-actor-typed"]', 'members = ["rakka-actor"]')
    open(os.path.join(ws, "Cargo.toml"), "w").write(rootm)
    return work, None


def run_mutant(scratch, name, rel, old, new, why):
    work, problem = prepare_scratch_workspace(os.path.join(scratch, name))
    if work is None:
        return HARNESS_ERROR, None, None, problem
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
        # the in-crate twins read the oracle golden through a relative `include_str!` that leaves the crate: give the scratch tree the same layout
        golden = os.path.join(scratch, "cases", "oracle-only")
        os.makedirs(golden, exist_ok=True)
        for golden_file in ("mailbox.golden", "nonblocking-bounded-mailbox.golden"):
            shutil.copy(os.path.join(ROOT, "cases", "oracle-only", golden_file), golden)
        return run_unit_mutant(scratch, name, work)
    try:
        b = sh(hermetic(["cargo", "build", "--release", "--manifest-path", os.path.join(work, "Cargo.toml"),
                         "--target-dir", os.path.join(scratch, "target-" + name), "--example", "classic-parity"]), timeout=BUILD_TIMEOUT)
    except subprocess.TimeoutExpired:
        return TIMEOUT, None, None, "cargo build"
    if b.returncode != 0:
        kind, detail = classify_cargo_failure(b)
        return kind, None, None, detail
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
        replay_result = replay(binary)
    except subprocess.TimeoutExpired:
        return TIMEOUT, None, None, "rust-anything replay"
    except replay_lib.ReplayLockTimeout:
        return BUDGET_EXHAUSTED, None, None, BUDGET_EXHAUSTED_LINE
    evaluation = replay_lib.evaluate(binary, replay_result)
    outcome = classify_evaluation(evaluation)
    # the report of THIS replay (read under the checkout lock), never the shared file as it is now; a verdict always rewrites it, so that an
    # earlier report can never sit under a verdict path that no longer applies
    scope = "output rejected" if replay_result.passed is None else f"{replay_result.total} cases"
    publish_mutant_report(name, f"full replay ({scope}); outcome {outcome[0]}",
                          replay_result.report_text if replay_result.report_text is not None else "(this replay produced no report)\n")
    # the history is seeded only by a confirmed detection and holds only the cases that count (not NOT_APPLICABLE, not NA:Q-CFG-30); a survivor has none; a replay that
    # could not be understood (an infrastructure failure) leaves the history as it was
    if outcome[0] in (DETECTED, UNDETECTED):
        confirmed = evaluation["applicable_failing"] if outcome[0] == DETECTED else []
        write_killers(name, confirmed)
        record_provenance(name, confirmed, replay_result.total)
    return outcome


FAST_KILL = True
FAST_KILL_CANDIDATES = 8


def killers_file(name):
    return os.path.join(REPORTS, f"replay-killers-{name}.txt")


def write_killers(name, cases):
    """The divergent cases of the last FULL replay of this mutant, one per line: the candidates of its next fast-kill. A fast-kill never shrinks it.
    Sorted and without duplicates, so that the same detection always writes the same bytes (a file that can be versioned and diffed); only case names, never a path."""
    with open(killers_file(name), "w", encoding="utf-8") as killers:
        killers.write("".join(f"{case}\n" for case in sorted(set(cases))))


PROVENANCE_HEADER = "mutant\tpatch\tcases\tkillers\tcrate_tree\tcorpus\n"


def provenance_file():
    return os.path.join(REPORTS, "killers-provenance.tsv")


def patch_hash(name):
    """16 hex digits that identify the DEFINITION of a mutant (file, pattern, replacement): a history recorded for another definition is not about this mutant. None: not defined."""
    definition = MUTANTS.get(name) or TOOL_MUTANTS.get(name) or SELFTEST.get(name)
    if definition is None:
        return None
    return hashlib.sha256("\0".join(definition[:3]).encode("utf-8")).hexdigest()[:16]


def crate_tree():
    """The git tree id of the code under test (traceability of a recorded history); `unknown` when git cannot tell. No path, no time."""
    done = subprocess.run(["git", "-C", ROOT, "rev-parse", "HEAD:crates/rakka-actor"], capture_output=True, text=True)
    return done.stdout.strip() if done.returncode == 0 and done.stdout.strip() else "unknown"


def read_provenance():
    try:
        with open(provenance_file(), encoding="utf-8") as manifest:
            rows = [line.rstrip("\n").split("\t") for line in manifest.readlines()[1:] if line.strip()]
    except OSError:
        return {}
    return {row[0]: row for row in rows if len(row) == 6}


def write_provenance(rows):
    """Sorted by mutant and rewritten whole: the same detections give the same bytes. Written aside and renamed, so that a reader never sees half a manifest."""
    body = PROVENANCE_HEADER + "".join("\t".join(rows[name]) + "\n" for name in sorted(rows))
    temporary = provenance_file() + ".tmp"
    with open(temporary, "w", encoding="utf-8") as manifest:
        manifest.write(body)
    os.replace(temporary, provenance_file())


def record_provenance(name, cases, corpus):
    """After a CONFIRMED detection: what the history of this mutant is about. `cases` empty (a survivor, or nothing that counts) removes the entry."""
    rows = read_provenance()
    if cases and patch_hash(name):
        with open(killers_file(name), "rb") as killers:
            digest = hashlib.sha256(killers.read()).hexdigest()[:16]
        rows[name] = [name, patch_hash(name), str(len(set(cases))), digest, crate_tree(), str(corpus)]
    else:
        rows.pop(name, None)
    write_provenance(rows)


def cases_listed_in_report(name):
    try:
        with open(os.path.join(REPORTS, f"replay-report-{name}.md"), encoding="utf-8") as report_file:
            return re.findall(r"(?m)^### (\S+)\s*$", report_file.read())
    except OSError:
        return []


def preserve_killers(name):
    """Before a fast-kill replaces the report of a mutant with its own single-case evidence, keep the divergent cases that report listed."""
    if not os.path.exists(killers_file(name)) and cases_listed_in_report(name):
        write_killers(name, cases_listed_in_report(name))


def known_killers(name):
    """The divergent cases an earlier full replay of this mutant recorded (the killers file; the report of older runs as a fallback), minus the
    NOT_APPLICABLE ones."""
    try:
        with open(killers_file(name), encoding="utf-8") as killers:
            listed = [line.strip() for line in killers if line.strip()]
    except OSError:
        listed = cases_listed_in_report(name)
    recorded = read_provenance().get(name)
    if recorded is not None and recorded[1] != patch_hash(name):
        return []        # recorded for another definition of the mutant (or one that no longer exists): more work, never another verdict
    not_applicable = replay_lib.load_not_applicable()
    return [case for case in listed if case not in not_applicable][:FAST_KILL_CANDIDATES]


_control_binary = None


def release_control_binary():
    """Ends the process's use of the control copy (also done at exit); a later call to control_binary_for_fast_kill() publishes it again."""
    global _control_binary
    if _control_binary:
        replay_lib.release_lease(_control_binary)
    _control_binary = None


atexit.register(release_control_binary)


def control_binary_for_fast_kill():
    """The unmutated parity binary of the current tree (built once per process), or None when it cannot be built."""
    global _control_binary
    if _control_binary is None:
        path = os.path.join(ROOT, "target", PARITY)
        try:
            with replay_lib.build_lock():   # build and publish are one step: no other build can swap target/ in between
                built = sh(list(clean_replay_evidence.CLEAN_REPLAY_COMMAND), timeout=BUILD_TIMEOUT, cwd=ROOT)
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
        # a blank line of a quoted diff is `+ ` or `- `: drop only that prefix-only trailing space, so a divergence that is itself whitespace stays visible
        text = f"Verdict path: {verdict_path}\n\n{report_text}"
        report_file.write(re.sub(r"(?m)^([+-]) $", r"\1", text))


def publish_fast_kill_report(name, kill):
    preserve_killers(name)
    publish_mutant_report(name, f"fast-kill: case {kill.case} confirmed by a single-case replay (`rust-anything app replay --case`); "
                                f"{kill.reason}; no full replay ran", kill.report_text)


MIRROR_NOT_LINKED = {".git", ".gates", "target", "upstream"}


def tool_mutant_workspace(scratch, name):
    """The tests of tools/parity find their repository from their own location (ROOT three levels up, the sibling tools/gates). A tool mutant therefore runs in a
    mirror of the repository: every entry of it is linked, except tools/parity, which is the copy that is mutated. -> the copy."""
    mirror = os.path.join(scratch, name)
    os.makedirs(os.path.join(mirror, "tools"))
    for entry in os.listdir(ROOT):
        if entry != "tools" and entry not in MIRROR_NOT_LINKED:
            os.symlink(os.path.join(ROOT, entry), os.path.join(mirror, entry))
    for entry in os.listdir(os.path.join(ROOT, "tools")):
        if entry != "parity":
            os.symlink(os.path.join(ROOT, "tools", entry), os.path.join(mirror, "tools", entry))
    work = os.path.join(mirror, "tools", "parity")
    shutil.copytree(TOOLS, work, ignore=shutil.ignore_patterns("__pycache__"))
    return work


def run_tool_mutant(scratch, name, rel, old, new, why):
    """A mutant of tools/parity itself: a copy in a mirror of the repository is patched and the unit tests of the tools must fail."""
    work = tool_mutant_workspace(scratch, name)
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
# Mutants retired from the active set. A retirement is not hiding an error: it needs (1) a pattern that is really obsolete (it no longer occurs in
# the crate), (2) the retired behaviour still covered by a live mutant that kills every recorded case that killed it, and (3) the Owner's
# authorisation; the history (the old report under rust/divergence-reports/) stays. tools/gates/test_mutant_patterns.py enforces (1) and (2): a
# pattern that applies again, or a covering mutant that stops covering, makes the retirement invalid. Any OTHER stale pattern is still an ERROR.
RETIRED_MUTANTS = {
    "mailbox-is-lifo": {
        "file": "src/actor/cell.rs",
        "old": "                } else {\n                    mb.user.pop_front()\n",
        "new": "                } else {\n                    mb.user.pop_back()\n",
        "why": "the mailbox hands out the newest message first (breaks per-sender ordering)",
        "covered_by": "mailbox-queue-is-lifo",
        "retired_on": "2026-11-21",
        "authorized_by": "Owner (Q-X-20 instruction, 2026-11-21)",
        "evidence": "the code it patched (`mb.user.pop_front()` in cell.rs) was removed by 3d85ae6 (#302): the user queue now lives in dispatch/message_queue.rs, "
                    "where `mailbox-queue-is-lifo` mutates the same behaviour; all 97 recorded cases that failed the old mutant fail the live one (102 cases); "
                    "history: rust/divergence-reports/replay-report-mailbox-is-lifo.md, docs/evidence/pilot/REPORT-3b.md",
    },
}

SELFTEST = {"selftest-equivalent-whitespace": (
    "src/actor/supervision.rs", "self.count <= max_nr_of_retries", "self.count <= max_nr_of_retries /* same */",
    "self-test of the launcher: an equivalent edit that must NOT be detected")}


def check_patterns():
    """`--check-patterns`: every mutant's pattern must occur exactly once (stale patterns after a refactor exit 2)."""
    bad = 0
    def occurrences(directory, relative_file, pattern):
        with open(os.path.join(directory, relative_file), encoding="utf-8") as source_file:
            return source_file.read().count(pattern)

    for name, (relative_file, old, new, _why) in {**MUTANTS, **SELFTEST}.items():
        found = occurrences(CRATE, relative_file, old)
        if found != 1:
            bad += 1
            print(f"STALE {name}: {found} occurrences in {relative_file}")
    for name, (relative_file, old, new, _why) in TOOL_MUTANTS.items():
        found = occurrences(TOOLS, relative_file, old)
        if found != 1 or old == new:
            bad += 1
            print(f"STALE {name}: {found} occurrences in tools/parity/{relative_file}")
    for name, entry in RETIRED_MUTANTS.items():
        print(f"RETIRED {name}: covered by {entry['covered_by']} (retired {entry['retired_on']}, {entry['authorized_by']}); history kept")
    print(f"{len(MUTANTS) + len(SELFTEST) + len(TOOL_MUTANTS)} mutant patterns checked, {bad} stale, {len(RETIRED_MUTANTS)} retired")
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
    path = tempfile.mkdtemp(prefix=f"pekko-mutants-{os.getpid()}-", dir=parent)     # the owner is in the name from the instant it exists
    with open(os.path.join(path, SCRATCH_OWNER_MARKER), "w") as owner_file:
        owner_file.write(str(os.getpid()))
    return path


def reusable_clean_replay(control_binary):
    """The evidence record of this gate run's complete clean replay of exactly this binary and these inputs, else None (the control then replays)."""
    ev = clean_replay_evidence
    stored, why = ev.reusable(ev.evidence_path(ROOT), ev.identity(ROOT, control_binary, os.environ), os.environ.get(ev.RUN_ID_ENV), ev.corpus_case_count(ROOT))
    if stored is None:
        print(f"clean-replay evidence not reused ({why}): the control replays", flush=True)
    return stored


def run_control_replay():
    """Replays the unmodified crate: builds the parity binary of the CURRENT tree first (a leftover binary would certify old code).
    -> (control_ok, passed, total, line, replay result or None)."""
    control_binary = control_binary_for_fast_kill()
    if control_binary is None:
        return False, None, None, "could not build the unmodified parity binary", None
    reused = reusable_clean_replay(control_binary)
    if reused is not None:
        r = reused["result"]
        return True, r["applicable_passed"], r["applicable_total"], (f"reused: the clean replay of this gate run ({r['applicable_passed']}/{r['applicable_total']} applicable + "
                                                                     f"{r['other_not_applicable']} not applicable; evidence {reused['key'][:12]}, same binary, inputs and toolchain)"), None
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
    control_passed = control_total = None
    for n in only:
        assert n in mutants, f"unknown mutant {n}"
    tool_only = bool(only) and all(n in TOOL_MUTANTS for n in only)
    os.makedirs(REPORTS, exist_ok=True)
    scratch = make_scratch_directory()
    results = {}
    try:
        if not control_only and not tool_only:
            provisioned, provisioning_problem = provision_dependencies()
            if not provisioned:   # the builds below are offline: each mutant then reports the infrastructure error itself
                print(f"dependency provisioning failed: {provisioning_problem}", flush=True)
        for name, (rel, old, new, why) in mutants.items():
            if control_only or (only and name not in only):
                continue
            print(f"=== mutant {name}: {why}", flush=True)
            res = (run_tool_mutant if name in TOOL_MUTANTS else run_mutant)(scratch, name, rel, old, new, why)
            results[name] = res
            print(f"{res[0]}  {res[3]}\n", flush=True)
        control_replay_result = None
        if skip_control:   # the caller (tools/gates) runs the control once for a whole batch
            control_ok, control_passed, control_total, control_line = True, None, None, "skipped (--skip-control)"
        elif tool_only:   # the tool mutants need no replay: the control would only repeat the last full run
            control_ok, control_passed, control_total, control_line = True, None, None, "skipped (tool mutants only)"
        else:
            control_ok, control_passed, control_total, control_line, control_replay_result = run_control_replay()
        print(f"=== control (unmodified rakka-actor): exit={control_replay_result.returncode if control_replay_result is not None and control_passed is not None else '?'}  {control_line}", flush=True)
    finally:
        shutil.rmtree(scratch, ignore_errors=True)
    # a replay of a different corpus size would make the pass counts incomparable
    sizes = {t for (_o, _p, t, _l) in results.values() if t is not None} | ({control_total} if control_total else set())
    rows = ["| mutant | outcome | passed/total |", "|---|---|---|"]
    rows += [f"| {n} | {o} | {f'{p}/{t}' if p is not None else '-'} |" for n, (o, p, t, _l) in results.items()]
    rows.append(f"| control (unmodified) | {'CLEAN' if control_ok else 'FAILED'} | {f'{control_passed}/{control_total}' if control_passed is not None else '-'} |")
    table = "\n".join(rows)
    counts = {k: sum(1 for r in results.values() if r[0] == k) for k in (DETECTED, UNDETECTED, PATCH_ERROR, BUILD_ERROR, HARNESS_ERROR, TIMEOUT)}
    print("\n" + table + f"\n\ncorpus sizes seen: {sorted(sizes)}  outcomes: {counts}  detected {counts[DETECTED]}/{len(results)}")
    if result_json:
        with open(result_json, "w") as result_file:
            json.dump({"mutants": {name: list(result) for name, result in results.items()}, "control": None if skip_control else control_ok}, result_file)
    if not selftest and not only and not skip_control and not control_only:
        with open(os.path.join(REPORTS, "SUMMARY.md"), "w") as summary_file:
            summary_file.write(table + f"\n\noutcomes: {counts}\n")
    budget_exhausted = any(result[0] == BUDGET_EXHAUSTED for result in results.values()) or control_line == BUDGET_EXHAUSTED_LINE
    sys.exit(final_exit_code(counts, sizes, control_ok, control_passed, budget_exhausted))


if __name__ == "__main__":
    main()
