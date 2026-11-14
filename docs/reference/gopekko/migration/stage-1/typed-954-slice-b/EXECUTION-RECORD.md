# EXECUTION RECORD: Typed `Terminated` delivery and death pact (#954, TYP-04, Slice B)

Scope: `docs/scopes/954-slice-b-terminated-death-pact.md` (PR #1249, branch `docs/954-slice-b-scope`), approved by the Owner at gate G2 on 2026-10-28 at `183171b9afdbf967bbbe73d79ecfa68a3b636a5f` (L3). Kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`, Pekko v1.7.1 (`9a54d1294c3ac7821bbed3c940769c3d5027d1ff`). Base: `develop` `fd5a04ecbe66c2e035175252282142d8e9311c71`. Delivery: six stacked draft PRs, `Refs #954`, none merged. The scope, the ADR amendment and the kit package (`RULEBOOK.md`, `RUNBOOK.md`, `manifest.tsv` in this directory) live on unmerged branches of the stack; they do not exist on this branch.

## 1. What the kit run was

No kit loop ran: no kit agents and no campaign. The TL and Dev agents (Claude Sonnet 5.5) followed the RULEBOOK by hand and wrote code and tests from the Pekko sources in `manifest.tsv`, read, not run. No JVM ran. The QA agent was independent of the TL and Dev and reviewed each PR head. Models and tokens are not reported by the session, so the cost row says `unknown` for wall clock and tokens.

**Cost row, `subagents` column.** The kit defines the column as the number of subagents a step ran (its example row has 35; `migration-kit/prompts/04-translation-kickoff.md`, line 62); earlier rows of this repository record 0 because no kit subagents ran (`migration/evidence/ana-003/kit-record.md`, "Subagents: 0 for the kit commands"). This unit's row records **7**: the number of subagents the orchestrating session started with the Agent tool for #954 Slice B, namely 2 read-only explorers, 3 Dev agents (B1; B2 and B3; B4) and 2 independent QA agents (one reviewed the scope; the other reviewed the kit, the ADR amendment, the code, the tests and this PR). Source: the 7 agent transcripts of that session (`.../tasks/<agent id>.output`, outside the repository, not versioned). The count excludes the TL/orchestrator itself, counts each agent once even when it was resumed several times, and is **not** a count of kit loop subagents (none ran). Wall clock and tokens remain `unknown`: the transcripts do not give a reliable per-unit figure.

## 2. Inputs and outputs

| Input | Output |
|---|---|
| `Behavior.scala:270-277`, `ActorAdapter.scala:163-180,220-227`, `PropsAdapter.scala:34`, `BehaviorSpec.scala:226-234,313-321`, `WatchSpec.scala:88,206`, `ActorContextSpec.scala:375,407` | `pekko-actor-typed/typed/{signal,interpreter}.go` (`Terminated`, `Who()`, `OnTerminated`, death pact) |
| Scenarios T1 to T16 | `signal_term_bhv_test.go` (T1 to T4), `signal_term_internal_test.go` (T5 to T11, T16, T6/T15 together), `signal_term_race_internal_test.go` (T12 to T14) |
| The module README contract | `pekko-actor-typed/README.md` "Terminated and the death pact" and limitations; this record; one `migration/cost-log.tsv` row |

## 3. PRs, heads and verdicts

Three different things are kept apart: the **reviewed head** (the SHA an independent QA verdict is bound to), a **later head** (a commit pushed after that review; it has its own QA verdict on its delta, and the earlier verdict is not rewritten), and the **integrated commit** (the squash on `develop`, recorded by a follow-up after the merges, as #1238 did for Slice A). Nothing is merged yet; every squash is pending.

| PR | Branch | Content | Reviewed head (QA FAVORABLE) | Later heads, each with its own delta QA FAVORABLE | Integrated commit |
|---|---|---|---|---|---|
| #1249 | `docs/954-slice-b-scope` | B0 scope | `81e5419214d04c7bf33478e3894b78de13da1371` (G2 approved at `183171b9afdbf967bbbe73d79ecfa68a3b636a5f`) | none | pending |
| #1251 | `docs/954-adr-0034-amendment` | B0b ADR 0034 amendment (P5 `Terminated` by ID, P6 wording); ACCEPTED by the Owner | `7732b8cb7c07ab33040d1256f407c4c2f76c25f6` | `96555f561fe11fcd08ed215746bb881277eb3f86` (one ADR 0035 cell), `2295528111232afb7c83c31cc905e1ad4377e28a` (one status sentence) | pending |
| #1252 | `docs/954-slice-b-kit` | B1 kit package | `5004690baec4dd84fc9789f1f3db185841847c50` | none | pending |
| #1253 | `feat/954-slice-b-terminated` | B2 code and first tests | `316a2c78db2780b7c92dc3a2343f40b99f940a13` | none | pending |
| #1254 | `test/954-slice-b-order-race` | B3 remaining tests | `bb70afef2e5f751e91d6183f104c4198d7f0e33c` | none | pending |
| #1255 (B4, this PR) | `docs/954-slice-b-record` | README, this record, cost-log | `1807490` and `8de6e3a` (QA FAVORABLE on each); a record cannot cite its own final head | in the PR itself | pending |

## 4. Owner decisions

| Decision | Content |
|---|---|
| L2 (2026-10-28) | D-B1 compared again; D-B2 `OnTerminated` without a public Typed watch, explicit that until #1171 it is no usable production capability; D-B3 internal tests in `package typed`, no new exports or seams; D-B4 `Same` is handled, explicit decline pending #1181 and not claimed; D-B5 generate the failure in interpretation and `StopSelf` without the Classic decider, no early #976; D-B6 the PR split, subject to each PR's real budget; creation of the M-A/`ChildFailed` story (#1250) |
| L3 (2026-10-28) | G2 approved at `183171b` with limits: D-B1 option A (`ID`); no re-slicing of #954/#1171; no full-parity claim; B1 and B0b authorized; Appendix A not accepted until reviewed; B2 and B3 only after the ADR amendment is accepted; one wording fix |
| L4 (2026-10-28) | T10 relabelled READ and a cosmetic fix at `:407` of the scope (a reduction of an evidence claim) |
| ADR acceptance | The Owner accepted the "Amendment (accepted)" of ADR 0034 (PR #1251): B-2 for `Terminated` only, by `ID`, and the wording of P6 |

## 5. Scenarios and evidence labels

Labels: SPEC = derived from a Pekko spec, read not run; READ = no spec found or the spec asserts less, alternative evidence **pending Owner approval (ADR 0033 D13)**; REGRESSION = pins current Go behavior, no oracle.

| Scenario | PR | Test | Label |
|---|---|---|---|
| T1 active handler gets `Terminated` with the watchee's ID | B2 #1253 | `TestTermBhv_T1_HandlerGetsTerminatedWithWatcheeID` | SPEC |
| T2 after a handled message | B2 | `TestTermBhv_T2_AfterMessage` | SPEC |
| T3 message after `Terminated` still handled | B2 | `TestTermBhv_T3_MessageAfterTerminated` | SPEC |
| T4 after a replacement the new handler gets it | B2 | `TestTermBhv_T4_AfterSwapNewHandlerGetsIt` | SPEC |
| T6 `Same` is handled (partial, D-B4) and T15 `PostStop` once | B2 | `TestTerminated_T6_T15_SameIsHandledAndPostStopOnce` | READ |
| T7a death pact stops the actor (default decider) | B2 | `TestTerminated_T7a_DeathPactStopsActor` | READ |
| T7b custom Restart decider is not consulted | B2 | `TestTerminated_T7b_DeathPactDoesNotConsultCustomDecider` | READ |
| T9 failing `Terminated` handler stops the actor | B2 | `TestTerminated_T9_FailingHandlerStopsActor` | READ |
| T5 replacement from `Terminated` is applied | B3 #1254 | `TestTerminated_T5_ReplacementBehaviorIsApplied` | READ |
| T8 death pact after `Setup` start and after a replacement | B3 | `TestTerminated_T8_DeathPactAfterSetupAndAfterSwap` | READ |
| T10 watching an already-dead actor delivers | B3 | `TestTerminated_T10_WatchingDeadActorDelivers` | READ (Go assertion; Pekko asserts less) |
| T11 watching twice delivers once | B3 | `TestTerminated_T11_WatchTwiceDeliversOnce` | READ (Go assertion; Pekko asserts less) |
| T12 no overlap with a message handler | B3 | `TestTerminated_T12_NoOverlapWithMessageHandler` | READ |
| T13 messages queued before are handled first | B3 | `TestTerminated_T13_MessagesQueuedBeforeAreHandledFirst` | REGRESSION, no oracle |
| T14 stopping watcher gets no `Terminated` after `PostStop` | B3 | `TestTerminated_T14_StoppingWatcherGetsNoTerminated` | READ |
| T15 Slice A, #847 and #956 tests still pass | existing tests | the module suite, plus T6/T15 above | REGRESSION |
| T16 public surface | B3 | `TestTerminated_T16_PublicSurface` | contract (API check) |

## 6. Evidence level and known gaps, honestly

Go unit evidence derived from Pekko specs. **Not V3**, no Typed parity claimed, no Typed oracle. Gaps found by the QA mutation checks and by the Dev:

- **T7a** cannot tell the default decider from an unconsulted one. **T7b carries the evidence**: the QA mutation that returns the error from `OnTerminated` makes T7b fail.
- **T14** does not force the queued-notification case and does not pin which runtime guard drops a late notification: removing both `stopRequested` guards left T12 to T14 green.
- **T12**: the overlap flag was not shown to fail on a genuine concurrent overlap; its QA mutation failed only through event order.
- **T13** is a regression of the Go runtime order (the notification is queued at the mailbox tail); no Pekko source asserts it.
- **T10, T11** are Go assertions; the Pekko specs assert less (T10's last assertion is the TestProbe's own death watch).
- The death pact error is **built and dropped on purpose** until failure logging exists (#1179); the cause is recorded nowhere.
- The module's tests use `t.Fatalf`, like its existing tests. The project convention (go-specs matchers, `mock.Controller`/`Spy`) is **not met**, and the module's `go.mod` has no go-specs. Flagged to the Owner.

## 7. Deviations, budget and gates

| Item | What | Why |
|---|---|---|
| PR split | B2 + B3 + B4 instead of the accepted B2 + B3 | scope §8 and D-B6: T5, T8, T10, T11, T16 moved from B2 to B3 for the 400-line budget; B3 was then split into B3 tests and B4 docs |
| Network isolation | the `unshare -rn` and container variants were **not run** | macOS host without `unshare`; tests ran without isolation, which is not equivalent |
| Remote CI | not run | GitHub Actions billing; nothing is attributed to it |
| `check-headers.sh` | 7 errors in other modules | they predate the stack |

Budget: `AGENTS.md` "PR budget and commits", at most 400 authored changed lines per PR; B4's count and its gates, bound to its head SHA, are in its PR body. The Go code of B4 is identical to `bb70afe`; golangci-lint at `bb70afe` reported 0 issues.

## 8. Not delivered

- Public Typed `Watch`/`WatchWith`/`Unwatch`: #1171. Until then no production caller can receive `Terminated`.
- `ChildFailed` and the cause half of B-2: #1250.
- Typed supervision capture of the death pact: #976.
- Explicit decline of `Terminated` (`Behaviors.unhandled`): #1181.
- No B3-level item is left undelivered. The abandoned-restart evidence gap of Slice A stays open.

#954 stays **open** and `status:blocked`. No parity claim. This PR changes no issue, label, ADR, scope, kit file, code or test.

## 9. Integration into `develop` (follow-up, added after the merges; the sections above are left as written)

Section 3 was written before the merges and says every squash is pending. The squashes exist now. The three notions stay apart: the **reviewed head** is the SHA a QA verdict is bound to; the **integrated commit** is the squash on `develop`; where a stacked branch had to be rebased onto `develop` to drop the commits of the PR merged before it, its **pushed head at merge** differs from the reviewed head and the content was compared against the reviewed head (a diff of 0 lines on the files of the PR).

| PR | Reviewed head (QA FAVORABLE) | Head at merge | Content vs reviewed head | Squash on `develop` |
|---|---|---|---|---|
| #1249 scope | `81e5419214d04c7bf33478e3894b78de13da1371` | same | identical (344 lines) | `48528350edec858b68b93417dbcda01ed669e57f` |
| #1251 ADR amendment | `7732b8cb7c07ab33040d1256f407c4c2f76c25f6`, later `96555f561fe11fcd08ed215746bb881277eb3f86` and `2295528111232afb7c83c31cc905e1ad4377e28a` (each with its own delta QA) | `2295528111232afb7c83c31cc905e1ad4377e28a` | identical (5 files) | `5cb51aa8722f0d25eb78564ec22fa69eb33e944c` |
| #1252 kit | `5004690baec4dd84fc9789f1f3db185841847c50` | `68458df2237e2998dde84abae4e51586887c077d` (rebased) | identical (0 diff lines; 3 files, 89 lines) | `30b868b6b4b6677f12b200c6b3057981dce6a052` |
| #1253 code | `316a2c78db2780b7c92dc3a2343f40b99f940a13` | same | identical in `pekko-actor-typed` | `08b0075852f4334678fff19cf0db00bde7ee9f97` |
| #1254 tests | `bb70afef2e5f751e91d6183f104c4198d7f0e33c` | `d6474f6ef8e200c572b788d470ac7272f8717b4b` (rebased) | identical in `pekko-actor-typed` (0 diff lines; 2 files, 205 lines) | `e80202d1edb538b7b11cbcede37462bb781e6660` |
| #1255 docs | `1807490` and `8de6e3a`, then `d8fb1d2d8749e44f7e5044f1c7834afb7c3f1646` (QA FAVORABLE on its delta) | `ec79f5c6bcd285b8e492255b16dce082270f3c72` (rebased) | identical (0 diff lines on its 3 files) | `8b4929e29f5bc090013b2791a604879b83ee7cba` |

Each merge used a squash with `--match-head-commit` on the head at merge, with no `--admin` and no protection bypassed. The rebases dropped only commits already integrated and were pushed with `--force-with-lease` on the PR's own branch.

**Local validation of the final `develop` `8b4929e29f5bc090013b2791a604879b83ee7cba`** (clean worktree; `pekko-actor-typed`): `go test -count=1 ./...` ok (47 tests); `go test -count=20 -run 'T12|T13|T14' ./typed` ok; `go vet ./...` clean; `gofmt -l .` empty; `go mod tidy -diff` clean; golangci-lint v2.14.0 with `../.golangci.yml`: 0 issues (gocyclo not raised, no `nolint`); coverage 100.00% against the gate of 85 (`scripts/check_coverage.sh`, profile in a temporary directory); `scripts/check_deps.sh` ok; `.github/scripts/go-sdk-update.sh --check` OK; `.github/scripts/check-headers.sh` reports 0 errors in `pekko-actor-typed` (the 7 errors in other modules predate the stack). Diff coverage: not applicable (this follow-up changes no Go file; the code PRs had 100% at their heads). Not run: network isolation (`unshare` is Linux-only; the container variant was not run), `-race`, remote CI. These are local results, not approved CI.

**State of the reconciled criteria of #954** (`docs/parity/typed-supervision-stories-reconciliation-2026-10-26.md`, items 1 to 3, with the updates of 2026-10-28). #954 stays open and `status:blocked`; this table does not close it.

| Reconciled criterion | Delivered by | Evidence label | Still pending |
|---|---|---|---|
| The current started behavior receives `PostStop` exactly once on stop | Slice A (S1, S2) | SPEC (S1, S2), READ (rest) | `StoppedWith` callback order (#1182) |
| A deferred behavior that never started receives no `PostStop` (B17) | Slice A (S5, S6) | READ | none |
| A watched actor's termination reaches the active signal handler | Slice B (T1 to T4, T10, T11) | SPEC derived (T1 to T4), READ (T10, T11) | no public caller before #1171; only the identity `Who()` is kept (no `path`, order, `toString`, send) |
| The `Signal` set is sealed (B16) | Slice A | READ | `ChildFailed` fields and `MessageAdaptionFailure` fields: #1250, #975 |
| `PreRestart` delivery | not delivered | none | #976 |
| `ChildFailed` | not delivered | none | #1250 |
| `MessageAdaptionFailure` | not delivered | none | #975 |
| An unhandled `Terminated` follows the death pact | Slice B (T7a, T7b, T8, T9) | READ, with T7b as the discriminating test | explicit decline (`Behaviors.unhandled`, the `Unhandled` result): #925, which keeps the unhandled marker (`Behaviors.scala:65`); #1181 only adds `empty` and `ignore` as constants over that marker and does not deliver the decline; capture by Typed supervision: #976 |
| Signal processing never overlaps message handling | Slice A (S8, S9), Slice B (T12) | READ | the overlap flag of T12 was not shown to fail on a genuine concurrent overlap |

READ rows are alternative evidence pending Owner approval (ADR 0033 D13). No Typed oracle exists (not V3). No parity claim.

**Attribution corrected by this follow-up.** The explicit decline of a signal belongs to #925 (the unhandled marker), not to #1181 (`empty`, `ignore`). Texts already on `develop` that still name #1181 for the decline are not edited here (outside this PR's diff and scope) and need a separate documented correction once the Owner decides it: the ADR 0034 "Amendment (accepted)", `docs/scopes/954-slice-b-terminated-death-pact.md` (D-B4 and its scenarios), `pekko-actor-typed/README.md`, sections 1 to 8 of this record, the kit `RULEBOOK.md` and `manifest.tsv`, the `TODO(TYP-04-edge)` comment in `interpreter.go`, the `ReceiveSignal` doc comment in `signal.go` and a comment in `signal_term_internal_test.go`. Some of those mentions of #1181 may be correct (for `empty` and `ignore`); each has to be read.

**Open decisions and pending items, not started.** Recording these squash commits; the `ChildFailed` destination in the reconciliation record (this PR adds a pointer to #1250); a clarification on #1113 (comment, no edit of its body); the tests use `t.Fatalf` as the module's existing tests do, and the project convention of go-specs matchers and `mock.Controller` is not met in `pekko-actor-typed` (its `go.mod` has no go-specs): whether and when to migrate them is an Owner decision; the worktrees created for this stack are not cleaned here.

## 10. Correction of attribution: explicit decline belongs to #925, not #1181 (added 2026-10-28; sections 1 to 9 are left as written)

**Fact.** The explicit decline of a signal by a handler (Pekko `Behaviors.unhandled`, `Behaviors.scala:65`) is the `Unhandled` result, which #925 (`TYP-03`) keeps ("SPLIT: keep (a) the unhandled marker"). #1181 (`TYP-47`) delivers `empty` and `ignore` over that marker; its R3 concerns the `empty` behavior failing on an unhandled `Terminated`. #1182 delivers `stopped`. Sections 1 to 8 of this record and the kit files below said "pending #1181" for the decline; the follow-up in section 9 already attributes it to #925 and lists these locations.

**What this correction does and does not touch.**

| Location | Old text | Treatment |
|---|---|---|
| This record, §4 row L2 and §8 "Not delivered" | "explicit decline pending #1181"; "Explicit decline of `Terminated` (`Behaviors.unhandled`): #1181." | Historical record, left as written; read #925 |
| `RULEBOOK.md` §2 (ActorContextSpec:483 PORT-GAP line) | "an explicit decline of `Terminated` waits for #1181" | Not edited: the rulebook is read-only after the kit package is delivered (kit rule 1; the package is the closed disposition record of PR #1252); this row is the linked correction. Read #925 |
| `manifest.tsv` row for `ActorContextSpec.scala:483` | "explicit decline (receiveSignal returns unhandled) pending #1181" | Not edited, same reason; the disposition `PORT-GAP` is unchanged. Read #925 |
| Scope `docs/scopes/954-slice-b-terminated-death-pact.md` | "pending #1181" in D-B4, scenarios T6, dependency row, decision table; Owner quotes L2, L3 | Not rewritten: dated correction note appended after the Log |
| `docs/adr/0034-typed-behavior-boundary.md` P6 and "Amendment (accepted)" | "an explicit decline ... stays pending in #1181" (three places) | Accepted text, **not edited**: a "Correction (proposed)" section with the exact old and proposed wording; acceptance pending the Owner |
| `pekko-actor-typed/README.md` (3 places), `interpreter.go` and `signal.go` comments | "#1181" for the decline | Corrected in place (living documentation; comments only, no behavior change) |
| `pekko-actor-typed/typed/signal_term_internal_test.go` comment (T6, line ~126) | "an explicit decline is pending #1181" | Not edited (the Owner asked not to modify tests); read #925 |

**Left unchanged on purpose because they are correct:** `docs/scopes/847-typed-behavior-install.md:30`, `docs/scopes/954-typed-lifecycle-signals.md:26` and `docs/scopes/956-typed-default-stop.md:41` ("Unhandled/Empty/Ignore (#925, #1181)" as joint references); `docs/parity/typed-supervision-stories-reconciliation-2026-10-26.md:237` (#1181 = empty/ignore); the audit documents under `docs/parity/actor-typed-design-audit-2026-10-25/` and `docs/parity/actor-typed-design-closure-2026-10-26/` (they use #1181 for empty/ignore); the quotation of #1181 R3 in the scope. No original SHA, verdict or result of this record is changed. #954 stays open and `status:blocked`. No parity claim; READ rows stay pending Owner approval (ADR 0033 D13).

