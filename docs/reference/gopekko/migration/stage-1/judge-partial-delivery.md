# Judge, partial delivery: what a Go pilot can rely on

Status: **partial**. Gate A is open. 00b is **not** closed and nothing here claims equivalence with goPekko.
This document is the input to one decision: which subset of the judge a Go pilot may start from, and which
features stay out until the judge covers them.

## 1. What "observed" means here

A clause is **observed** when a mutation designed to break it was run on the pinned Pekko (`v1.7.1`,
`9a54d1294c`), the comparator reported that clause, and the execution of that run is proved (section 5).
It means **the judge can detect a violation of the clause on Pekko**. It does not mean goPekko satisfies
the clause: goPekko has no runtime or Go harness yet, so goPekko-side observation is 0 of 35 relations.

Source of every number below: `python3 -I scripts/check-judge-matrix.py --clauses` on this branch
(`judge/mutation-report.tsv`, 50 rows). 46 rows are backed by an entry of `judge/backfill.tsv`; the four `M6` rows
are not: they are `run_evidence=executed`, recorded by the runner at `136eef1` (see section 5).

Three facts, reported separately by the checker. None stands in for another:

| Fact | Value |
|---|---|
| **Designed** coverage (relation has a scenario and a mutation) | 35 / 35 relations. It is a plan: it holds no result |
| **Observed** detection: clauses with at least one designed mutation caught **by that clause** in a valid campaign | **26 / 28** |
| unknown (an unreliable run decides nothing) | 0 |
| pending | 2: `G8.a`, `I3.b` |
| Mutations detected by the rules they were designed for | **24 / 24** implemented Pekko mutations |
| **Execution reliability**: report rows that are not a result (`timeout`) | 8 of 50. Not counted as detection or survival (section 4) |
| Violations seen in campaigns that are not a valid result | none (the checker prints them as observations when they exist) |

Recomputed with the corrected rules (a campaign with any harness-error or timeout run is never `failed`, and a log set
must be exactly the requested run identifiers): **the result is unchanged, 26 / 28**. When this was done, every one of the
50 rows that is a result kept its outcome when recomputed from its own counts, and no row mixed failing runs with invalid
ones, so no Pekko campaign needed a rerun. That is a **one-off audit of the report as it was**, recorded here as evidence;
it is not a test, and no CI check asserts it, because a later legitimate row (a mixed campaign, or a `harness-error` from
a crashed sbt that left every log passing) must not turn a test of the code red. The counts of a row do not say whether
the process died: that is a fact of the process (its exit status), and `judge/selftest_runner.py` tests it independently.

**A second defect of the same kind, found in review, and its measured reach.** A run is counted by the state of its
execution, not by what its log shows. The comparator returns both (`verdict` and `state`), and a run that ended in
timeout whose prefix already proves a violation comes back as `fail` with state `timeout`; `evaluate.py` used to count
only the verdict, so such a run counted as a `fail` and could make a campaign `failed`. It is fixed (`run_class`): that run
counts as a `timeout`, and the rules stay in `fired_rules` as an observation. To know what the defect had touched, the
preserved logs of the report were re-judged with the comparator, read-only, running no Pekko campaign: of the 50 rows,
the 46 whose logs are preserved in `gate-a-review-evidence-20261008` contain **no** run with verdict `fail` and state
`timeout` (the eight `timeout` rows are `timeout` by verdict as well), so **no committed result changes**. The other four
rows (`M6` on `S10` and `S11`, run at harness `136eef1`, evidence `executed`) have no preserved logs in that set, so they
were not re-judged. What can be said of them comes from the comparator's code, not from their logs. The `S11` row that is
`failed` fired `REQ`, which the comparator reports only on a complete log (in `comparator.evaluate`, the `required` events
are checked only `if done`), so it cannot come from a run that ended in timeout. The `S10` row that is `failed` fired only `I3.a`:
on a timeout prefix `r_I3a` can report only a duplicate (`n > 1`; it reports `n < 1` only `if done`), and `M6`, which
drops the publication, cannot produce a duplicate. Both are inferences from code, stated as such; the logs of those four
rows remain unavailable, and the two `timeout` and `survived` rows of `M6` are not `failed` rows, so this defect cannot
have promoted them.

