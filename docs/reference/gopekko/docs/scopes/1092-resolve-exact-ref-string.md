# Scope: Resolve an exact local reference string (#1092, IDN-21)

Status: **Draft for gate G2; not approved; no implementation. #1092 stays blocked** (by #893 and #1323). Rewritten on 2026-11-04 against the ADR 0031 amendment of 2026-11-04 (PR #1324, merged into develop as `537002a`) and Apache Pekko v1.7.1 `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`. The earlier text (typed stale error, epoch/number token, foreign-epoch question, tokenless text as malformed) is replaced: the Owner's principle is that every functional decision is Pekko's and only Go form differs. This scope edits no issue and no label.
**Kit run required (ADR 0017/0024)**: this is an adaptation of `resolveActorRef(String)` (`A/ActorRefProvider.scala:622-657`, `A/dungeon/Children.scala:230-244`); the Dev runs the migration kit with its own manifest, RULEBOOK and RUNBOOK.

## Authority (Owner decision, points 3 and 4 of the amendment, quoted in #1324)

3. "String exacto: formato `path#uid` como en Pekko. uid entero distinto de cero. Sin uid, o con `#0`, significa cualquier encarnación. La forma de generar el uid es implementación Go, con dos condiciones: entero de 32 bits distinto de cero, y distinto para cada encarnación del mismo path."
4. "Resolución de un string obsoleto: como Pekko. Se obtiene una referencia que entrega a dead letters, no un error."
Points 5 and 6 also apply: no difference from Pekko is accepted; non-differential evidence is recorded under ADR 0033 D13 without declaring parity.

## What this delivers (when unblocked)

Resolving an exact string `path#uid` of this System returns a reference, never an error, for every outcome Pekko's `resolveActorRef` gives a reference for; the exception is the inputs on which Pekko throws `NumberFormatException` (rows 189, 211, 214, 215, 220, 221, listed below), whose Go form is D-3. Pekko v1.7.1 executed rows (`migration/stage-1/identity-oracle/pekko-repro/observed.txt`, PR #1292 revision 2 and PR #1326 `5499f74` revision 3; the probe line number is the row number):

| Input | Pekko outcome | Probe row | Scenario (`TestResolve_...`) |
|---|---|---|---|
| `path#uid` of the live incarnation | the live actor | **190** | `LiveUid` |
| `path` without uid | the live actor now at the path (any incarnation) | **187** | `NoUidIsAnyIncarnation` |
| `path#0` | the live actor now at the path (any incarnation) | **188** | `ZeroUidIsAnyIncarnation` |
| `path#oldUid`, the incarnation stopped and the name reused | `EmptyLocalActorRef`: terminated, equal to the old reference, old path, delivers to dead letters; **not** the new actor, **not** an error | **186**, with **185** (the uids differ) | `StaleUidDeliversToDeadLetters`, `StaleNeverReachesTheReusedName` |
| a path with no live actor (`/user/zzz`) | `EmptyLocalActorRef`, delivers to dead letters | **191** | `UnknownPathDeliversToDeadLetters` |
| another system's address, a remote address | `deadLetters` reference | **192, 193** | `ForeignAddressIsDeadLetters` |
| relative text `foo`, empty text | `deadLetters` reference | **194, 195** | `UnparsableIsDeadLetters` |
| `path#x` (uid not an integer) | **throws** `NumberFormatException` | **189** | `CorruptedUid`: Go form is an error (D-3) |

Revision 3 rows (Pekko v1.7.1 executed, rows 211-225). `ActorCell.splitNameAndUid` parses the uid with `Integer.valueOf` (`A/ActorCell.scala:400-404`), and `newUid` draws any non-zero `Int` (`:392-398`), which is why the uid is a signed 32-bit integer.

