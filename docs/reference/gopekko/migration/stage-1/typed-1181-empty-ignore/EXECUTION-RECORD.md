# EXECUTION RECORD: Empty and Ignore (#1181, TYP-47)

Kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`, Pekko v1.7.1 (`9a54d12`), base `develop` `bc727c7`. PR #1264 (`Refs #1181`) is **integrated** into `develop` as squash `595595cfa87ed6d9ffcd143043707f54f6437763`; its final candidate was `32e7870`.

**Current state (read this first).** The kit package was accepted for this unit as an exception to scope/G2, and diff coverage 83% was accepted exceptionally, both by Owner decision on `043ffaa` (section 6). Still pending under ADR 0033 D13: the READ rows and the alternative evidence for AC-R4-1. #1181 stays open. Sections 1 to 6 are the **history up to the candidate that was integrated**; the words "proposed", "no merge" and the SHAs in them describe the PR before the Owner decisions and before the squash, and are kept as written. Section 7 records what changed after the integration.

## 1. Run

1. Model: Claude Sonnet 5.5 (`claude-sonnet-5-5`), one Claude Code session, no subagents. Tokens and wall clock were not measured.
2. Inputs: `Behavior.scala:248-249,270-275,297-306`, `BehaviorImpl.scala:67-69,78-84`, read, not run. Dispositions in `manifest.tsv` (5 rows).
3. Outputs: `behavior.go` (`Empty`, `Ignore`, `emptyB`, `ignoreB`), `interpreter.go` (resolves both), `empty_ignore_internal_test.go`, `unhandled_internal_test.go` (U9 pin), README ("Empty and Ignore"), `go.mod`/`go.sum` (go-specs v0.4.0).
4. Manual changes and deviations: (a) first version built both as receive behaviors; review found that `ReceiveSignal` then accepted them as a base (ADR 0034 P2), so they became zero-size types; (b) the first tests passed under the `Ignore` mutation (no synchronization); they were rewritten with an observable barrier; (c) tests use go-specs after the Owner's remark.

## 2. RED evidence (observed locally)

Before the implementation the test file failed to compile: `undefined: Empty`, `undefined: Ignore`. After the rewrite of section 1.4 the mutations below were applied with `go test -overlay` (no file changed):

| Mutation | Result |
|---|---|
| Ignore answers a message Unhandled | FAIL (record appears) |
| Ignore answers Terminated Unhandled | FAIL (death pact) |
| Empty answers Terminated Same | FAIL (no death pact) |
| Interpreter keeps the old handler on Empty/Ignore | FAIL (second message reaches it) |
| Ignore (or Empty) answers Terminated right but other signals the other way | FAIL once the ordinary-signal specs were added; it SURVIVED before (found by the independent read-only QA subagent) |

Before the rewrite, the first and second mutations passed or failed intermittently.

## 3. Gates (local; remote CI attributes nothing)

Commands are those of `AGENTS.md`, "Go". Two environments: **host** = macOS (Darwin 25.3.0) arm64, go1.27.2 (matches `.go-version`), golangci-lint v2.14.0; **container** = `golang:1.27-alpine`, linux/arm64, go1.27.1 (see impediment 1).

| Gate | Tree | Command | Exit | Result |
|---|---|---|---|---|
| Unit tests, no network | `d08b600` | `docker run --rm --network none -v <worktree>:/src -v <modcache>:/go/pkg/mod:ro -e GOPROXY=off -e GOTOOLCHAIN=local -w /src golang:1.27-alpine`, running `go test -count=1 ./...` in each module read from `go.work` | 0 | 11 of 11 modules ok (container) |
| Per-module coverage | `d08b600` | `scripts/each_module.sh bash -c '... go test -count=1 -covermode=atomic -coverprofile=... ./...'`, then `scripts/check_coverage.sh "$COV" 85` | 0 / 0 | `pekko-actor-typed` 100.00% (78 statements); every module at or above 85% (host) |
| Diff coverage | `d08b600` | `pipx run --spec diff-cover==11.0.0 diff-cover "$COV"/*.xml --compare-branch=origin/develop --fail-under=85` (`origin/develop` = merge base = `bc727c7`) | **1** | **FAILED: 83% (10 of 12 lines).** `interpreter.go` 100%; `behavior.go` 80%, missing lines 57-58. See impediment 2 |
| Headers, global | `d08b600` | `.github/scripts/check-headers.sh` | **1** | 7 findings, all outside this diff: `pekko-actor/actor/names.go` and six `pekko-persistence` files (`inmem/snapshot.go`, `inmem/snapshot_conformance_test.go`, `persistent/snapshot_ops.go`, `persistent/snapshot_recovery_test.go`, `snapshot/snapshot.go`, `spi/snapshot.go`). None under `pekko-actor-typed`, none in the PR diff. Same count as the #956 record (section 3); not caused by this PR and not fixed here |
| Go SDK drift | `d08b600` | `.github/scripts/go-sdk-update.sh --check` | 0 | `OK: 11 go.work on go1.27.2` |
| vet, gofmt, tidy, tests | `3757230` | `go vet ./...`, `gofmt -l .`, `go mod tidy -diff`, `go test -count=1 -cover ./...` in `pekko-actor-typed` (host) | 0 | clean; coverage 100.0% |
| golangci-lint | `3757230` | `golangci-lint run --config ../.golangci.yml ./...` in `pekko-actor-typed` (host) | 0 | 0 issues (gocyclo <= 10) |
| Architecture, parity | `3757230` | `scripts/check_deps.sh`; `make parity-guard` | 0 / 0 | ok; structure only |