An unmet expectation of one mutation never erases the detection another mutation gave a clause. `I3.a` is the case:
`M6` caught it in controlled mode (`M6/S10` and `M6/S11`), and that evidence stands whatever `M7` fires.

Relations with every clause observed: 12 of 14 (G/I tier). The two that are not: `G8` (needs `G8.a`) and `I3` (needs `I3.b`).

## 2. Subset a Go pilot can start from

The 26 observed clauses are the judge's reliable surface. They are split by the mode in which their detection
was actually observed, because the two modes do not have the same standing today.

### 2.1 Usable now: detection observed in controlled mode (24 clauses)

Controlled mode is deterministic (one run), so it does not depend on a run count.

| Feature a pilot can exercise | Clauses | Detecting mutations (controlled) |
|---|---|---|
| Child creation is ordered before the first message; hooks never overlap `Receive` | `G2.a` `G3.a` `G3.b` | M16, M17 |
| Restart: hook order, failing message not redelivered, queued messages kept | `G4.a` `G4.b` `G4.c` | M13, M41, M3, M40 |
| Stop: `post-stop` runs, children before parent, once per instance | `G5.a` `G5.c` `G5.d` | M2, M17 |
| Death watch: `terminated` on stop, on an already-stopped subject, none after `unwatch`, none on restart | `G6.a` `G6.b` `G6.c` `G6.d` | M58, M11, M10, M12 |
| Default supervision decider (restart vs stop) | `G7.a` `G7.b` | M13, M14 |
| Ask: timeout failure | `G8.b` | M36 |
| `terminated` ordering after the subject's messages and `post-stop`; exactly once | `I1.a` `I1.b` `I2.a` | M58, M9, M5 |
| A message to a terminated actor, or queued at its stop, is dead-lettered once (controlled mode only) | `I3.a` | M6 |
| Ask to an already-terminated actor fails at once | `I4.a` | M15 |
| A reply after the ask completed becomes a dead letter | `I5.a` | M38 |
| Conflicting `watch`/`watchWith` fails the watcher; the same message is a no-op | `I6.a` `I6.b` | M43, M59 |

### 2.2 Observed only in concurrent mode: usable as detectors, not as a gate (2 clauses)

| Clause | Feature | Detecting mutation |
|---|---|---|
| `G1.a` | FIFO between one sender and one receiver | M1 |
| `G5.b` | Stop: queued messages are not received after the stop | M7 |

Their controlled-mode run of the same mutation **survived** (`M1/S01` controlled: pass), so the violation shows
only under real concurrency. `judge/calibration.tsv` is all-pending, so there is no justified `N`
(`docs/judge.md` section 5). The fixed 30 runs are enough to see a detection, but they do not justify
"zero violations in `N` runs" on the Go side. A pilot may run these two as detectors; it may not use a green
result as evidence until the calibration exists.

## 3. Excluded for lack of coverage

| Excluded | Why | What lifts it |
|---|---|---|
| `G8.a`: the ask reply payload is the first reply | `M37` has no patch (the ask result is a `Promise` completed once, so a second reply cannot overwrite it in Pekko) | A mutation that can break it, or a documented decision that it is covered by a goPekko-only mutation |
| `I3.b`: a message is never both received and dead-lettered, and never dead-lettered twice | No mutation is designed for it, so there is no witness. `M7` cannot be one (section 6.1): it receives the message, so nothing is left to dead-letter | A mutation whose run contains a dead-letter event for a message that was also received, or two for one message (section 6.1) |
| All 21 S-tier relations: `P1-P4` provenance, `F1-F2` forward, `A1`, `C1-C2` metadata, `H1`, `K1`, `X1`, `T1-T5` tenancy, `O1-O4` observers | Own specification of goPekko: there is no Pekko oracle. Detection rests on goPekko mutations (`M18` onward), which need the runtime | The goPekko runtime and harness (milestone 1) |
| `N1-N3` | Not guaranteed by Pekko: never asserted by design | Nothing: allowed orders must pass |
| Anything outside the contract of `docs/judge.md` section 3: non-default mailboxes, remote and cluster actors, persistence | The I-tier is verified for local actors with the default FIFO mailbox only | A new contract section and mutations |

