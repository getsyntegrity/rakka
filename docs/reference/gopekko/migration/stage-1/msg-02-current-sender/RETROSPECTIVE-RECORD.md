# Retrospective record: msg-02-current-sender (#814), PR 1 + PR 2a

Factual record written after the fact. It does not reconstruct any approval and does not alter any date. Real timestamps come from `git log --format='%H %aI %s'`.

## (a) Authorization that existed

- Owner G2 message of 2026-10-28 (recorded in `docs/scopes/814-msg-02-current-sender.md`, "Owner authorization"): approval only for the part covered by accepted ADR 0032 D1, D2, D3, D4, D6, D7; Q-D5 deferred; Q-INV decided; DeadLetter sender deferred to #896; no ADR PR.
- Scope commit `39ff8e027bcfd733f006f614618adf9bb926c9f1` (2026-10-28T22:22:15-03:00, `docs(scopes): record the Owner G2 authorization for #814 ...`), after the proposal commit `ee5778ddb72586effa950da9e39f5e254f28ac41` (2026-10-28T20:56:29-03:00).

## (b) Kit step that was missing

The kit requires the Owner to approve the RULEBOOK hash before any code (kit rule 3; RUNBOOK sign-off, the "T1-style" gate of earlier units). That approval did not happen before the code:

- `c6de3e34d28d23951cc086eca68e50c0786c5589` 2026-10-28T22:26:18-03:00 `docs(migration): kit package for #814 MSG-02 current sender (Refs #814)` (adds manifest, RULEBOOK, RUNBOOK).
- `5c945bdc53316a24bfa410446f75f842bc1b5fd7` 2026-10-28T22:26:18-03:00 `refactor(actor): carry the sender slot in an internal mailbox envelope (Refs #814)` (the Go change).
- Both commits carry the same timestamp, and the RULEBOOK was still `DRAFT, awaiting Owner approval of its sha256` when the code was committed. The temporal breach stays on record.

## (c) Exact content pending acceptance

- `migration/stage-1/msg-02-current-sender/RULEBOOK.md`, added in `c6de3e34d28d23951cc086eca68e50c0786c5589`, unedited since; SHA-256 `7b116f3522c310e33c11fe7328448e584cc404385f666530f3fd13ee5794bfd2`.
- `manifest.tsv`: status DRAFT. It lists the Pekko `sender()` rows as READ (read, not run); the rows for `sender_test.go` are PLANNED deliverables of PR 2b and no such file exists. It was corrected after `c6de3e3` (citations of `ActorRefSpec.scala:37-54` and `:83-113` are helper actors, not specs; the spec is `:390-413`); the corrected content is the one in the commit that adds this record.
- `RUNBOOK.md` is also unapproved and states that no cost figures were measured.

## (d) Correspondence

| RULEBOOK clause | Implementation | Executed evidence |
|---|---|---|
| 2a: internal `envelope[T]{msg, sender}` as the queue element | `pekko-actor/actor/mailbox.go:20-26` (type), `:38`, `:47`, `:55` (push, pop, pending); `cell.go:43`, `:165`, `:309`; `ref.go:66`; `stop.go` drain; `watch.go` runtime delivery | `go test -count=1 ./...` in `pekko-actor`: exit 0; `TestCellMailboxCounters` new subtest passes (RED before: compile error `undefined: envelope`) |
| Scenarios S1-S10 | not implemented (PR 2b) | pending |
| 2b: `TellFrom`, `Context.Sender`, `ErrOutsideTurn`, README | not implemented | pending |
| 3: execution record, cost-log row | not written | pending |
| Rule: nothing weakened, no sleeps | existing assertions only adapted to the envelope shape | pass/fail list of the actor suite equal to base except the new subtest (see PR #1268 comment) |

## Proposal

The Owner accepts this record retrospectively for this unit only, preserving the temporal breach on record, and then approves (or amends) the RULEBOOK hash above before PR 2b.

## Owner decision (2026-10-28, confirmed by the Owner in a real user message)

The Owner accepted this record retrospectively, for unit #1268 only, against RULEBOOK SHA-256 `7b116f3522c310e33c11fe7328448e584cc404385f666530f3fd13ee5794bfd2` (verified with `shasum -a 256` before writing this section). The temporal breach stays on record: the RULEBOOK was approved after the code was executed.

The acceptance does NOT accept differential evidence, parity, D5, or any ADR 0032 amendment. `RULEBOOK.md` is not edited.
