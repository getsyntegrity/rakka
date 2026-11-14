# RULEBOOK: ActorPath navigation (unit actor-852-path-navigation, #852 IDN-04)

Status: **approved at gate G2 by the Owner on 2026-10-29** for `docs/scopes/852-actor-path-navigation.md` at head `76810743150fea3a18be5e72bc39f2a5e627b5f6` (PR #1284; latest scope head `fe93c8ca41d80d2ade9cb000e0a1bcbb6175072b`, whose later commits are documentation delta only). Read-only inside any loop (kit `CLAUDE.md` rule 1); it does not change after PR K.

## 1. Unit

- Capability: two pure navigation members on the existing `ActorPath` value (`pekko-actor/actor/actor_path.go`): `Parent() ActorPath` and `Descendant(names ...string) (ActorPath, error)`. Neither touches a `System`, the registry or an actor. Source: Pekko `ActorPath.scala:188` (`parent`), `:286` (`RootActorPath.parent = this`), `:203-204` (`/(Iterable)`), `:209` (`descendant`) at v1.7.1 (`9a54d1294c3ac7821bbed3c940769c3d5027d1ff`).
- Governing decisions: ADR 0031 P-4, P-5, P-6, P-7; ADR 0011 (individual export justification, Owner decision 1); ADR 0017 "Delivery units"; ADR 0033 D3, D13; the six Owner decisions of 2026-10-29 recorded in the scope.
- Manifest: `manifest.tsv` follows the kit runner contract (`migration-kit/scripts/queue_runner.mjs`): `source` and `target` are the first two columns and the only ones it reads; the other columns are the disposition record. Moved rows point at this RULEBOOK, whose recorded disposition is their output.
- Writable: exactly the "Allowed files" of the scope for the PR in hand. Nothing under `docs/adr/`, `scripts/`, `.github/`, `CLAUDE.md`, `AGENTS.md`, `names.go`, `address.go`, `go.mod`, `go.sum`, the root `README.md`, nor the scope file, nor the #812 scope.

## 2. Evidence classes

- **Executed**: rows produced by running Pekko v1.7.1 in a disposable `git archive` copy, never in `upstream/pekko` (PR #1292, `migration/stage-1/identity-oracle/pekko-repro/observed.txt`, probe revision 2, commit `d4ac1c479f478254bddd1f8dc44ea5d35f35eea1`). Rows used, by `observed.txt` line: 1 `root.parent==root` true; 2 `root.parent` `pekko://sys/`; 3 `(root/user/a).parent` `pekko://sys/user`; 4 its parent `pekko://sys/`; 5 `root.descendant(user,'',a)` `pekko://sys/user/a` elements n=2 ["user","a"]; 6 `root./(Seq(user,'',a))` same; 7 `root.descendant(Seq(''))` `pekko://sys/` elements n=1 [""]; 8 `descendant` of an empty list `pekko://sys/`, same=true; 9 `root./("")` `pekko://sys/`, n=1 [""]; 10 `root.elements` n=1 [""]; 69 `fromString('pekko://sys/a#5')` uid=5, elements n=1 ["a"] (accepted difference: Go rejects `#`).
- **SOURCE**: read, not run.
- This is **not a JVM run of Go**. Go was never run against Pekko. The executed rows are Pekko behavior only; no parity is claimed.
- Not executed: the zero-value rows (S3, S7, zero cases of S5/S6), `Parent` of the zero path (Pekko has no zero path), the Go-only error rows (S8, S9, S11) and S10.

## 3. Upstream search

`ActorPathSpec.scala` contains no test that mentions `parent` or `descendant` (grep of the file). `ActorSelectionSpec.scala:271` calls `descendant(Seq("a","b").asJava)` with no empty name. `SupervisorMiscSpec.scala:178` and `pattern/AskSpec.scala:178`, `:198` use `sender().path.parent` on child paths only. This is "no spec found by these searches", not a proof of absence; the root rule and the skip rule rest on the executed rows.

## 4. Decisions (Owner, 2026-10-29)

1. Deliver `Parent` and `Descendant`, each with its own ADR 0011 justification.
2. `Root` is out of this unit; it stays in the general scope of ADR 0031 P-4.
3. The kit is required (`AGENTS.md`).
4. `Parent(zero)` is the zero path.
5. `Descendant` on the zero path returns the zero value and `ErrInvalidPath`, even with no names or only empty names. On a valid path, no names or only empty names return the receiver without error.
6. The comparison "appending segments equals parsing the complete path" (criterion 2b) moves to #812 over the shared domain of the scope ("Hand-off to #812"). It is **blocked by #812 / #1187** and is not asserted, deduced or approximated here.

## 5. Conventions

- Header: Apache-2.0 plus exactly one `// Ported from upstream/pekko/...ActorPath.scala` line in `actor_path.go` (unchanged); checked by `.github/scripts/check-headers.sh`.
- Scenarios S1 to S11 of the scope go in `pekko-actor/actor/actor_path_test.go` with go-specs matchers; no `time.Sleep`, no goroutine, no network, no `-race`.
- `Descendant` reuses `checkPathElement`; the validator is not copied. Changing `Child` is a stop.
- No new exported symbol beyond `Parent` and `Descendant`; no `//nolint`; cyclomatic complexity at most 10.
- Invariants of `Descendant`: skip only the empty string; the zero-receiver check precedes the skipping; a bad name fails the whole call and returns the zero value with `ErrInvalidPath`, the receiver unchanged (scope S8/S9).
- Unknown behavior: a greppable `// TODO(IDN-04-edge): <reason>` and keep going (kit rule 8).
- Comments and names claim no more than is shown: no Pekko parity.