A pilot that touches an excluded feature has **no judge verdict** for it, not a passing one.

## 4. Open items that bound this delivery

- `judge/calibration.tsv`: all rows pending. No concurrent run count is justified (section 2.2).
- Eight report rows are `timeout` (`M2`, `M6`, `M11`, `M13`, `M14`, `M40` x2, `M58`). They are not evidence and are
  not counted. None of them leaves a clause unknown, because each affected clause has another valid detection.
  They are redundancy to recover, not coverage to claim.
- `I3.a` is observed in controlled mode only and is **not** extended to concurrent mode: `docs/judge.md` makes it
  strict only there, `M6/S10` concurrent survived and `M6/S11` concurrent timed out. Neither row is evidence. A rerun
  of `M6/S11` concurrent would not change the count and is not requested.
- No rerun moves a pending clause: `G8.a` needs `M37`, `I3.b` needs a new mutation (section 6.1). Neither is part of this delivery.
- Provenance of `50e60c1`: 23 report rows re-evaluate logs produced at harness commit `50e60c1`, which is not in this
  PR's history. It is preserved on `origin/judge/gate-a-comparator` and the tag `evidence/gate-a-comparator-107b999` (`107b999`, published for this purpose only: eight
  judge/ and scripts/ commits, no secrets or absolute paths, no change outside those paths). A full clone verifies it
  (checked on a fresh clone of this branch); CI checks out shallow and skips that probe.
- Native review of `87b1f10..17843e8` (11 paths, 894 lines, high risk, four lenses): **approved**, no blocking finding.
  Advisory findings left open on purpose (no judge expansion in this delivery): `executed` is a self-asserted
  `run_evidence` (the four `M6` rows); `backfill.py` raises on a missing original `sbt.log` instead of leaving the row
  `unknown`; `reeval` does not compare log hashes with the ledger entry; `unreliable` is keyed by mutation, not by
  scenario; `backfill.py` duplicates run-count constants. Three defects in this unit's own text and tests were fixed after the
  review (the ledger claim above, a self-test precedence bug, a status word list); those edits are not covered by it.
- The review of the comparator and harness units is open: references A `6551202` and B `87b1f10` remain the
  current ones until the correction lands. Their review evidence is preserved in
  `gate-a-review-evidence-20261008`; no approval is transferred to this branch.

## 5. Why the numbers can be trusted

- A result needs a proved execution. `judge/backfill.py` accepts a row only with: the recorded launch command
  (found by tool-use id in the session transcript, with `JUDGE_RUNS=30` and the work directory), a runner that
  exports `JUDGE_RUNS`, original logs written inside the run's time window, a clean `sbt.log`, a finished-mutation
  line, and byte-identical preserved copies listed in `SHA256SUMS`. "There are 30 logs" is never the proof.
- `python3 -I judge/backfill.py verify ...` reproduces 50 of 50 ledger entries from the evidence.
  That covers the ledger, not the four `M6` rows: ledger entries `B043`-`B046` describe the earlier `M6` campaign
  (runner `931e6df`) and no report row cites them. The `M6` rows (and with them the controlled-mode detection of
  `I3.a`) rest on `executed`, the runner's own record of a run; nothing ties them to a recorded launch command or
  `sbt.log` hash. This is a weaker proof than the ledger, stated here rather than hidden.
- Limit: the runners did not record the process exit status. A clean `sbt.log` is its proxy, and every ledger entry
  says so in `reason`.
- Without a ledger entry a re-evaluated row is `unknown`; violations seen in its logs remain visible as
  observations and are never counted as detection.
- Tests: `judge/selftest.py` 131, `judge/selftest_runner.py` 67, `judge/selftest_backfill.py` 51 checks, all passing. CI runs
  the three in the `judge-selftests` job, which `ci-ok` requires (`docs/ci.md`).
- `judge/evaluate.py` judges the logs that exist. Alone it proves no campaign: without `--expect-runs N` it exits 3
  when every available log passed, never 0. With `--expect-runs N` it also requires exactly the identifiers `r1..rN`
  (controlled: its one log) and names the missing, duplicate and surplus ones. The runner, `reeval` and `partial.py`
  apply the same identifier check, so the right number of logs with the wrong identifiers is `incomplete`.

