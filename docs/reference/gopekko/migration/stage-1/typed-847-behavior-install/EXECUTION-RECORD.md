# EXECUTION RECORD: install and replace a Typed receive Behavior (#847, TYP-02)

Scope: `docs/scopes/847-typed-behavior-install.md`, approved by the Owner at gate G2 on 2026-10-27 as commit `9310063`. Kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`, Pekko v1.7.1 (`9a54d12`). Base: `develop` `108880b`. Delivery: PR 1 #1227 (kit package), PR 2 #1228 (T2 and T3), PR 3 (T4 and T5, this record).

## 1. What the kit run was

No kit loop ran: no kit agents and no campaign. The one kit script used is `queue_runner.mjs`, only to validate `manifest.tsv`, which in the final PR 1 (`cfe9bef`) is a kit-conformant `source`/`target` manifest: `status` 4/14 done (the rows whose target is the RULEBOOK), `next --batch 20` lists the 10 Go targets still pending on the PR 1 branch (it has no Go files), `verify` OK, all exit 0. The Dev (Claude Sonnet 5.5, one AO session) followed the RULEBOOK by hand and wrote the code and tests directly from the Pekko specs read, not run, listed in `manifest.tsv`. Independent QA sessions (gopekko-18 for PR 1, gopekko-19 for PR 2) and AO's automatic reviewer read the result. Pekko was read, never run; no JVM ran.

## 2. Inputs and outputs

| Input | Output |
|---|---|
| Pekko `Behavior.scala`, `ActorAdapter.scala`; `DeferredSpec`, `BehaviorSpec`, `ActorContextSpec` rows of `manifest.tsv` | `pekko-actor-typed/typed/{behavior,context,interpreter}.go` (PR 2) |
| Scenarios S1 to S11 and the `Setup`-returning-`Same` edge | `typed_test.go` (S1 to S10 and the edge), `identity_test.go` (S11) |
| The module README contract | `example_test.go`, `pekko-actor-typed/README.md`, the root README state cell (PR 3) |

## 3. Evidence level, honestly

Go unit evidence derived from Pekko specs. **Not V3**: no Typed differential run exists. READ rows (S2, S4, S8, S9, S10 and the mailbox half of S11): pending Owner approval at G2 on 2026-10-27; **accepted by the Owner as alternative READ evidence on 2026-10-28** (ADR 0033 D13, see section 7). That is not differential Pekko parity, not V3 certification and not an executed Pekko scenario. Registered gap: a returned `Setup` whose factory returns `Same` fails the returning turn, Pekko the next one (`TODO(TYP-02-edge)`). Interim divergence: Restart, not Stop, until #956. No Typed parity is claimed.

## 4. Manual changes and review rounds

PR 1 (#1227) head moved: `abc24fa`, `42a4f7c`, `764b07f`, `f03aab8`, `802c694`, `cfe9bef`. PR 2 (#1228): `d6a294c`, `a909db1`, `26543c8`.

| Step | What changed | Why |
|---|---|---|
| PR 1 `abc24fa` to `42a4f7c` | QA REQUEST_CHANGES: StashSpec path lacked `scaladsl/`; S8 search counts wrong; PR body text | fixed in `42a4f7c`, approved |
| `764b07f` | `manifest.tsv` renamed `spec-rows.tsv` and a RULEBOOK sentence "not a kit queue" | AO's automatic reviewer: the file did not parse with `queue_runner.mjs:57`. **Superseded at `cfe9bef`**: `manifest.tsv` is restored as a real kit manifest and the RULEBOOK sentence states the runner contract |
| `f03aab8`, `802c694` | scope status line and T1 count edited, then reverted | the approved scope text is not ours to change without the Owner. **Final outcome at `cfe9bef`**: only the status line is updated to the approved text, by the Owner's authorization; the T1 row keeps "244 lines" |
| **Owner's corrective delta**, `cfe9bef` | `manifest.tsv` restored in the runner format (`source`, `target`, then `disposition`, `pekko_row`, `scenario`, `evidence`, `note`; 14 rows); RULEBOOK sentence replaced by the runner contract; scope **status line only** changed to "approved by the Owner at gate G2 on 2026-10-27" (T1 keeps "244 lines", the file has 248) | the Owner authorized a minimal delta after the two AO threads; the rename was superseded |
| Parser validation | from the repo root, `node migration-kit/scripts/queue_runner.mjs --manifest <manifest.tsv> status`, `next --batch 20`, `verify`: all exit 0; 4/14 done (rows whose target is the RULEBOOK), 10 Go targets pending on the PR 1 branch (it has no Go files), `verify` OK | no campaign was run |
| The two AO review threads | replied to with the commit SHAs and the parser result, and resolved | facts only |
| PR 2 `a909db1`, then `26543c8` | merge commits of the PR 1 heads `802c694` and then `cfe9bef`; the final PR 2 head is `26543c8`, and PR 2's own diff and `pekko-actor-typed` subtree (`2c72065`) are unchanged | restack without a force-push |
| `RULEBOOK.md` section 3, first bullet (a PR 1 file) | rewritten to record the Owner's READ decision of 2026-10-28 and the history; the earlier wording "The Owner accepted this on 2026-10-27." is replaced; `manifest.tsv` keeps its pre-decision label "READ, pending Owner approval", superseded by that bullet | done in PR 3 so that the heads of PR 1 (`cfe9bef`) and PR 2 (`26543c8`) did not move; a deviation from the RULEBOOK header ("does not change after PR 1") and from the PR 3 file list, authorized by the TL on the Owner's decision |
| `migration/cost-log.tsv` row placement | the #847 row sits before the `791-mbx-13-bounded-mailbox-evidence` row, not at the end | it merges cleanly with #1226's row appended on `develop` at `9a7076f` (two appends at the end conflict); the log has no ordering rule and its order is not chronological; resolved by the TL under the Owner's authorization of 2026-10-28; PR 3's diff and code are otherwise unchanged |
| `pekko-actor-typed/typed/identity_test.go`, the comment above `TestS11_RefAndMailboxIdentity` | the comment now states that the mailbox half is READ evidence accepted by the Owner as alternative evidence on 2026-10-28 (ADR 0033 D13), not parity, not V3, not an executed Pekko scenario; every changed line is a comment line | executable code and tests are identical (no Go token changed); the `typed/` tree hash moved from `7590bf5b99b5812dee054d875b154ce7fbbb37db` to `c7db46c3c8d381ab4b13ee8c34fe1b8ae4399424` only because of this comment, the module tree from `c62d997a00573e4c203838e5b41ee25bc3977760` to `b45d2b17915ca8b9dd20022f399ee112c23295c4` also because of the README line; `go.mod` and `go.sum` are identical; checked with `git diff`, `gofmt -l` and `go vet`; the Owner may ask to revert this comment edit |
| Gate setup | `golangci-lint` rebuilt in `bin/` with go1.27.2 (a first build with `GOTOOLCHAIN=local` used go1.27.1 and was discarded) | `.go-version` |

## 5. QA verdicts, bound to SHA and tree

| PR | Reviewer | Verdict | SHA | Tree |
|---|---|---|---|---|
| 1 (#1227) | gopekko-18 | APPROVE (also earlier at `42a4f7c` and `802c694`) | `cfe9bef` | `741b066` |
| 2 (#1228) | gopekko-19 | APPROVE (also earlier at `d6a294c`, tree `020dbfe`) | `26543c8` | `5d4f919` |
| 3 (#1229) | pending | | | |

These are AO-internal verdicts, not GitHub approvals. No merge is authorized and #847 stays open.

## 6. Budget rule and gates

`AGENTS.md` ("PR budget and commits"): at most 400 authored changed lines (additions plus deletions); lockfiles such as `go.sum` are excluded. PR 1 316; PR 2 419 added minus 42 `go.sum` is 377; PR 3 is stated in its body. Each PR carries its gate comments bound to its SHA (command, environment, exit code, full output). Remote CI could not run (GitHub Actions billing: jobs never start); nothing is attributed to it. Models and tokens are not reported by the AO sessions, so the cost row says `unknown`.

## 7. Owner decisions of 2026-10-28

Provenance: messages of the Owner relayed by the orchestrator (session gopekko-1) through the TL.

- **READ evidence.** "The Owner accepts Go tests as alternative READ evidence for S2, S4, S8, S9, S10 and the mailbox portion of S11." This is the acceptance of ADR 0033 D13. It is not differential Pekko parity, not V3 certification and not an executed Pekko scenario; the rows still block any V3 or parity claim.
- **Remote-CI billing exception.** "The Owner authorizes a TYP-02-specific remote-CI billing exception for PRs #1227, #1228 and #1229, conditional on valid local gates and independent QA on each final candidate. This does not waive missing mandatory local gates or authorize describing unexecuted checks as passing." Bounds: it waives no mandatory local gate; no unexecuted check is described as passing (remote CI did not run); the exceptions of #1226 and #1230 do not apply and this one does not extend to another story. State: PR 1 and PR 2 have QA approvals at their final heads `cfe9bef` and `26543c8`; PR 3's independent QA verdict is not recorded yet, so the condition for PR 3 is not met until it is.
