# EXECUTION RECORD: Ask late reply (#163)

Base: `develop` `7be6da7` (#162 merged). Package approved by the Owner at `4901e85` (RULEBOOK sha256 `9c446c56c1107d57863a346bb45c6bbff360fa41170b8f31881adbd7cbf0cb98`, RUNBOOK `78f374e699a47d68ae7489b8f71ecae952b81a503aef0b231b9cd9212560e2f0`). The RULEBOOK is unchanged and keeps its original header ("DRAFT, awaiting Owner approval"); **this section is the record of its approval**, as in #162. The launcher (`39611d65...`) and settings (`8ffce17a...`) of the #162 package were reused byte for byte; no permission was added.

## 1. Sources and scope

Pekko v1.7.1 `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`: `AskSupport.scala` 614-629 and, **adapted**, typed `AskSpec.scala` 132-182. Pekko's late replies follow a *timeout*; these tests exercise a late reply after a *normal return*. They do **not** prove timeout parity (`PORT-GAP(late-after-timeout)` in the test file; timeout is #164, cancellation and AK10 are #165, termination is #166). No Pekko oracle (sbt) was run: this PR does not claim Pekko parity.

## 2. What changed and who wrote it

| Artifact | Author |
|---|---|
| `pekko-actor/actor/ask_firstreply_internal_test.go` (+163/-1 against the base; the implementer commit is +153/-1 and the fix commit +17/-7): tests L2, L3+L4, L5, L6, a helper, L1 citations, `PORT-GAP` marker | **kit** (Sonnet implementer, then Sonnet fixer on confirmed findings) |
| `gofmt` indentation of two comments in the implementer's output | orchestrator (mechanical) |
| Runtime (`ask.go`, `ask_endpoint.go`, `errors.go`) | unchanged; no defect found |
| package documents, this record, cost entry | manual preparation |

L1 is satisfied by the existing #162 test (`StableResultAndExactDeadLetter`) plus citations, as the Opus design review advised; L4 is asserted inside the L3 test. Scenario mapping: L1 existing test; L2 `TestAskLateReply_Sequential`; L3 and L4 `TestAskLateReply_CrossAskAndRecipient` (adapted: the target captures B's endpoint and never replies, the test completes B); L5 `TestAskLateReply_RegistriesUnchanged`; L6 `TestAskLateReply_Concurrent`.

## 3. Roles, models and costs (self-reported by the CLI JSON, not independently verified)

Tokens are input + output + cache creation, the definition the Owner approved; cache reads are listed separately and not counted.

| Role | Model | Tokens | Cache reads | USD | Seconds | Turns |
|---|---|---|---|---|---|---|
| Architect/TL design review | claude-opus-5-5 | 44,479 | 79,930 | 0.4576 | 78 | 8 |
| Implementer | claude-sonnet-5-5 | 51,208 | 333,133 | 0.3074 | 81 | 15 |
| Architect/TL final review, principal reviewer | claude-opus-5-5 | 46,508 | 205,805 | 0.5157 | 95 | 11 |
| Second reviewer | claude-sonnet-5-5 | 42,186 | 80,873 | 0.2173 | 71 | 8 |
| Fixer | claude-sonnet-5-5 | 24,598 | 201,186 | 0.1458 | 37 | 17 |
| Architect/TL review of the fix delta | claude-opus-5-5 | 17,114 | 79,240 | 0.1852 | 31 | 6 |
| Total | | 226,093 | 980,167 | 1.8290 | 393 | 65 |

Timestamps and minutes in `entries/cost-log.entry.tsv` are approximate (rounded from the session clock and the CLI durations). Agents: 6 of 6. Tokens: 226,093 of 600,000. Wall clock of the agents: about 6.5 minutes of 60. Subscription only; no extra charge. Reviewer read-only access was controlled by the prompt and by `shasum` of the four Go files before and after every agent; all comparisons matched except the expected change of the test file by the implementer and the fixer.

## 4. Reviews

| Review | Verdict | Outcome |
|---|---|---|
| Opus design review | AMEND, no blocker | Guidance for the implementer: L1 already covered, merge L4 into L3, a deterministic recipe for L3, barrier ordering, no atomicity claim for L6. Applied |
| Opus final review | FINDINGS, no blocker | PORT-GAP marker missing (confirmed, fixed); L3 comment overclaims (confirmed, fixed); L5 measured once (confirmed, fixed); citations beyond the case table (rejected: narrative) |
| Sonnet second review | FINDINGS, no blocker | L5 measured once (confirmed), L3 not marked as adapted (confirmed), payloads never equal or repeated (confirmed as hardening: the "equal to the result" mutant was already caught by L6, so only a repeated payload was added to L2); no dead-letter collector in L5 (rejected: not required by RULEBOOK L5, and Opus verified the ordering); loop variable capture (rejected: `go 1.27.0`) |
| Opus delta review (17 insertions, 7 deletions) | PASS | one informational style note (a 170-character comment) |

No third reviewer was needed. The two reviewers did not disagree on any blocker.

## 5. Mutations (isolated copies of `pekko-actor`, `GOWORK=off`; the delivery tree was only read)

Control (unmutated copy): no failing test. Each mutant of `askEndpoint.Tell` made tests fail: M1 a late reply overwrites the stored reply, M2 a late reply is not published, M3 it is published twice, M4 it is addressed to a fresh id (each fails six tests, including `TestAskLateReply_Sequential`, `_CrossAskAndRecipient` and `_Concurrent`); M5 a late reply registers the endpoint in `byID` (fails only `TestAskLateReply_RegistriesUnchanged`); M6 a late reply equal to the stored result is dropped (fails `TestAskFirstReply_ConcurrentReplies`, `TestAskLateReply_Concurrent` and `TestAskLateReply_Sequential`). A first version of M5 did not compile (a literal of an interface type); it was invalid, not a detection, and was fixed and rerun. **No mutation revealed a runtime defect**: they check that the tests detect wrong runtimes, and the real runtime passes.

## 6. Gates (darwin/arm64, this worktree, head `6f8b2a3`)

`gofmt` clean; `go vet` and `go mod tidy -diff` ok; `golangci-lint` v2.14.0 (the pinned version, installed outside the repository): 0 issues; `go test -race -count=1` in the six workspace modules, with the coverage gate over the six profiles (exit 0: factory 99.54%, pekko-actor 98.60%, tools 100%, three modules without executable code skipped) and its selftest (exit 0); 500 `-race` runs of the Ask tests; `check_deps.sh`, `check-headers.sh`, `check-adr-signatures.sh`, `check-judge-matrix.py`, `modules_selftest.sh`, `go-sdk-update.sh --check`: exit 0. `go test` never ran inside the loop (denied); the orchestrator ran it. The judge campaign was not renewed (no judge input changed; the checker passed). Not run: Linux locally and the Pekko oracle.

## 7. Limits and follow-ups

- Native review: the package commit `4901e85` was closed as low risk and acknowledged. The code candidate (medium risk) is **pending the Owner's answer** to the consent envelope; if it is declined or never answered, this PR states that the test commits have no native review of their own. The review inside the kit (Opus and Sonnet) is a separate, recorded review.
- `ask` still blocks without a deadline and ignores its context while waiting (#164, #165); no goroutine or timer leak measurement exists (AK10, #165/#166).
- Carried over from #162, not done here: `afrBuildPanics` calls `ask` without an upper bound (review R3-001) and `run-agent.sh` does not escape the root when rendering the settings (R3-003).
- The checked-in copies of these two follow-ups remain open work; they are not part of this slice.