## 6. Provenance of this branch

- Pekko pin: `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`.
- This unit is a stack on B (`87b1f10`) on `judge/c-reliability-on-b`: `bbe9592` (evidence rules), `136eef1` (this
  document), `569e0b7` (the four `M6` rows re-recorded with harness and comparator `136eef1`, `run_evidence=executed`),
  `193590b` (two metrics and the `M7` amendment), `73ae279` and `1f04523` (their self-tests).
- Campaign evidence: `gate-a-review-evidence-20261008/` (`run1-preliminary` is preliminary and is not a result;
  `run2-consolidated` and `run3-subscriber-barrier` produced the report rows).
- The branch `judge/gate-a-comparator` (`107b999`) is an older sibling of A and B, not a descendant of either. It is
  published only to keep `50e60c1` reachable and is not part of this unit.

### 6.1 Amendment of the `M7` expectation, and what `I3.b` needs

`M7` was designed to break `G5.b`, `I3.a` and `I3.b`. Observed: `M7/S10` concurrent failed with `G5.b` only;
`M7/S10` controlled survived. The expectation is amended to `G5.b`, because of what the patch does, not because of
the count it would give:

- `M7` removes `suspendNonRecursive()` during the children's termination, so a queued message is dequeued and
  **received** while the stop is in progress. That is exactly `G5.b`.
- A received message has left the queue, so cleanup has nothing to dead-letter for it. The patch never drops a
  publication (`I3.a`, which is `M6`'s defect) and never publishes a message that was also received, or twice
  (`I3.b`). Expecting those from `M7` was a design error, not a judge gap.

Traceability: the previous expectation (`G5.b,I3.a,I3.b`) and this reason are in `AMENDMENTS` in `judge/mutate.py`,
which the checker cross-checks against the design and prints. The two report rows of `M7` written earlier keep
their original `expected_rules`; they are history and are not rewritten.

`I3.b` is **not** covered by association with `I3.a` or `M6`. `r_I3b` in `judge/comparator.py` fires only on a
`dead-letter` event, and then on either (a) the same message also `received` by the same actor, or (b) two
`dead-letter` events for it. A witness must therefore make a message take both paths (for example, process it
without removing it from the queue, so that cleanup dead-letters it too) or publish it twice. Both are new
mutations, not designed here, and the campaign is not extended in this delivery. `I3.b` stays pending until one
exists and a valid run catches it.

### 6.2 The evidence ref `judge/gate-a-comparator`

- **What it preserves:** commit `50e60c1` (`fix(judge): scenario deadline_ms matches what the runner waits`), with
  its eight ancestors not in `develop`. Two later commits sit on top of it (`bd7f730`, `107b999`), so the ref holds
  11 commits not in `develop` in total. It is an older sibling of units A and B, not a descendant of either.
- **Immutable anchor:** the tag `evidence/gate-a-comparator-107b999` points at `107b999` and is what keeps
  `50e60c1` reachable. The branch `judge/gate-a-comparator` is a convenience pointer and may move; the tag must
  never be moved, deleted or force-pushed, and neither must any ref be pruned while the report still cites `50e60c1`.
- **Relation to the report:** 23 rows of `judge/mutation-report.tsv` are re-evaluations of logs produced at harness
  commit `50e60c1`. The strict checker needs that commit to exist in a full clone to accept those rows; the tag is
  what keeps it reachable. CI is shallow, skips that probe and prints a note saying so.
- **What it is not:** it is historical evidence only. It is not integrated, not current pilot code, and its content
  is not what `develop` runs. Where the two differ, `develop` wins.
- **Review status:** the candidate `c2f1593..evidence/gate-a-comparator-107b999` (49 files, 2653 lines, high risk: an executable bit and shell
  in `judge/run-pekko.sh`, a process launcher in `judge/calibrate.py`) was **declined** for review by the owner on
  this candidate only. Declining is not approval: nothing on that ref is reviewed, and nothing authorizes merging it.
  Reviews stay enabled globally.
