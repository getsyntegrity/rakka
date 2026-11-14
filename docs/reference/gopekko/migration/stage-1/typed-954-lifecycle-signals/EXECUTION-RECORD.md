# EXECUTION RECORD: Typed signals infrastructure and PostStop delivery (#954, TYP-04, Slice A)

Scope: `docs/scopes/954-typed-lifecycle-signals.md`, PR #1234, approved by the Owner at gate G2 on 2026-10-28 (head `c5e7f19`, AO review approved at `52b6153` and `c5e7f19`). Kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`, Pekko v1.7.1 (`9a54d12`). Base: `develop` `3b47251`. Delivery: PR 1 #1235 (kit package), PR 2 #1236 (T2 to T4), PR 3 (T5, this record); stacked, draft, `Refs #954`.

## 1. What the kit run was

No kit loop ran: no kit agents and no campaign. The Dev (Claude Sonnet 5.5, one AO session) followed the RULEBOOK by hand and wrote code and tests from the Pekko sources listed in `manifest.tsv`, read, not run. No JVM ran. `manifest.tsv` keeps the kit runner contract (`source`, `target`, then the disposition columns).

## 2. Inputs and outputs

| Input | Output |
|---|---|
| `MessageAndSignals.scala`, `Behavior.scala:198`, `ActorAdapter.scala:309-331`, `BehaviorSpec.scala:210,214,297,301` | `pekko-actor-typed/typed/{signal,behavior,interpreter}.go` (PR 2) |
| Scenarios S1 to S10 and the D-7 edge | `signal_bhv_test.go` (S1, S2), `signal_runtime_test.go` (S3 to S10, edge) |
| The module README contract | `example_test.go` (`ExampleReceiveSignal`), `pekko-actor-typed/README.md` "Signals" (PR 3) |

## 3. Evidence level, honestly

Go unit evidence derived from Pekko specs. **Not V3**, no Typed parity claimed. S1 and S2 are read against `BehaviorSpec.scala`, not run. S3 to S10 are READ rows: no Pekko spec found; alternative evidence **pending Owner approval (ADR 0033 D13)**. Recorded gaps: a Stop that abandons a restart has **no scenario and no coverage is claimed** (D-4; the runtime's restart step cannot be held from outside `pekko-actor`); a `PostStop` handler error is a panic (`TODO(TYP-04-edge)`, D-7); the runtime's `poststop-panic` event is not observable from a test, so B17 "no panic" rests on "no handler call and `Done` closes". Temporary mechanism: the no-op `actor.PreRestarter` also skips the old instance's child unwatch/stop at a restart until #976, which must land before #1170.

## 4. Deviations and manual changes

| Item | What | Why |
|---|---|---|
| Test style | plain `testing`, not go-specs | go-specs is not a dependency of `pekko-actor-typed` and `go.mod`/`go.sum` are out of scope; matches the existing package tests |
| S6, S10 harness | local `restartHarness` with `Budget{MaxRetries: -1, Within: -1}` | the shared `harness` uses `Budget{}`, which denies every restart (found when S10 first failed); the scope named the shared harness |
| Seal of signals | one embedded unexported `sealed` type instead of five marker methods | the first PR 2 head `bf6daca` had 60% diff coverage (empty marker methods); fixed in `87d5edf`, 87% |
| Mutation check | with the no-op `PreRestart` renamed away, S6 and S10 fail; restored | proves they pin D-4 (a) |
| Gate environment | macOS arm64, go1.27.2; no `unshare` on this host | tests ran without the network-isolation wrapper; not equivalent to the `unshare -rn` variant, stated as such |

## 5. Budget and gates

`AGENTS.md` ("PR budget and commits"): at most 400 authored changed lines. PR 1: 64. PR 2: 393 added + 2 deleted = 395 (after `87d5edf`). PR 3 is stated in its body. Gates run once per final tree and posted as PR comments bound to the pushed SHA (command, environment, exit code, output). Remote CI could not run (GitHub Actions billing); no remote result is attributed. Models and tokens are not reported by the AO session, so the cost row says `unknown`.

## 6. QA verdicts

Pending: PR 1, PR 2 and PR 3 await `ao review trigger` and an independent QA verdict on each final head. These would be AO-internal verdicts, not GitHub approvals. No merge is authorized and #954 stays open (Slice B, B-2 and the M-A `ChildFailed` story are pending).

## 7. Integration and SHA correspondence

All four PRs were squash-merged into `develop`; the SHAs cited above are pre-squash heads of the stack as it moved, and are kept as the evidence trail. Gates in section 5 were run on `a0a0653` and re-checked on the rebased heads (`pekko-actor-typed` tests, `go vet`, golangci-lint); the code tree is identical across the rebases. Remote CI never ran (billing); nothing is attributed to it.

| PR | Pre-squash heads (in order) | Squash commit on `develop` |
|---|---|---|
| #1234 scope | `c5e7f19` (approved), `ae76544` (status only) | `85d2e76` |
| #1235 kit package | `807670f`, `9c37a04`, `cd94bba` | `8d350c5` |
| #1236 code and tests | `87d5edf`, `6e07145`, `0422231` | `ab9b44c` |
| #1237 example, README, record | `a0a0653`, `ba3426f`, `a5b919d` | `a03e38d` |

#954 stays **open**: Slice B (`Terminated`, death pact), B-2, the M-A `ChildFailed` story and the abandoned-restart evidence gap remain pending. Delivering Slice A closes none of them.
