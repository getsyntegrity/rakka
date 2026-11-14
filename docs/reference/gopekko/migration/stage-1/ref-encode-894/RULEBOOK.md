# RULEBOOK: encode an exact local reference string (unit ref-encode-894, #894 IDN-09)

Status: **G2 granted by the Owner on 2026-10-29** on #1288 `d65884dbbb5aa5c17b9efa197c43558b0385a6c7` and #1291 `5ac4ae89ecc47a1acbb989fec0d632ac4be04255` (complete message quoted verbatim in the scope). The contract is the scope `docs/scopes/894-encode-exact-ref-string.md` at its final head `3ad7e7230ad14d5f00c5090443255ceab4cf4e18` (#1288, **not merged**) and the token proposal `docs/adr/proposals/0031-addendum-incarnation-token-1186.md` at `33d16d863aabb8abf1323aeff45c8cb337eb266d` (#1291, **not merged**, addendum accepted by the Owner for encoding). Read-only inside any loop (kit `CLAUDE.md` rule 1). Authorized: kit, implementation, independent QA. **Not authorized: merge, issue edits or closure, the #1020 index, remote CI, `-race`.**

## 1. Unit

One exported free function `func ExactRefString(h Handle) (string, bool)` in `pekko-actor/actor/ref_string.go` (no method on `Ref[T]` or `Handle`), a guarded ID counter shared by both ID minters (`nextID`, `nextAskID`) with permanent exhaustion, and the release of the name reservation when an exhaustion panic occurs inside `commit`. Pekko source: `ActorPath.scala:248, :299, :374-378, :449-452`, `ActorCell.scala:390-398`, `ActorRef.scala:188-190` at v1.7.1 (`9a54d1294c3ac7821bbed3c940769c3d5027d1ff`). Kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`. No parse, no resolution, no #1020 index. Manifest: `source` and `target` are the first two columns (kit runner contract); moved rows point at this RULEBOOK.

## 2. Evidence classes

**Executed**: Pekko rows of `migration/stage-1/identity-oracle/pekko-repro/observed.txt` (rows 15, 17, 23, 183-185), disposable copy, never `upstream/pekko`. **READ**: Pekko source read, not run. **Go-only**: no Pekko equivalent; the Owner accepted Go-only evidence **as the method**: "su cumplimiento requiere ejecutar y revisar las pruebas", so each Go-only row below is met only when its test is run and reviewed: the token form, the epoch, the `("", false)` results, the root and guardian refusal, foreign System handling, counter exhaustion and its permanence, the ID numeric contract (E2, E4, E7-E9, C1-C8 and the Go side of E1, E3, E6). The optional Classic serialize-after-stop scenario (E6) is **not run: proposal only** (the Owner did not accept it). K contains no probe. Go is never run on a JVM; no parity is claimed.

## 3. Invariants (each is a test)

- **Token.** String = `ActorPath.String()` + `#` + `epoch-number`, exactly one `#`. epoch = 16 lowercase hex digits from 8 bytes of `crypto/rand`, read once per process (`sync.OnceValue`), never all zero (redrawn), shared by all Systems of the process. number = the decimal numeric part of the ID `actor-<n>`, 1 to 2^64-1, no sign, no leading zero. Token at most 37 characters. No per-cell field.
- **`("", false)` (J8):** nil, typed-nil, nil cell, user doubles, hand-built cells, ID-only handles (including the stale failed child; `Terminated.Who()` is an `ID`), ask endpoints, root, `/user`, `/system` (detected by `p.IsRoot() || p.Parent().IsRoot()`). A terminated original `Ref` keeps its string (no registry lookup, no liveness test). A handle of another System instance is encoded with its own path and Address; Address equality is not proof of belonging to a System. The encoder never emits a tokenless string.
- **Counter.** One `idCounter` shared by both minters: `next()` is a CAS loop that never advances past `math.MaxUint64`; at that value it panics with the unexported sentinel `errIDExhausted` **without changing the value** and stays exhausted; first value 1, so 0 is never issued. `mint(prefix, c)` builds the ID; the process-wide instance is `idSeq` (the scope text calls it `ids`: it was renamed because `hierarchy_internal_test.go:106` already defines `func ids(hs []Handle) []ID` in the package, a compile collision; it is a name only, not a design change); `nextID` returns `idMint()`, whose default is `mint("actor-", &idSeq)`, and `nextAskID = mint("ask-", &idSeq)`.
- **Release (m).** An exhaustion panic inside `commit` releases only its own reservation (a `defer` in `commitCell` calling the existing `rollback`) before propagating; no partial cell; other reservations untouched; the counter stays exhausted.
- **`idMint` seam.** A package-level function variable in `id.go` that replaces the minter (not the counter); only C7 and C8 swap it, restore it in `t.Cleanup`, never use `t.Parallel`, and run while no other System spawns. No per-System seam (it would edit `system.go`).
- **Spawn, SpawnAnonymous, Ask:** ID text and order unchanged (`actor-<n>`, `ask-<n>`).

## 4. Inventories (at develop `899d93e`)

- **Callers of the minters:** `git grep 'idSeq\|nextID\|nextAskID'` over `pekko-actor*`: `id.go:14-18` (counter, `nextID`), `ask_endpoint.go:38-40` (`nextAskID`) and `:53` (its call), `hierarchy.go:156` (`r.assignID(nextID())`, unchanged); no test references them.
- **Builders of the string:** none exist; after the change only `ExactRefString` builds one.
- **`hierarchy.go`:** `reserve` `:105-121` (entry `:118`, `n.reserved++` `:119`), `rollback` `:125-135` (frees only the entry with `rec == nil` and the same token; calls `readyLocked` `:190-196`, then `schedule` `:199-203`), `commit` `:149-169` (lock `:150`, deferred unlock `:151`, `assignID(nextID())` `:156`: before it only reads, after it writes `byID`, `n.out`), `outstanding` `:359-362`. Not edited unless proven necessary; then a stop.
- **`spawn.go`:** `commitCell` calls `commit` at one place; callers `spawn` and `newGuardian` (`guardian.go:123`); the release is the only edit.
- **Path reader:** `PathOf` and the `pathHolder` accessor (`ref.go:96-123`); no method is added to `Ref[T]` or `Handle`.

## 5. Conventions

Headers per `.github/scripts/check-headers.sh` and the `PathOf` precedent; a new Go-only file satisfies the script without a new error. go-specs matchers, `mock.Controller`/`Spy` for doubles, no sleeps, no network, no `-race`. No `//nolint`; cyclomatic complexity at most 10. README per the module contract. Unknown behavior: a greppable `// TODO(IDN-09-edge): ...`.

## 6. Stops

Any deviation from the contract of the scope (the API, the token format, the counter design, the release shape, the Allowed files, the error set); a file outside the Allowed files must change; `hierarchy.go` must change; any new `check-headers.sh` error against the current base; any production behavior change in Spawn, SpawnAnonymous or Ask ID text; three red iterations; PR over 400 authored lines (move README, comments or an independent test file to a follow-up PR, contract unchanged). Report as a concrete contradiction; never skip, delete or weaken a test. The rename of an unexported identifier to avoid a compile collision (`ids` to `idSeq`, see the counter invariant) is not a deviation of the counter design: it is reported in the execution record and does not stop the run.
