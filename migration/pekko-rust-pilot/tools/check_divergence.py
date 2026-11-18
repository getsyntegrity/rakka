#!/usr/bin/env python3
"""Proves the recorded replay can fail: replays deliberately broken copies of the Rust port.

For each mutant it copies pekko-actor/ into a scratch dir (the real sources are never touched),
applies one source edit, builds it, runs `rust-anything app replay --build <mutant binary>`, and
requires a non-zero exit with at least one failing case. The per-mutant replay reports are kept in
rust/divergence-reports/. Finally the unmodified port is replayed again as the control.

  tools/check_divergence.py [mutant ...] [--selftest]

Exit code: 0 every mutant DETECTED and the control clean; 1 a mutant went UNDETECTED or the control failed;
2 the experiment is invalid (patch, build, harness error or timeout). `--selftest` runs an equivalent mutant that must exit 1.
"""
import difflib, os, re, shutil, subprocess, sys, tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CRATE = os.path.join(ROOT, "pekko-actor")
PARITY = os.path.join("release", "examples", "classic-parity")
REPORTS = os.path.join(ROOT, "rust", "divergence-reports")
CASES_REPORT = os.path.join(ROOT, "rust", "app-cases", "replay-report.md")

# name -> (file, old, new, why it is a realistic bug)
MUTANTS = {
    "utf8-lossy-decoder": (
        "src/identity/jdk_uri.rs",
        "out.extend(decode_utf8_jdk(&bytes));",
        "out.extend(String::from_utf8_lossy(&bytes).encode_utf16());",
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
        "if mb.closed || mb.suspend > 0 { None } else { mb.user.pop_front() }",
        "if mb.closed || mb.suspend > 0 { None } else { mb.user.pop_back() }",
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
        "src/identity/actor_path.rs",
        "if self.uid != 0 {\n            s.push('#' as u16);",
        "if false {\n            s.push('#' as u16);",
        "toSerializationFormat forgets the #uid suffix",
    ),
    # --- slice 3a: hierarchy identity and DeathWatch ---
    "ref-identity-is-path": (
        "src/identity/actor_ref.rs",
        "a.uid == b.uid && a.address == b.address && a.elements == b.elements",
        "a.address == b.address && a.elements == b.elements",
        "two references are equal when their paths are equal (the uid is ignored), so a new actor with the old path "
        "is taken for the old one",
    ),
    "children-match-by-path": (
        "src/actor/children.rs",
        "Some(ChildStats::Child(c)) if c == actor => Some(c.clone()),",
        "Some(ChildStats::Child(c)) if c.path().elements == actor.path().elements => Some(c.clone()),",
        "the parent recognises a child by path only: a stale reference is treated as the current child",
    ),
    "watch-self-check-by-path": (
        "src/actor/cell.rs",
        "        if *subject == me {\n            return;",
        "        if subject.path().elements == me.path().elements {\n            return;",
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
        '                k.send_system(SystemMessage::Recreate { cause: cause.clone() });',
        '                k.send_system(SystemMessage::Resume { cause: Some(cause.clone()) });',
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
        'restart_stats.entry(k.clone())',
        'restart_stats.entry(self.self_ref())',
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
        'let own = if Some(&k) == perp.as_ref() { cause.clone() } else { None };',
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
    return sh(["rust-anything", "app", "replay", ROOT, "--build", binary], timeout=REPLAY_TIMEOUT)


def classify(r):
    """-> (outcome, passed, total, line). A mutant is detected only when the replay ran to completion, printed its
    `N/M cases passed` line, exited non-zero and at least one case diverged (N < M)."""
    m = SUMMARY.search(r.stdout)
    if not m:
        return HARNESS_ERROR, None, None, (r.stdout.strip() or r.stderr.strip())[-300:]
    passed, total = int(m.group(1)), int(m.group(2))
    if r.returncode == 0 and passed == total:
        return UNDETECTED, passed, total, m.group(0)
    if r.returncode != 0 and passed < total:
        return DETECTED, passed, total, m.group(0)
    return HARNESS_ERROR, passed, total, f"exit={r.returncode} but {m.group(0)}"


def run_mutant(scratch, name, rel, old, new, why):
    work = os.path.join(scratch, name)
    shutil.copytree(CRATE, work, ignore=shutil.ignore_patterns("target"))
    path = os.path.join(work, rel)
    src = open(path).read()
    if src.count(old) != 1:
        return PATCH_ERROR, None, None, f"pattern occurs {src.count(old)} times in {rel}"
    patched = src.replace(old, new)
    open(path, "w").write(patched)
    if patched == src and name not in EQUIVALENT:
        return PATCH_ERROR, None, None, "patch did not change the file"
    print("".join(difflib.unified_diff(src.splitlines(True), patched.splitlines(True), f"a/pekko-actor/{rel}", f"b/pekko-actor/{rel}")))
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
    return classify(r)


# `--selftest` adds a mutant that changes nothing observable (a whitespace edit): it MUST come out UNDETECTED, so the run must exit non-zero.
EQUIVALENT = {"selftest-equivalent-whitespace"}
SELFTEST = {"selftest-equivalent-whitespace": (
    "src/actor/supervision.rs", "self.count <= max_nr_of_retries", "self.count <= max_nr_of_retries /* same */",
    "self-test of the launcher: an equivalent edit that must NOT be detected")}


def main():
    args = sys.argv[1:]
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
            c = replay(os.path.join(ROOT, "target", PARITY))
            c_out, c_pass, c_total, c_line = classify(c)
            control_ok = c.returncode == 0 and c_pass is not None and c_pass == c_total
        except subprocess.TimeoutExpired:
            control_ok, c_pass, c_total, c_line = False, None, None, "TIMEOUT"
        print(f"=== control (unmodified pekko-actor): exit={c.returncode if c_pass is not None else '?'}  {c_line}", flush=True)
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