| Input | Pekko outcome | Probe row | Scenario (`TestResolve_...`) |
|---|---|---|---|
| the live uid | `RepointableActorRef` equal to the live actor. Observed live uids include negative ones: `-13786451` (row 190), `-1596376024` (the uid used in row 221), and positive `581136746` (rows 218, 219) | **218, 219** | `LiveUidMayBeNegative` |
| uid `0` is never a live uid | `0` is `undefinedUid` and means any incarnation (rows 187, 188); `newUid` retries on `0` (`A/ActorCell.scala:394-397`) | 187, 188 | `UidIsNeverZero` (Go-only generator check) |
| `path#` (empty uid) | **throws** `NumberFormatException: For input string: ""` | **211** | `EmptyUidIsAnError` (D-3) |
| `path#2147483648`, `path#4294967296` (above `Int` range) | **throws** `NumberFormatException` | **214, 215** | `UidOutOfInt32RangeIsAnError` (D-3) |
| `path#2147483647`, `path#-1`, `path#-5` (valid, not the live uid) | `EmptyLocalActorRef` that delivers to dead letters, with that uid in its path (`.../user/a#2147483647`, `#-1`, `#-5`) | **213, 216, 217** | `NonMatchingUidDeliversToDeadLetters` |
| `#` alone | `DeadLetterActorRef` (`pekko://sys3/deadLetters`) | **212** | `HashAloneIsDeadLetters` |
| uid on an intermediate element: `/p#<wrong>/c` and `/p#<live>/c` | **throws** `NumberFormatException` (the uid parsed is `12345/c` and `-1596376024/c`) | **220, 221** | `IntermediateUidIsAnError` (D-3) |
| uid on both: `/p#<live>/c#<childuid>` | `DeadLetterActorRef` | **222** | `UidOnBothElementsIsDeadLetters` |
| K3 stop window, stop told and not yet processed: resolve with the live uid, and without a uid | the live actor (`RepointableActorRef`, not terminated, equal to it) | **223, 224** | `StopRequestedNotProcessedResolvesLive` |
| K3, once the stop is processed: resolve with the old uid | `EmptyLocalActorRef`, terminated, equal to the old reference | **225** | `StopProcessedOldUidIsTerminated` |

"Delivers to dead letters" is asserted observably: `Tell` on the returned reference publishes a `DeadLetter` and reaches no actor. A reference that is the live actor compares equal to it by identity.

## Contract consumed (not owned)

| From | Needed | Why |
|---|---|---|
| **#1323** (encode and parse `path#uid`; owner of the parse if the Owner confirms O1 recommendation A) | the parse of the string into path and uid, with uid 0 or absent meaning any incarnation; the non-zero 32-bit uid that is different per incarnation of one path | this scope parses nothing itself under A |
| **#1322** (absolute path parser) | the path prefix, with Pekko's results | reached through #1323 |
| **#893** (exact lookup of the live actor by path; the selection front) | K1: the current live actor at a validated path of this System, without spawning; K2: it exposes the incarnation's uid so it can be compared (Pekko: `uid == undefinedUid || uid == crs.uid`, `Children.scala:240`); K3: the lookup follows Pekko's container state: a child told to stop whose `Terminated` is not yet processed is still in Pekko's container (`ChildrenContainer.scala:190`, `Children.scala:238-240`) and resolves as live; the stop window is observed in rows 223-225 (see the revision 3 table); K4: a read-only lookup (no spawn, no registry change, no dead letter); K5: a lookup racing a stop returns the live actor or the dead-letters outcome, never the reused name's actor for an old uid | R1 to R3 of the issue |

**No alternative resolver may be implemented to bypass #893.** #1092 must not read `System.byID`, walk `node.names` (`hierarchy.go:52-69`) or add a lookup of its own. If #893 returns the live actor without exposing its uid (K2), this story stops and reports.

## What this does not deliver

