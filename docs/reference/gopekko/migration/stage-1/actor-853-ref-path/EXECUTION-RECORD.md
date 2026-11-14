# EXECUTION RECORD: a Ref reports the ActorPath it was spawned at (#853, IDN-05)

Scope `docs/scopes/853-ref-reports-actor-path.md` (PR #1287, squash-merged into `develop` as `171477f65960441d825f41263c3dbcf1534e970e` after the Owner's integration authorization: section 11). G2 was granted by the Owner on head `82c97fc7690d85c1b9cb01c85f0372d57f663595`; head `360951389190f36ea85fcad25f84e0970e9b6586` was the one reviewed before the post-G2 amendment of section 8; the scope head now cited is `e6efd6f65848a433ac2b197e4d7439f309523a76`. Kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`, Pekko v1.7.1 `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`, base `develop` `db614b877a1c5ab7f00bf6364caa2c29ca1ac044` (unchanged at the last fetch). Four stacked draft PRs, all `Refs #853`, never `Closes`, in order **K #1297 (`0b0ff8c748e1825dfa6f1e52629e3f4892a00ced`) -> C #1298 (`6265496cb7ab523122dead83af684cc61758592b`) -> C2 #1300 (`31bd5cf8feff88426e1a559400a6b5c235ae12e6`) -> R #1299 (this commit; its head cannot be written in itself)**. **Gate evidence of section 4 was run on C `e20be2b` (code identical to the earlier `aff18e3`) and C2 `f34161e`; the later heads of K, C and C2 differ from those only in `RULEBOOK.md` wording (`git diff e20be2b <C> -- '*.go'` is empty), so the gates were not re-run on them and the Go evidence still applies to the same Go tree; any later change to Go files invalidates it. R adds only documentation. The Owner authorized the integration afterwards (section 11); #853 and #812 stay open.**

## 1. Authorization and conditions

Owner G2 (verbatim in the scope and in the K RUNBOOK): D-1 approved; D-6 Go-only evidence for P1-P4 plus the extra Classic scenario, stopping the affected part on a contradiction; D-7 #853 before #894; G-1 `PathOf(h Handle) (ActorPath, bool)`; G-2 a foreign-System handle reports its own path, with no claim that Address equality proves belonging to a System instance; G-3 guardians included, `Terminated.Who()` and the stale failed child deferred and documented as `(zero, false)`; G-4/G-5 kit and K/C/R split; G-6 macOS without network isolation and the pre-existing headers failure, conditioned on comparing with the current base and stopping on a new error. Three conditions were stop gates: (a) inventory of every publication and read route, (b) review of `idHandle` comparisons and keyed uses before editing it, (c) no zero path for a valid actor, proof of name invariance, explicit treatment of the defensive failure. At G2, no merge, issue closure, remote CI, `-race`, #894 or #1020 index was authorized (the integration was authorized later: section 11). The split changed no criterion or symbol.

## 2. Run

A Dev subagent (Claude Sonnet 5.5, `claude-sonnet-5-5`) ran K then C under the RULEBOOK and RUNBOOK; the writer of this record and two independent QA subagents are the same model. The kit's queue runner was not used: this is a kit-governed manual run, not an automated loop. Tokens and wall clock were not measured (`unknown`); the subagents known to this record at first writing were 4 (Dev, writer of R, QA of K, QA of C); QA runs launched later by the coordinator (section 7) and the Dev continuations are not counted, so the cost-log value 4 is a lower bound, not a measurement.

| SHA | PR | Subject |
|---|---|---|
| `8b745c9` | K | docs(actor): kit package for the #853 Ref path report (PR K) |
| `30e1d69` | K | docs(actor): bring the #853 kit package under the 400-line budget |
| `aff18e3` | C | feat(actor): a Ref reports the ActorPath it was spawned at (PR C) |
| `71686f7` | K | docs(actor): fix the #853 kit minors and record the post-G2 amendment |
| `e20be2b` | C | chore: merge docs/853-kit-package into feat/853-ref-path (normal merge) |
| `f34161e` | C2 | docs(actor): cite the Classic rows and document idHandle equality (PR C2) |
| `1b5bccd` | K | docs(actor): correct the #853 kit RULEBOOK wording (K1-K3) |
| `a6f5f8c` | C | chore: merge docs/853-kit-package into feat/853-ref-path (normal merge, docs only) |
| `350f985` | C2 | chore: merge feat/853-ref-path into docs/853-ref-path-citations (normal merge, docs only) |
| `0b0ff8c` | K | docs(actor): make the #853 kit comparison count reproducible |
| `6265496` | C | chore: merge docs/853-kit-package into feat/853-ref-path (normal merge, docs only) |
| `31bd5cf` | C2 | chore: merge feat/853-ref-path into docs/853-ref-path-citations (normal merge, docs only) |
| `e6efd6f` | scope | docs(scope): record the post-G2 Allowed-files amendment for #853 |

Outputs: K: `manifest.tsv`, `RULEBOOK.md`, `RUNBOOK.md`, `pekko-repro/{Probe.scala,RUN.md,observed.txt}`. C: `ref.go` (`PathOf`), `cell.go`, `spawn.go` (`childPath`), `hierarchy.go`, `stop.go` comment, `ref_path_internal_test.go`, `README.md`, and one two-line edit of `cell_release_internal_test.go` (section 8). R: this file and one cost-log row.

## 3. Stop conditions, extra Classic scenario, RED

- (a) none fired: `commitCell` holds the only `cell[T]` and `ref[T]` literals, the path is written there before `commit` publishes the cell, and `identity()` is its only copy. (b) none fired: no non-test code compares or keys a `Handle`; equality of an `idHandle` is unchanged for every handle that is compared or keyed. Residual edge, not approved away: a hand-built `idHandle` with a cell's `id`, `sys` and `done` and no path would compare unequal after the change and equal before; it exists only as a test row that is never compared (`registry_retention_internal_test.go:237`, and that row uses another System). (c) none fired: every accepted or generated name is a valid path element; `childPath` panics as an internal invariant before `reserve`, as `newGuardian` does, with no change to the documented error set. Analyses: K RULEBOOK sections 5-7; QA re-verified all three independently; the re-verification is in the QA relays of K and C (https://github.com/getsyntegrity/gopekko/pull/1297#issuecomment-6504406800, https://github.com/getsyntegrity/gopekko/pull/1298#issuecomment-6504407314) and the later relays on the scope (https://github.com/getsyntegrity/gopekko/pull/1287#issuecomment-6504862770), K (https://github.com/getsyntegrity/gopekko/pull/1297#issuecomment-6504863282), C (https://github.com/getsyntegrity/gopekko/pull/1298#issuecomment-6504877475), C2 (https://github.com/getsyntegrity/gopekko/pull/1300#issuecomment-6504878012), R (https://github.com/getsyntegrity/gopekko/pull/1299#issuecomment-6504863834) and the candidate (https://github.com/getsyntegrity/gopekko/pull/1299#issuecomment-6504886786).
- Extra Classic scenario (D-6): a real classic `ActorSystem` in a disposable copy, 38 rows in `pekko-repro/observed.txt`; no row contradicts the contract. QA re-ran it in a fresh offline copy: 38 rows, byte-identical. `upstream/pekko` untouched in both runs.
- RED (Dev, local, not stored in the repository): with a stub `PathOf` returning `(zero, false)` and a `childPath` returning the zero path, the N1-N4, P5-P8, P11, P12, equality and README tests failed; P1-P4 and P10 passed because they pin the negative answers.

## 4. Gates bound to C `e20be2b` and C2 `f34161e`

Environment: macOS arm64 (Darwin 25.3.0), Go 1.27.2, golangci-lint v2.14.0, one machine. Run by the Dev on both heads with identical results (exit codes below); the earlier run on `aff18e3` was also reproduced by QA. The coverage figure and the 31 diff lines are the same on C2: its changes are comments and docs.

| Gate | Exit | Result |
|---|---|---|
| `cd pekko-actor && go test -count=1 ./...` | 0 | ok |
| `gofmt -l`, `go vet ./...`, `go mod tidy -diff` | 0 | clean |
| `scripts/check_deps.sh`, `.github/scripts/go-sdk-update.sh --check` | 0 | ok |
| `golangci-lint run --config ../.golangci.yml ./...` | 0 | 0 issues |
| coverage recipe + `check_coverage.sh "$COV" 85` | 0 | pekko-actor 98.64% (1697 statements) |
| `diff-cover==11.0.0 ... --compare-branch=origin/develop --fail-under=85` | 0 | 100% (31 lines) |
| `make ci-fast`, `git diff --check` | 0 | ok |
| `.github/scripts/check-headers.sh` | **1** | 7 errors (`pekko-actor/actor/names.go` and six `pekko-persistence` snapshot files): output byte-identical to a detached worktree of `db614b87` on both heads, so pre-existing, no new error (G-6) |
| `unshare -rn ...` | 127 | `command not found` on macOS: unit tests ran **without network isolation** (accepted under G-6); the container variant was not run |

Not run: remote CI, `-race` (not authorized), `make parity-guard` (not applicable). Raw outputs are not stored in the repository. Differential (Spawn, SpawnAnonymous, restart, watch, handle equality) on a scratch copy of `db614b87` against C: 28 lines, identical (Dev); QA reported an identical result on its own public-API driver.

## 5. Pekko evidence and limits

Executed rows are Pekko-side only (K `pekko-repro` and PR #1292 for the earlier rows). Go was never run on a JVM. **No parity is claimed.** P1-P4 are Go-only evidence approved at G2; P6, P10-P12 are Go design.

## 6. Size (authored lines)

K 399 additions against `develop`. C 390 + 10 = 400 against K, exactly at the budget (merges excluded). C2 15 + 1 = 16 against C. R: this file plus one cost-log row, 94 authored lines against C2 (`git diff --numstat origin/docs/853-ref-path-citations HEAD`, merges excluded; recount in the PR body).

## 7. Independent QA, each bound to its SHA

Each verdict is a relay of an independent read-only QA subagent report, posted by the orchestrating Claude Code session through the Owner's account: **not a human review, not an Owner decision, not a merge approval**. A verdict covers only the SHA it names. AWM = APPROVE_WITH_MINORS.

| PR | Head | Verdict | Where |
|---|---|---|---|
| K #1297 | `30e1d699` | AWM | https://github.com/getsyntegrity/gopekko/pull/1297#issuecomment-6504406800 |
| C #1298 | `aff18e3` | AWM | https://github.com/getsyntegrity/gopekko/pull/1298#issuecomment-6504407314 |
| scope #1287 | `e6efd6f` | AWM | https://github.com/getsyntegrity/gopekko/pull/1287#issuecomment-6504862770 |
| K #1297 | `71686f7` | AWM | https://github.com/getsyntegrity/gopekko/pull/1297#issuecomment-6504863282 |
| C #1298 | `e20be2b` | APPROVE | https://github.com/getsyntegrity/gopekko/pull/1298#issuecomment-6504877475 |
| C2 #1300 | `f34161e` | AWM | https://github.com/getsyntegrity/gopekko/pull/1300#issuecomment-6504878012 |
| R #1299 | `8afaf42` | AWM | https://github.com/getsyntegrity/gopekko/pull/1299#issuecomment-6504863834 |
| K #1297 | `1b5bccd` | AWM | https://github.com/getsyntegrity/gopekko/pull/1297#issuecomment-6505077196 |
| C #1298 | `a6f5f8c` | AWM | https://github.com/getsyntegrity/gopekko/pull/1298#issuecomment-6505077737 |
| C2 #1300 | `350f985` | AWM | https://github.com/getsyntegrity/gopekko/pull/1300#issuecomment-6505078259 |
| R #1299 | `8e48166` | AWM | https://github.com/getsyntegrity/gopekko/pull/1299#issuecomment-6505078813 |
| candidate (local) | `c4bed692` | APPROVE, **superseded** | https://github.com/getsyntegrity/gopekko/pull/1299#issuecomment-6504886786 |
| candidate (local) | `8b88543` | APPROVE, **superseded** | within https://github.com/getsyntegrity/gopekko/pull/1299#issuecomment-6505078813 |
| candidate (local, current) | see the PR body of #1299 (a record cannot hold the head of a branch built after it) | **pending** | not claimed |
| K, C, C2, R | `0b0ff8c`, `6265496`, `31bd5cf`, this commit | **pending**: documentary-fix deltas not yet reviewed; the coordinator will add the verdicts | not claimed |

## 8. Post-G2 Owner decision and the former deviation

The edit of `pekko-actor/actor/cell_release_internal_test.go` (two lines: `path` in the keep set and the live-kept set of the cell field-classification guard) was made in C outside the Allowed files and reported as a deviation. **After G2, on 2026-10-29, the Owner decided** (verbatim): "Autorizo ampliar Allowed files para incluir las dos líneas de cell_release_internal_test.go, exclusivamente para reforzar el guard relacionado con esta unidad. Registrá la ampliación como decisión posterior a G2, sin afirmar autorización previa." This is **not** part of G2 and **not** an earlier authorization: the edit was made first and reported, and the Owner's decision followed. The scope records it at `e6efd6f65848a433ac2b197e4d7439f309523a76` (Allowed files restricted to those two lines; contract text unchanged).

## 9. QA minors, now addressed, and new documentation

- K minors fixed in `71686f7`, counts recomputed on `db614b87`: the hand-built test count is 49 raw pattern lines, of which 7 are `[]*cell[guardianMsg]{` slice literals of existing guardian cells and 1 is `[]waInvalidHandle{` matched by the substring `idHandle{`, so **41 constructions** in 10 files (QA's 43 is not reproduced); the handle `==`/`!=` inventory no longer claims completeness: it states its method (964 lines, 291 after the filter) and the limit of an operand-text filter, and found no comparison with two `idHandle` operands; `manifest.tsv` row 5 cites section 7.
- Production consumers of `idHandle` structural equality: none found by the stated grep (RULEBOOK section 6: 215 raw lines, 198 without comment-only lines); not a claim of universal equivalence. The residual edge (a hand-built `idHandle` with a cell's `id`, `sys`, `done` and no path flips from equal to unequal) stays disclosed in the RULEBOOK and in the `idHandle` doc comment.
- C2 (`f34161e`, 15 + 1 = 16 authored lines against C): test comments and the README cite rows of K `pekko-repro/observed.txt` (1-2, 4-12, 14-27, 28-37) that supplement, and replace none of, the assertions; `observed.txt` is unchanged. The citations are rows of Pekko output, not parity.
- K `1b5bccd` (documentary): K1 and K2 reworded; K3 completes the post-G2 quote in RULEBOOK section 3, byte-identical to the scope and RUNBOOK. K `0b0ff8c` makes the comparison count reproducible: the RULEBOOK states the exact two-line command (964 lines matched, 291 kept), re-run from a fresh detached worktree of `db614b87` with the lines extracted literally from the RULEBOOK; the numbers did not change (QA's own approximation left 242, which is not the stated filter). K is at 399 authored lines.
- Still open: C's minor "README and test comments cite no individual `observed.txt` rows" is addressed by C2, pending QA of C2. C is at 400 authored lines against K (390 + 10, merges excluded: `git diff --numstat origin/docs/853-kit-package HEAD`); the additions went to C2 instead of C, contract unchanged.

## 10. Open items and limits

- Open QA minors, not fixed: C2: the N1 and P7 citations overlap on rows 22-24, row 35 (guardian parent equals the root path) is cited for P8 without a Go assertion, and one README line is long; K and C sit at 399 and 400 authored lines with no margin. Confirmed by QA and recorded as inaccurate, not fixed: the message of scope commit `e6efd6f` says blank lines after six headings were removed; four were removed (after the title, "Owner decisions for #853", "Owner G2 approval and conditions" and "What it delivers").
- QA of the new K, C, C2 and R heads is pending. No human review is claimed; the QA texts are relays of subagent reports.
- Evidence comes from one macOS machine, without network isolation (`unshare` exit 127), `-race` or remote CI; a green repository-wide header gate is not claimed. No parity is claimed.
- This record changes no code, contract, issue or label. The integration of the PRs was authorized later (section 11); closing #853 and the state of #812 are Owner decisions that have not been made.

## 11. Integration (after the Owner's authorization)

Owner, 2026-10-29 (verbatim): "Autorizo integrar #853 en el orden propuesto". Method: a squash merge of each PR into `develop`, in the order scope, K, C, C2, R, each bound to its pushed head by `--match-head-commit`, with a normal merge of `develop` into each successor before its squash (no rebase, no force-push, no `--admin`). Squash commits on `develop`: scope #1287 `171477f65960441d825f41263c3dbcf1534e970e`; K #1297 `98a5489df2fe2e37413d1a73c33176ae34cb4947` (merged head `cbf637c26431c8390da3f1625cc07899b3addc2a` = QA-approved `0b0ff8c` plus develop; the delta is the scope file only); C #1298 `3afc1143f30e0031f9c6f499580b45b5a7a9c93f` (merged head `5fcdc72b38d4d156e936c72d01787452f813634d`; the Go diff against the QA-approved `6265496` is empty); C2 #1300 `7a5f4074080dcb6296800cc44d3e1b9d8db9ca57` (merged head `b1412b50a2c45f492715e4f48dab932a06c39894`); R #1299: its own squash commit cannot be written here.
- Conflict: merging `develop` into C2 gave add/add conflicts in `pekko-actor/README.md` and `pekko-actor/actor/ref_path_internal_test.go` after the squash of C. `develop`'s three files were byte-identical (md5) to the C head `6265496` that C2 already contained, so the branch's side was kept; the result equals `develop` plus C2's own 15+1 lines (diff against `develop` = those 3 files), the Go diff against the reviewed `31bd5cf` is empty, and `go test -count=1 ./...` in `pekko-actor` exits 0 on it. The merges of `develop` into K, C and R changed no Go file.
- QA covers the reviewed heads in section 7, not the merged heads above (they add only earlier squashes to the reviewed content). No human review is claimed. The remote checks `go` and `label` failed in 2 s on #1287; they are not required checks and no remote CI was requested. #853 and #812 stay open and #894 and the #1020 index are not started.