**Equivalence of inherited rows.** The rows at `3757230` were not repeated because the code did not change afterwards: `3757230..d08b600` touches only `migration/` (checked with a name-only diff between the two commits), and the commit that holds this record touches only `migration/` as well (the same check against the final head is stated in the PR comment). Go sources, `go.mod` and `go.sum` are byte-identical across those commits.

**Impediments and exceptions that need the Owner (nothing here is marked green):**

1. **Network isolation variant.** `unshare` does not exist on macOS, so the container variant was used. The alpine image carries go1.27.1, not 1.27.2, and has no bash, so `scripts/each_module.sh` could not run inside it; the loop reads the same module list from `go.work`. The tests passed with `GOTOOLCHAIN=local`. **Superseded:** the same run on `golang:1.27.2-alpine` (go1.27.2, `.go-version`), same command and tree content (`d2f601c`, docs-only over `d08b600`), exit 0, 11 of 11 modules ok. The go1.27.1 run is kept above as the first attempt. Only the absence of `each_module.sh` inside the image (no bash) remains as a documented variant.
2. **Diff coverage 83%, below 85%.** The two missing lines are the sealed-interface marker methods `func (emptyB[T]) behavior(T) {}` and `func (ignoreB[T]) behavior(T) {}` (`behavior.go:57-58`). They have zero statements (profile blocks `57.35,57.35 0 0`), so the module gate is unaffected (100%); `diff-cover` counts the lines because the Cobertura converter lists them. The existing marker methods at `behavior.go:50-56` have the same shape and the same 0 hits, but are outside the diff. No test was added only to call them: that would be coverage padding. **Owner decision (PR #1264 at `043ffaa`): accepted exceptionally; no padding test.** Bound to that SHA.
3. **Global headers check fails on 7 files that predate this PR.** No decision is needed for this PR unless the Owner wants them fixed first.
4. **Remote CI** was not examined (the PR lists 2 status checks whose content was not read), so no remote result is attributed.

**Independent QA (read-only subagent, head `6ffac5b`):** verdict ESCALATE; no defect in the implementation; `go test -count=20 ./typed/` ok; 9 mutations, 1 survived (ordinary signals), fixed afterwards by the "ordinary signals" specs. Still open: AC-R4-1, indirect evidence only (section 4). The subagent's report is data, not an approval.

**Re-run after that fix (host macOS, go1.27.2):** `go vet`, `gofmt -l`, `go test -count=20 ./typed/`, golangci-lint v2.14.0 (0 issues): exit 0; both M10 variants fail the new specs. Not repeated: tests with no network, diff coverage, check-headers. Production files are unchanged since `3757230`, so those results stand for them; the new lines are test code.

## 4. Evidence class and pending approval

Go unit evidence from a READ of Pekko sources, **not V3**, no Pekko run. **All rows are READ, pending Owner approval (ADR 0033 D13).** AC-R3-1 differential scenario is a PORT-GAP (#1046). AC-R4-1 stays in #1181 and has only alternative (behavioral) evidence because no public aliveness accessor exists (RULEBOOK 3, manifest row 4). No parity is claimed.

## 5. Not delivered

Publication of `UnhandledMessage` (#925 Slice B), `Stopped` (#1182), Typed watch (#1171), an aliveness accessor, the Typed-harness differential scenario.

## 6. QA and integration

Review of PR #1264 at `e424d29` requested changes (three P2). Item 1 (`ReceiveSignal` accepted both as a base) and item 2 (tests that could pass without processing the stimulus) are fixed in `3757230` and were not reopened. Item 3 asked for the kit evidence; this package is that evidence, written after the implementation and **proposed, not approved**. A later review asked for the remaining gates (section 3) and for AC-R4-1 and the authorization to be reconciled (RULEBOOK 3 and Status).

**Owner decisions (PR #1264, head `043ffaa`, 2026-10-28; message partly truncated, legible parts only):** (a) the kit package is accepted for this unit as an exception to scope/G2; it does not make the retrospective steps earlier ones; (b) diff coverage 83% accepted exceptionally; (c) READ rows and AC-R4-1 stay **pending** under ADR 0033 D13. The network-isolation image was resolved by the go1.27.2 run.

**Still open:** approval of the READ rows and of the alternative evidence for AC-R4-1 (ADR 0033 D13); no Pekko parity or V3 is claimed. Independent read-only QA on `6ffac5b` found no implementation defect and one surviving mutation, fixed in `51e874d`; the reviewer of `043ffaa` reported no new defects in the delta. *(History, written at `043ffaa`: "No merge".)* #1264 was later integrated (section 7). #1181 stays open and keeps its labels.

## 7. After the integration

1. **Integration.** #1264 was squash-merged into `develop` as `595595cfa87ed6d9ffcd143043707f54f6437763`; the final candidate of the PR was `32e7870`. The accepted exceptions (kit package; diff coverage 83% on the two zero-statement marker methods) keep the scope of the Owner decision of section 6 and are not extended to later changes.
2. **Audit finding on the tests.** `eiRunning` concluded that the actor was alive when `Done` did not close within 300 ms. That is a time window used as proof of absence. In the correction (a separate draft PR, `Refs #1181`, `Refs #956`) the helper is replaced by two observable ones: `eiAlive` sends a later message and waits for the events it must produce including its barrier `done:<later>`; `eiDied` waits for `Done`. The 30 s timeout of `dsRecv` remains only as a limit against a hang. No production code, export or dependency changed.
3. **Meaning of the new interaction.** A later message that is really processed is Go unit evidence from a read; it does **not** fulfil AC-R4-1 automatically. AC-R4-1 (a public-API test of aliveness) and the READ rows stay **pending** under ADR 0033 D13, for the Owner to decide.
4. **Oracle wording.** Pekko defines this behavior (`Behavior.scala:297-306`); an earlier wording "no Typed oracle" meant that no Typed scenario has been executed, not that Pekko has no applicable behavior. The differential scenario against the Typed harness stays a PORT-GAP (#1046). No JVM run, parity or V3 is attributed.
5. **Negative check and gates of the correction** are recorded in the PR of the correction, with SHA, command, environment, result and exit code (section 8).

## 8. Negative check and gates of the correction

Base `develop` `595595c`. The gates ran on that base plus the delta of the correction (one test file and `migration/` text), before the commit; the Go files are byte-identical to the committed ones, and the commit adds no Go change after them. Environments: **host** macOS (Darwin 25.3.0) arm64, go1.27.2, golangci-lint v2.14.0 built from `.golangci-version`; **container** `golang:1.27.2-alpine`, linux/arm64, go1.27.2.

**Controlled negative check** (`go test -overlay`, copy of `behavior.go` in a scratch directory; no source file changed, `git status` shows only the intended delta). Command: `go test -count=1 -timeout 150s -overlay <m.json> -run TestEmptyIgnore ./typed/` in `pekko-actor-typed`.

| Mutation (in `emptyReceive`/`ignoreReceive`) | Result | How it is detected |
|---|---|---|
| M1 Empty answers a message Same | exit 1, FAIL | missing `rec:<msg>` |
| M2 Empty answers every signal Same | exit 1, FAIL (30.0 s) | no death pact: `Done` never closes, `dsRecv` limit |
| M3 Ignore answers every signal Unhandled | exit 1, FAIL (30.0 s) | death pact: the later message is never processed, `dsRecv` limit |
| M4 Ignore answers a message Unhandled | exit 1, FAIL | unexpected `rec:<msg>` |
| M5 Ignore answers Terminated Same but other signals Unhandled | exit 1, FAIL | ordinary-signal specs |

**Gates** (all on the tree above):

| Gate | Command | Env | Exit | Result |
|---|---|---|---|---|
| gofmt, vet, tidy | `gofmt -l .`; `go vet ./...`; `go mod tidy -diff` in `pekko-actor-typed` | host | 0 / 0 / 0 | clean |
| Unit tests | `go test -count=20 ./typed/` | host | 0 | ok (14.1 s) |
| Module coverage | `go test -count=1 -covermode=atomic -coverprofile=... ./...` | host | 0 | `pekko-actor-typed` 100.0% (the other modules are untouched) |
| golangci-lint | `golangci-lint run --config ../.golangci.yml ./...` | host | 0 | 0 issues (gocyclo <= 10) |
| Unit tests, no network | `docker run --rm --network none ... golang:1.27.2-alpine sh -c 'go version && go test -count=1 ./...'` in `pekko-actor-typed` | container | 0 | ok (`each_module.sh` cannot run in the image: no bash; one module, the only one with a code change) |
| Architecture | `scripts/check_deps.sh` | host | 0 | ok |
| Headers, global | `.github/scripts/check-headers.sh` | host | **1** | 7 findings, the same files as section 3, none under `pekko-actor-typed`, none in this diff; not caused here and not fixed here |

**Inherited by equivalence, not repeated:** `go-sdk-update.sh --check` (no `.go-version`, `go.mod` or `go.work` change), `make parity-guard` (no exported symbol or manifest change).

**Not applicable:** diff coverage of production. No production line changed (test file and text only), so there are no new or modified production lines to measure; the 83% exception of section 3 belongs to `behavior.go` of #1264 and is **not** carried over to this delta. **Not run:** GitHub Actions (no credit), `-race`, workbench, wide campaigns.