Encoding and parsing (#1323), absolute parsing (#1322), relative parsing (#1089), selection by path and wildcards (#893), Typed use (#957), remote and cluster. ActorSelection and Identify are included (ADR 0010 amendment, `42cf138`, PR #1325; #1022 closed as completed), so K1 to K5 are the lookup contract #893 provides.

## Evidence

Executed (Pekko v1.7.1): the rows above. Pekko side of each scenario is therefore a differential observation, not source reading. **Pending approval (ADR 0033 D13):** the Go-only part (the uid generation conditions, the reference type that delivers to dead letters, the Go form of the `NumberFormatException` rows, and K5 races), because the probe has no Go-side row. Go tests must be detecting tests (they fail if the implementation compares the path only). Source: `A/ActorRefProvider.scala:622-657` (`EmptyLocalActorRef` at `:655`), `A/dungeon/Children.scala:230-244`. Level: unit, public API of `pekko-actor/actor`, go-specs matchers, barriers not sleeps for the stop/reuse ordering.

## Allowed files (future, only after #893, #1323 and G2)

`docs/scopes/1092-resolve-exact-ref-string.md` (this file, PR #1289); `migration/stage-1/ref-resolve-1092/{manifest.tsv,RULEBOOK.md,RUNBOOK.md,EXECUTION-RECORD.md}` and one `migration/cost-log.tsv` row; `pekko-actor/actor/ref_resolve.go` and `ref_resolve_test.go` (new); `pekko-actor/README.md`. No change to `errors.go` is expected: no new error is a Pekko behavior. Every Go file ported carries the Apache-2.0 header and exactly one `// Ported from upstream/pekko/...` line. Nothing under `docs/adr/`, `scripts/`, `.github/`, `go.mod`, `go.sum`.

## Dependencies

| Dependency | State | Effect |
|---|---|---|
| ADR 0031 amendment (#1324) | merged, `537002a` | authority |
| #1323 | open, `status:blocked` | the string format, parse and uid; blocks this story |
| #893 | open, blocked | the live lookup (K1 to K5); blocks this story |
| #1022 | closed (`42cf138`, #1325) | ADR 0010 exclusion of selection resolved |
| #1322, #853, #852 | open | transitive |

## Go form (not behavior)

Name and receiver of the resolver, the type of the dead-letters reference, export (unexported by default until a consumer outside `pekko-actor/actor`, ADR 0011), and the test names are Go form. The three typed parse errors and the typed foreign/stale errors of the earlier scope are **removed**: Pekko returns references for those inputs.

## Decisions for G2 (Owner)

- **D-1.** The amendment (#1324) is on develop as `537002a` (done).
- **D-2.** O1 (carried to #1323): #1323 owns encode and parse, #1092 consumes it (recommended); or #1092 owns the parse.
- **D-3.** Go form of row 189 (Pekko throws `NumberFormatException` for a non-integer uid): an error return as the Go form of the exception, or a dead-letters reference. Recommendation: an error return. It covers every input on which Pekko throws `NumberFormatException`: rows 189, 211, 214, 215, 220 and 221.
- **D-4.** Approve K1 to K5 as the consumption contract, or require changes to #893's scope.
- **D-5.** Accept the Go-only evidence as "pending approval (D13)", or require more; the executed rows are Pekko-side only.
- **D-6.** Probe extension: done (rows 211-225, PR #1326, `5499f74`). It covered: `#` alone, a uid above 2^31, `path#-1`, a negative uid (the executed uids are negative signed Ints), a uid on an intermediate element (`/user/a#uid/b`; Pekko runs `getSingleChild` per element), and the stop window of K3.

## Verification and size

`AGENTS.md` "Go" commands when implementation is authorized (coverage and diff coverage >= 85%, golangci-lint gocyclo <= 10, `scripts/check_deps.sh`, `check-headers.sh`; no `-race` without explicit request). Forecast (estimate): about 150 authored lines, limit 400; if it projects over, stop and report. Risk: none for this PR (documentation only); for the code PRs, item 3 (new public resolver) and item 4 (lookup racing a stop, K5).
