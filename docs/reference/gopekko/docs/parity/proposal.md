# Persistence adapter parity guard: proposal

Status: **D1 to D12 accepted by the Owner on 2026-10-22, with the precisions of section 9.** The acceptance is limited to those decisions: it approves no
exception, creates no ADR 0030 (it stays unwritten) and accepts nothing else. Only slice A is authorized for implementation; slices B to E are not.
**Slice A is integrated** (see "Slice A: integrated" below). Slices B to E have not started.

Synchronized on 2026-10-22 with `origin/develop` `ed9654d01b5d6ebbf13c843aaca7b79496757ec8`. **Integrated there:** U-5 SQLite (#634 to #641, #644); U-1
PostgreSQL (#652 to #657, and #661 which records it), including `pgschema.ErrSchemaGap` (#655) and the durability floor of `Migrate` (#656, kept within the
complexity limit by #666); and U-2 PostgreSQL (#666 to #671, and #672 which records it): the `pgjournal.Delete` transaction with the floor (#667), the journal
tests and scenarios (#668 to #671). **Not claimed here:** anything about U-3 to U-5 on PostgreSQL, and any global equivalence of the adapters: the record of U-2
limits its comparison to the matrix that was executed. Where this document cites a run, it cites what `develop` itself records (author-reported local runs,
ADR 0029, and no remote CI result: every GitHub job was not started because of the billing block); nothing here was executed against PostgreSQL by the guard or
by this review. It is documentation: no code of the guard depends on it, and it can be refreshed again without touching the guard.

Owner objective: a capability added or changed in any persistence adapter (SQLite, PostgreSQL, future ones) is reflected in every applicable adapter in the
same delivery unit, or records an explicit, justified exception. The rule is bidirectional. It would replace the one-way "SQLite parity" rule in `CLAUDE.md`.

## 1. Starting point

Marks: **V** = read in the current code or docs for this proposal; **R** = reported by a read-only survey, to be re-verified in slice 2. Nothing below was
executed against PostgreSQL.

| Fact | Evidence |
|---|---|
| Only the Journal has a reusable conformance suite (`journaltest`). SnapshotStore and CheckpointStore have none. | R: `pekko-persistence/journaltest`; no `snapshottest`/`checkpointtest`. The suite now runs on `pgjournal` too (next rows). |
| The PostgreSQL smoke test and the U-1 migration cases **ran on a real PostgreSQL 17.6, per the record `develop` keeps** (author-reported, ADR 0029): L1, with the image pinned by digest. An earlier version of this document said the smoke test had not run; that was true of the README it was read from and stopped being true when U-1 PostgreSQL was integrated. The old README is not evidence about the present. | V: `gopekko-testint/README.md` on `1e42780` (rows "U-1 migrations, PostgreSQL half" and "Anything run against a real engine"). |
| **U-2 PostgreSQL is integrated and recorded as executed at L1** on PostgreSQL 17.6: `journaltest.Run` on `pgjournal` (35 cases, none skipped), scenarios shared with SQLite, and the floor of `Write` and `Delete` observed inside the transaction. The record limits the comparison with SQLite to the matrix that was executed ("not a global equivalence of the adapters") and says the floor is a configuration read, not a demonstration that an acknowledged write survives a crash. `Journal` therefore has a conformance suite run on both engines. | V: `gopekko-testint/README.md` row "U-2 journal, PostgreSQL half" and `git log origin/develop` (#666 to #672). An earlier version of this document, written when that row still read "candidate, not integrated", described U-2 as not closed; that was true then and is not now. |
| U-3 to U-5 (snapshots, checkpoints, recovery, failures, durability) have no PostgreSQL counterpart on `develop`: they are SQLite only. | V: `README.md` rows U-3 to U-5 and "Anything run against a real engine". |
| Public surfaces are nearly the same (`New`, `Config`, `ErrInvalidConfig`, schema `Version=3`, migrations `0001` to `0003`). | R. |
| **Integrated** (`develop` `a44d390`): `sqliteschema.ErrNotDurable` is the public name of the SQLite durability refusal, an alias of the internal `sqlitetx.ErrNotDurable`. | V: `pekko-persistence-sqlite/sqliteschema/sqliteschema.go:57-63`. Before #641 it was internal only; this document had described that state as a candidate. |
| **Integrated:** U-5 SQLite, evidence recorded by `develop` at L1 and L2 for SQLite only. | V: `gopekko-testint/README.md` rows for U-5 (#634 to #641, #644). The guard does not read it; the manifest will cite it in slice B. |
| PostgreSQL has no counterpart sentinel for the durability refusal. | V: no `ErrNotDurable` in the PostgreSQL module. |
| **PostgreSQL out-of-order migrations (D9) are no longer a gap on `develop`:** `pgschema.ErrSchemaGap` exists and `Migrate` refuses a migration applied after a later one, as SQLite does (ADR 0028 D5). Integrated by #655. The capability is still to be entered in the manifest, with its evidence, in slice B. | V: `git grep ErrSchemaGap origin/develop -- pekko-persistence-postgres` (the error, `gap_test.go`, the README); `git log origin/develop`. |
| **The `synchronous_commit` floor (D10) is no longer missing on `develop`:** `pgschema.Migrate` raises it inside its transaction (#656, restructured by #666) and `pgjournal.Delete` now runs in one transaction that sets the floor first (`syncCommitFloor`, then the delete, then commit; #667). Both were gaps when D10 was accepted; neither is described as one anymore. Their evidence for slice B is what `develop` records, not a run made by the guard. | V: `git show origin/develop:pekko-persistence-postgres/pgjournal/journal.go` (`Delete`) and `.../pgschema/pgschema.go` (`durabilityFloor`); `git log origin/develop`. |

## 2. Three separate questions per capability

The earlier draft used one status (`implemented | exception | not-applicable`). That mixes three different things and made "implemented but never run
against the real engine" look like a functional exception. The manifest keeps them apart:

| Axis | Values | Meaning |
|---|---|---|
| **Obligation** (per capability, per adapter) | `M` mandatory, `O` optional (declared), `E` engine-specific (names the common obligation it serves); `applicable` or `not-applicable` + contract clause | What the contracts require of that adapter. |
| **Implementation** | `implemented`, `absent`, `divergent` (differs from the contract or from the sibling, with reference), `undetermined` (the contract does not say; needs an Owner determination) | What the code does now, with `file` + symbol references. |
| **Validation** | evidence level reached **and whether it ran** (see below), plus the test/case reference | What has been shown. |

Evidence levels (the vocabulary of `gopekko-testint/README.md` and ADR 0029, plus L0 for unit tests):

| Level | What was done |
|---|---|
| L0 | Unit test with doubles/mocks inside the module. Shows the code does what the test says, not what the engine does. |
| L1 | Logic on the real engine (files for SQLite, a server for PostgreSQL). |
| L2 | The client process is killed (SIGKILL); acknowledged writes checked after reopening. |
| L3 | The engine process or container is stopped hard and restarted on the same data. PostgreSQL only; **no independent SQLite counterpart.** |
| L4 | Power loss. **Unverified in every adapter** and stays so. |

Each validation entry states `written`, `ran` (with date and commit of the run) or `planned`. A test that exists but was never run is `written`: it proves
that the intended coverage exists, **never that it ran**, and it is not execution evidence. No PostgreSQL level above L0 is described as executed anywhere in
this document.

**Required level per capability and adapter.** Levels are not interchangeable: a run at L3 does not satisfy a requirement of L2, and L1 does not satisfy L2.
Each manifest entry names the single level it requires (`required_level`, one of L0 to L3) and is satisfied only by a `ran` entry at exactly that level.
**L4 is a physical guarantee that this environment cannot demonstrate.** It cannot be a required level (the guard rejects it as a structural error), it is
outside what `--closing` can demand, and it is listed as `unverified` in every adapter.

**Applicability.** An applicable mandatory capability never becomes `not-applicable` to avoid a gap. `not-applicable` needs a contract clause (SPI text or an
accepted ADR decision) that permits the distinction; the guard checks that the clause reference resolves and the reviewer checks that it says so.

**Evidence is bound to a commit.** A `ran` entry records the commit of the run. If any implementation file referenced by that capability and adapter changed
after that commit, the evidence is **stale** and counts as pending until the run is repeated; evidence from another commit is never current automatically.

An **exception** is a separate record, only for a deliberate, approved deviation from an applicable obligation. An existing implementation whose real-engine
validation is pending is **not** an exception: it is `implemented` + `validation: L0 ran, L1 planned`. A real divergence with no approved exception is an
**open gap**, tracked and visible, never silently green.

## 3. Initial matrix (corrected; to be encoded in slice 2)

Both columns describe **integrated** `develop` (`ed9654d`) only; SQLite includes U-5, PostgreSQL includes U-1 and U-2. Evidence marked "recorded" is what `develop` itself records (author-reported local runs, ADR 0029), not something the guard or this proposal re-ran; each cell is turned into a reference with commit and date when the manifest is written (slice B). Open work of other sessions is not in the matrix.

| Capability | Lvl | SQLite implementation | SQLite validation | PostgreSQL implementation | PostgreSQL validation |
|---|---|---|---|---|---|
| `schema.migrate / check / checksum / versioning` | M | implemented (R) | L1 ran (V1 cases) | implemented (R) | L0 ran; L1 recorded by `develop` (U-1 PostgreSQL, PostgreSQL 17.6, linux/arm64 only) |
| `schema.no-out-of-order-apply` | M (D9) | implemented: `ErrSchemaGap`, "never applied out of order" (V: ADR 0028 D5) | L1 ran (R) | implemented since #655: `pgschema.ErrSchemaGap`, a later migration applied while an earlier one is missing fails and rolls back (V: `git grep` on `develop`). It was a divergence until then | L0 ran (`gap_test.go`); L1 recorded by `develop` (U-1 PostgreSQL found this defect and corrected it) |
| `journal.write-unit-atomicity / replay-order / highest / delete / empty-unit / uncertain-outcome` | M | implemented | L1 ran via `journaltest` (U-2) | implemented (R) | L0 ran; `journaltest.Run` on `pgjournal` over a real server and shared scenarios are integrated (#668 to #671); L1 recorded by `develop` as executed (U-2 PostgreSQL), limited to the executed matrix |
| `snapshot.*` (latest selection, save-replace, delete, undecodable is error) | M | implemented | L1 ran (U-3) | implemented (R) | L0 ran; L1 planned. No shared suite yet for either |
| `checkpoint.*` (read/save/clear, validation) | M | implemented | L1 ran (U-3) | implemented (R) | L0 ran; L1 planned. No shared suite yet |
| `recovery.after-orderly-close` | M | implemented | L1 ran (U-4, integrated) | implemented (R) | planned (U-4 PostgreSQL half not started) |
| `recovery.after-client-kill` (L2) | M | implemented | L2 recorded by `develop` (U-5 SQLite, #636 to #639, #644) | implemented (R) | planned |
| `recovery.after-engine-restart` (L3) | E | not applicable: no separate engine process (contract clause: SQLite is embedded) | n/a | implemented (R) | planned; extra evidence, no SQLite counterpart |
| `durability.write-acknowledged-after-durable-commit` (journal `Write`, snapshot/checkpoint `Save`) | M obligation; mechanism E | implemented: refuses a non-durable database (`ErrNotDurable`) | L1 recorded by `develop` (U-5 SQLite, #634) | implemented: `synchronous_commit` raised to at least `on` in the write tx (V: `pgjournal/journal.go:212`, `pgsnapshot/snapshot.go:95`, `pgcheckpoint/checkpoint.go:26`; ADR 0022 R1, P3) | L0 ran (mock); `Write` floor observed inside its transaction on a real server, recorded by `develop` for U-2 (L1; a configuration read, not a crash guarantee); L3 (additional) planned |
| `durability.journal-delete` | M obligation (D10) | `Delete` runs through `sqlitetx.Run` (V: `sqlitejournal/journal.go`) | U-5 integrated; whether a case covers `Delete` is confirmed when slice B cites it | implemented since #667: one transaction, `syncCommitFloor` first, then the delete, then commit; rolled back through `txcleanup` on failure (V: `pgjournal/journal.go`, `Delete`). It was a bare `pool.Exec` without the floor when D10 was accepted; ADR 0022 R1 names `Write` and P3 names snapshot `Delete`, so the journal case is covered by the Owner's D10 | L0 ran; the floor of `Delete` observed on a real server, recorded by `develop` for U-2 (L1; a configuration read, not a crash guarantee) |
| `durability.schema-migrate` | M obligation (D10) | `Migrate` refuses a non-durable database (V: `sqliteschema.go`) | R | implemented since #656: `durabilityFloor` raises `synchronous_commit` to at least `on` inside the transaction of `Migrate`, preserving `remote_apply`; moved into `lockAndBootstrap` by #666 (V: `pgschema.go`). ADR 0022 R1/P3 do not name DDL; the Owner accepted the floor for it (D10) | L0 ran; the floor read inside its own transaction on a real server, recorded by `develop` (U-1 PostgreSQL, L1) |
| `failure.bounded-cleanup / no masking of the original error` | M obligation; mechanism E | `sqlitetx.Run` owns cleanup (V: `journal.go:162`) | L1 recorded by `develop` (U-5 SQLite, #635) | `txcleanup.Rollback`, bounded by its own timeout (ADR 0025) | L0 ran (`rollback_test.go`) |
| Public name for the durability refusal | not an obligation to share | `sqliteschema.ErrNotDurable`, public on `develop` since #641 | n/a | no counterpart | n/a |
| Power loss (L4) | M obligation | unverified | **unverified** | unverified | **unverified** |

**No approved exception exists in this matrix, and no row of it is a known open gap in the implementation of PostgreSQL on `develop`.** What remains open is
evidence, not implementation: U-3 to U-5 on PostgreSQL have no counterpart, the U-2 comparison is limited to the matrix that was executed, and L2/L3 are not
recorded for PostgreSQL. Those are `PENDING-EVIDENCE`, never exceptions. Required levels: schema, journal, snapshot and checkpoint behavior L1; client-kill recovery L2 for both engines; `recovery.after-engine-restart` L3 for PostgreSQL only; power loss is unverified and never
required.

## 4. Static guard (local)

Manifest: `docs/parity/capabilities.json` (JSON, standard library only), with `schema_version`. Tool: `tools/cmd/parityguard`, run by `make parity-guard`.
Adapters are registered in the same file (`module`, `engine`, test locations).

**Output categories** (each finding has exactly one):

| Category | Example | Exit code |
|---|---|---|
| `FAIL: structure` | Mandatory capability with no entry for a registered adapter; incomplete entry; reference that does not resolve (file, symbol, `Test*`, suite case ID, ADR anchor); unregistered `pekko-persistence-*` module; unregistered or missing adapter; exception with a missing, unresolvable or expired decision reference | non-zero |
| `OPEN-GAP` | Implementation `absent`/`divergent`/`undetermined` for an applicable M capability, recorded with a tracking reference | zero by default; printed in a mandatory summary; non-zero with `--strict` or `--closing <module>` (D8) |
| `PENDING-EVIDENCE` | Implementation exists, validation level not reached or only `written` | zero; always printed; never reported as pass |
| `WARN: impact` | Diff heuristic fired without a declaration (section 5) | zero |
| `INFO` | Valid engine-specific difference | zero |

An `OPEN-GAP` without a tracking reference is `FAIL`. A mandatory capability cannot be turned off to obtain green: the only legal states are
`implemented`, an open gap, or an approved exception. The tool never prints a plain "OK" while gaps or pending evidence exist; it prints the counts.

**What the guard proves and does not prove.** It proves declaration and traceability: references resolve, nothing is missing from the matrix, dates are not
expired. It does **not** prove behavior; a manifest entry is never functional evidence. It also **cannot prove that a human approved an exception.**
An exception needs `decision_ref` (a path with anchor, or a PR/issue URL) and `human_check: "required"`; the guard checks that a local reference resolves
and that the URL is well formed, not that the decision exists upstream or who made it. The field `approved_by` alone is not accepted as proof. A reviewer
must read the decision. The guard prints "approval NOT verified" for every exception.

## 5. Bidirectional rule without proof-by-touch

"Both adapters changed" is never accepted as proof of parity, and "only one changed" is never a rejection.

For a delivery unit that changes functional behavior, the unit adds `docs/parity/impacts/<unit>.json`, one entry per affected capability, with exactly one of:

| Declaration | Required content |
|---|---|
| `requires-equivalent` | The other adapter's matrix entry in the same unit is `implemented` with a reference, or an `OPEN-GAP` with tracking. |
| `already-satisfied` | A reference to the other adapter's code **and** to the evidence (test/case) that shows it. |
| `engine-specific-preserves-obligation` | The common obligation id it serves and the test/case that checks that obligation in both adapters (or the open gap that says it does not yet). |
| `exception` | A reference to an exception record (section 4) or an open pending one. |

`parityguard --base <ref>` maps the changed files to capabilities through the manifest's implementation references. If an adapter's implementation of a
capability changed and no impact entry names it, the finding is `WARN: impact`, not a rejection: the mapping is by path and can false-negative (logic moved
to a shared helper) or false-positive (a refactor). An impact entry that **is** present is validated strictly (`FAIL: structure` if a reference is broken or
the declaration is inconsistent with the matrix). Whether the missing declaration should be a rejection is D7.

## 6. Shared contracts and executable conformance

- **Suites beside the SPIs, as support packages, not new modules.** Extend `journaltest`; add `snapshottest` (in `pekko-persistence`) and `checkpointtest` (in `pekko-projection`, where the SPI
  lives). A mandatory capability field has no zero-value escape. A case that cannot run is skipped with a reason only for level O. For level M it fails.
- **One scenario definition, many adapters.** Each suite exports its case IDs so the static guard can verify `conf` references without running anything.
  `inmem` keeps running them as unit tests. Real engines run them only from `gopekko-testint`, as one table `adapters × suites`. Functional modules keep
  unit tests with doubles. PostgreSQL mock pools cannot run behavior suites, so PostgreSQL behavioral evidence depends on the pending U-1 to U-3 PostgreSQL
  halves and shows as `PENDING-EVIDENCE` until then. (Update 2026-10-23: U-1 to U-3 PostgreSQL are integrated, #652-#657, #666-#671, #702-#704.)
- **Normalization only where the contract admits a difference.** A normalizer registry exists, **empty at the start**. An entry must cite the contract
  clause (SPI doc, ADR decision) that allows the difference. Timestamp precision, key identity, promised order and error classes are **not** normalized
  just because engines differ; a divergence there fails the comparison.
- **Error classes.** Adapter errors map to a common category only through explicit per-adapter checks in the harness (`errors.Is` against that adapter's
  own sentinels). The harness presupposes no shared sentinels: today the schema errors are separate per package and the durability refusal has none in
  PostgreSQL.
- **Cross-adapter comparison.** The same operation script runs against each applicable adapter and compares public-API observations: stored data, per-unit
  atomicity, promised order, contractual errors, recovered state.

## 6b. Future engines (MySQL, Redis, Cassandra, others)

**Scope today: only PostgreSQL and SQLite are supported.** No other engine is built, planned or registered, and the manifest holds no placeholder entry for
one. This section is a design constraint, not work: the guard and the suites must not hardcode those two names, so a later engine (MySQL, Redis, Cassandra or
any other) is a new row and not a redesign. Nothing in the tool, the manifest schema or the suites names SQLite or PostgreSQL; they are only the first two rows.

When a new adapter module appears (`pekko-persistence-<engine>`):

1. **Discovery is automatic.** The guard enumerates `pekko-persistence-*` modules from `go.work` (the same source as `scripts/list_modules.sh`). A module
   that is not registered in the manifest is `FAIL: structure` from its first commit, so an adapter cannot land unseen.
2. **Every mandatory capability is required by default.** Registering the adapter creates no free pass: each M capability must be `implemented`, an
   `OPEN-GAP` with tracking, or `not-applicable` with a **contract clause** that justifies it. A missing entry is `FAIL`.
3. **Engines that cannot offer a guarantee do not get it silently dropped.** A key-value or wide-column store may lack multi-event atomic units or a
   durable-commit acknowledgement. That is then `not-applicable` (only if the SPI or an ADR allows it) or an **approved exception** with a reviewable
   decision. It is never solved by skipping a case: for level M the suite fails.
4. **Same suites, one new row.** The adapter runs `journaltest`, `snapshottest` and `checkpointtest` through the same factory, and gets one row in the
   `gopekko-testint` adapter × suite table (real engine only there; unit tests with doubles stay in the module).
5. **Comparison covers it automatically.** The differential harness (section 6) and the durability predicates (section 7) iterate over the registered
   adapters, so a third engine is compared against the others without new comparison code. Fault levels not available for an engine (for example no
   separate process to restart) are declared per adapter, like L3 above.
6. **Engine-specific mechanisms use E.** Each names the common obligation it serves, and that obligation is tested for every adapter.
7. **Dependency direction.** The new module needs its `scripts/deps.allow` entry and, if tests need it, a line for `gopekko-testint`; the guard reports a
   registered adapter without them, since `check_deps.sh` would fail anyway.

Added slice-A self-test: a fixture workspace with a third module `pekko-persistence-<new>` that is not registered must produce `FAIL: structure`; once
registered with only its identity and no capability entries it must still fail, listing every missing mandatory capability.

## 7. Durability comparison (Q1 as clarified by the Owner)

Two checks, never "identical outcomes":

1. **Each adapter against its own guarantees**, after a comparable fault: every acknowledged write is present; an unacknowledged operation is present or
   absent **as its contract allows** (both outcomes accepted, not normalized to one); no unit is partially present.
2. **Equivalence of those guarantees across adapters** under comparable faults: the same predicates hold for both. The comparison is of predicate verdicts,
   not of stored state for unacknowledged operations.

Comparability: L2 (client SIGKILL) for SQLite and PostgreSQL, each within its documented mechanism (WAL and `synchronous` policy for SQLite; WAL flush with the
`synchronous_commit` floor for PostgreSQL). L3 (engine restart) is **additional PostgreSQL evidence** and is not compared with SQLite, which has no
independent counterpart. L4 power loss stays **unverified** and is listed as such.

## 8. Delivery in separable slices (each about 400 changed lines or less, tests with the code)

| Slice | Content | Needs a real PostgreSQL? |
|---|---|---|
| A. Static guard | Manifest schema, `parityguard`, output categories, self-tests, `make parity-guard`. No behavior change. | No |
| B. Manifest content | The matrix of section 3 encoded, gaps with tracking references, zero exceptions | No |
| C. Shared suites | `snapshottest`, `checkpointtest`, case registries, wiring for `inmem` and SQLite | No |
| D. Real-engine table | `gopekko-testint` adapter × suite; PostgreSQL rows enabled as the PostgreSQL halves of U-1 to U-3 are run | Yes, later, by the Owner's unit |
| E. Differential and durability | Comparison harness, predicates of section 7 | Yes for PostgreSQL rows |

A to C can proceed with PostgreSQL unvalidated, and the guard keeps the lack visible instead of simulating green.

**Negative self-tests for slice A** (each a fixture that must produce the stated category, and nothing else):

| Case | Fixture | Expected |
|---|---|---|
| Functional gap | M capability, PostgreSQL implementation `absent`, no tracking | `FAIL: structure`; with tracking: `OPEN-GAP` |
| Missing evidence | Implementation present and referenced, validation `L0 ran, L1 planned` | `PENDING-EVIDENCE`, never `FAIL`, never pass |
| Invalid reference | `impl` points to a symbol that does not exist; `conf` points to an absent case ID | `FAIL: structure` |
| Valid difference, nothing to change in the other engine | Diff touches only the PostgreSQL rollback code; impact entry `engine-specific-preserves-obligation` with resolving references | `INFO`; no demand to touch SQLite |
| Also | Expired exception; exception with an unresolvable `decision_ref`; unregistered `pekko-persistence-*` module; `approved_by` without `decision_ref` | `FAIL: structure` |

## 9. Decisions (accepted by the Owner on 2026-10-22, with the precisions below)

The table keeps the recommendations as proposed. The **Owner acceptance record** after it states what was actually accepted and supersedes the table wherever
they differ.

| ID | Decision | Recommendation | Consequence if approved | Consequence if not |
|---|---|---|---|---|
| D1 | Levels M/O/E plus the three axes (obligation, implementation, validation) | Approve | One vocabulary for every adapter; "implemented, not yet run" is representable without an exception | Back to a single status that mislabels pending validation as an exception |
| D2 | JSON manifest and `tools/cmd/parityguard`, stdlib only | Approve | No new dependency; one more tool to maintain | YAML needs a dependency or a custom parser |
| D3 | Exceptions: `decision_ref` + `human_check: "required"` + expiry (60 days max); the guard states it cannot verify approval | Approve | Reviewable trail; a human must read each decision | Free-text approvals the guard cannot even check structurally |
| D4 | Shared suites beside the SPIs (`snapshottest`, `checkpointtest`) | Approve | Scenario defined once; new modules to maintain | Per-adapter expectations keep drifting |
| D5 | Real-engine adapter × suite table only in `gopekko-testint` | Approve | Keeps the Owner rule of mocks-only in functional modules | Real-engine tests would leak into functional modules |
| D6 | Guard is local-only (`make parity-guard`), not in Actions | Approve | Consistent with `docs/ci-economy.md`; depends on discipline | Adding it to the fast job contradicts the policy |
| D7 | Diff heuristic is a **warning**; a present impact declaration is validated strictly; absence of any declaration for a detected functional change is a warning | Approve as a warning; reviewers enforce the declaration | No artificial edits to the other engine; human review carries the missing-declaration check | If made a rejection: false positives on refactors and engine-only fixes |
| D8 | `OPEN-GAP`/`PENDING-EVIDENCE` do not fail by default but fail under `--strict`/`--closing <module>` | Approve | Development is not blocked; a module cannot be closed with gaps | Always failing blocks the guard itself; never failing hides gaps |
| D9 | Obligation: no adapter applies a migration out of order. ADR 0023 (`line 42`) says migrations apply in ascending order; ADR 0028 D5 makes SQLite reject gaps with `ErrSchemaGap` and calls it contract parity; neither ADR 0022 nor 0023 names a gap error | Treat it as **M**; record PostgreSQL as an open gap, correction in a separate unit; the error name stays adapter-specific | Correctable divergence is visible and tracked | If optional: institutionalizes the divergence |
| D10 | Is the `synchronous_commit` floor required for `pgjournal.Delete` and `pgschema.Migrate`? Today ADR 0022 R1/P3 cover `Write` and snapshot writes only | Owner determines; recommended yes for both, since SQLite refuses non-durable `Delete` and `Migrate` and "success only after durable commit" is the stated rule | PostgreSQL gets two tracked gaps to correct | If no: record the difference as an approved `engine-specific` entry with an ADR decision |
| D11 | Impact declaration (section 5) is a required part of every functional persistence unit | Approve | Reviewers get a uniform statement; small overhead per unit | Parity stays implicit and unreviewable |

| D12 | Engine-neutral design (section 6b), supported engines today: PostgreSQL and SQLite only; adapters discovered from `go.work`, every M capability required for any new engine, `not-applicable` only with a contract clause | Approve | MySQL, Redis, Cassandra or others enter the same guard, suites and comparison as a new row | Each new engine would need bespoke checks and could slip in unguarded |

### Owner acceptance record

Source: Owner message of 2026-10-22 accepting D1 to D12 of PR #642 at HEAD `657506282584b1028f86751a38a50134cc922262`. It is an explicit decision of the
Owner and **only these items are accepted**; nothing else in this document is.

| ID | Accepted with these precisions |
|---|---|
| D1, D2, D4, D5, D6 | As recommended. Shared suites are support packages beside their SPIs, not new modules, unless a new module is justified. |
| D3 | Exceptions carry a reviewable reference, a human check and an expiry of at most 60 days. **This message approves no exception.** |
| D7, D11 | The impact declaration is a **mandatory delivery rule**. The file heuristic only warns. A declaration that is present is validated strictly. |
| D8 | `--strict` and `--closing` fail for mandatory gaps not covered by an exception **and** for required evidence still pending. An exception covers only the scope it expressly names. |
| D9 | No adapter may apply an earlier migration after a later one. PostgreSQL is a recorded open gap; the correction is another unit. |
| D10 | `synchronous_commit` at least `on`, preserving `remote_apply`, is required in `pgjournal.Delete` and `pgschema.Migrate` too. Both are recorded gaps; the corrections are not part of slice A. |
| D12 | Engine-neutral design; PostgreSQL and SQLite are the current adapters. Discovery by module name is a verifiable convention, not proof that no adapter exists outside it. |

Authorized now: **slice A only** (manifest schema, `tools/cmd/parityguard`, result categories and summary, normal / `--strict` / `--closing` modes,
structural validation of references and exceptions, impact warning and validation of present declarations, negative fixtures, `make parity-guard`).
Not authorized: slices B to E, the PostgreSQL corrections, Actions, any PostgreSQL or container run.

### Behavior of slice A (exit codes)

| Exit | Meaning |
|---|---|
| 0 | No `FAIL`; in normal mode open gaps and pending evidence are printed but do not fail |
| 1 | At least one `FAIL`; or, with `--strict` / `--closing <module>`, at least one uncovered `OPEN-GAP`, required evidence still pending, or an `INCOMPLETE` inventory (counted in scope, below) |
| 2 | Usage or input error (unreadable manifest, unknown module for `--closing`, bad flag, `-base` that looks like an option); the verdict is not computed |

`--strict` counts every adapter. `--closing <module>` counts gaps and pending evidence of that module's adapter only, so a gap exclusive to another adapter
does not fail it; an `INCOMPLETE` inventory belongs to the whole manifest and is always in scope; structural failures (`FAIL`) always fail in every mode. The
summary prints the counts of each category, always ends in a `result:` line that describes structure and never coverage, and never reads as a pass while gaps,
pending evidence or an incomplete inventory exist.

### Review corrections to slice A (2026-10-22)

The review of the accumulated candidate changed these points; the Owner decisions above are unchanged.

- **Bootstrap inventory.** The manifest carries `inventory_status`: `bootstrap` or `complete`. The committed manifest is `bootstrap` with no capability: a
  normal run passes as `result: STRUCTURE ONLY`, and `--strict` and every `--closing` exit 1, because a closure cannot be certified from an inventory nobody
  declared complete. `complete` needs at least one mandatory capability **and** an `inventory_review` record (a resolving `decision_ref`, a date, `human_check:
  "required"`), and even then the output reads `INVENTORY CLAIMED COMPLETE (a human claim, not verified)`: one capability passes the same checks as a full
  matrix, so completeness is a judgment about the matrix that only a reviewer can make and the guard states it cannot.
- **Slice B enables closure.** Slice B writes the production matrix from the verified state of `develop` at that moment (D9 and D10 are corrected on
  `develop` as of `1e42780`, so they enter as implemented capabilities with their recorded evidence, not as gaps), and the Owner marks it `complete`. Only
  then can `--strict` or `--closing` exit 0, and only if no gap, pending evidence or `FAIL` remains. Evidence the author recorded as executed enters as
  `ran` only with a commit and a date, never by citing a README.
- **Suite-case references are pending.** No suite registers its cases yet (slice C), so the manifest has no field for them; one is rejected as an unknown field
  rather than trusted. Slice C adds the registry and the field together.
- **Currency of evidence.** A run is compared with the **working tree** (uncommitted edits included), over the implementation files, the files its own
  evidence points to and the contract file. A commit or comparison that cannot be resolved is a `FAIL`, never "current". A listed file that did not exist at
  the run (new, renamed, tracked or not) makes it stale; a deleted one fails its reference. The commit of a run must belong to the history of `HEAD`; one that
  does not is pending (`evidence-foreign-commit`) unless `applies_because` explains why it still applies. Anything outside the listed files, an untracked
  helper included, is not seen, so a current run is not a complete verification.
- **Git as data.** Refs reach git after `--end-of-options`, are resolved to a full object name before any comparison, and file names are read NUL-separated.
- **`make parity-guard`.** `make` returns 2 for any failed recipe, including the guard's own exit 1; the target builds first and prints the real code of the
  step that failed (`go build exited N` or `parityguard exited N`), returns it to the caller and removes its temporary directory on every exit it can trap.
  Tested with a fake `go` under bash and dash. The two unexplained `make` exits of 2 seen during development remain **undetermined**: their output was
  discarded and no later run reproduced them.
- **Not done here.** The `CLAUDE.md` rule below is not applied: changing it is outside slice A. The D9 and D10 corrections were PostgreSQL units of other
  sessions and are integrated on `develop`; this work made none of them.

### Slice A: integrated (2026-10-22)

The Owner authorized the integration by squash of #642, #643, #645, #646, #647, #648, #649, #651, #658, #659, #660, #662, #663, #664 and #665, in that order.
Each was merged with its head pinned (`--match-head-commit`), without `--admin` and without bypass. Where `develop` had moved, the branch was synchronized
mechanically (a merge commit of `develop`, conflicts only in the paths of the chain, resolved with the branch version after proving `develop` held exactly the
previous reviewed content there; never `-s ours`). Each squash was then checked: its diff equals the PR's own diff, it touches nothing outside `tools/`,
`docs/parity/` and `Makefile`, and its chain paths are identical to the pinned head.

| PR | Effective head (reviewed head if different) | Squash on `develop` |
|---|---|---|
| #642 | `dda47b3` | `3842038` |
| #643 | `2908268` | `41a9ac2` |
| #645 | `ab46158` (`82217a9`) | `f6a0999` |
| #646 | `bc53e41` (`2c21b6d`) | `cf1cde3` |
| #647 | `87646dc` (`1ad4831`) | `40ca559` |
| #648 | `8bb165c` (`8e2d7be`) | `924403e` |
| #649 | `38f2c02` (`f184e67`) | `6d98530` |
| #651 | `6735c5a` (`30da130`) | `6e370a6` |
| #658 | `ec20112` (`15247c8`) | `b26819b` |
| #659 | `50a421b` (`e39834a`) | `76d736e` |
| #660 | `91b8069` (`e36194f`) | `8e88632` |
| #662 | `78f6434` (`fc39231`) | `93e8dfd` |
| #663 | `c4aa8ad` (`5de0c02`) | `f9935c0` |
| #664 | `afe7df7` (`61a5ade`) | `1e42780` |
| #665 | `c2acdce` (`59c980e`) | `ed9654d` |

What `develop` holds: `tools/cmd/parityguard` and `tools/internal/parityguard` (standard library only), `make parity-guard`, and a **bootstrap manifest**
`docs/parity/capabilities.json` that registers SQLite and PostgreSQL and declares **no capability**. It is not the inventory. With it:

| Mode | Result |
|---|---|
| normal | `result: STRUCTURE ONLY`; the guard exits 0; it says nothing about functional coverage |
| `-strict` | exit 1: the inventory is incomplete |
| `-closing pekko-persistence-sqlite` and `-closing pekko-persistence-postgres` | exit 1 for the same reason, whichever adapter |

`make` reports 2 for any failed recipe; the real code is the one on the last line the target prints (`parityguard exited N`). Validation was **local only**: all 30
check-runs of the 15 heads (`go` and `label`) carry the annotation "The job was not started because recent account payments have failed or your spending
limit needs to be increased", with zero steps, so no remote CI result exists and none is claimed. The guard packages are excluded from `ci-fast` by policy in any
case, so even a green remote check would not have validated them. The two unexplained `make` exits of 2 seen during development remain **undetermined**.

Not part of slice A, and pending for slice B: the production matrix written from the verified state of `develop` and marked `complete` by the Owner with its
`inventory_review`; each cell as a reference with commit and date (what `develop` records as run enters as `ran` only that way); the suite-case registry and its
reference (slice C); the `CLAUDE.md` rule; real-engine comparison and durability predicates (slices D and E).

No decision changes `CLAUDE.md`, any module, or U-5. After the Owner decides, the rule text below replaces the "SQLite parity" item:

> **Persistence adapter parity (Owner rule):** a creation or change of behavior in any persistence adapter or shared persistence contract carries an impact
> declaration for every other applicable adapter (equivalent implementation, already satisfied with evidence, engine-specific preserving the common
> obligation, or an exception with a reviewable decision). A mandatory capability is never disabled to obtain green. `make parity-guard` checks structure
> and traceability only; it does not prove behavior or approvals.
