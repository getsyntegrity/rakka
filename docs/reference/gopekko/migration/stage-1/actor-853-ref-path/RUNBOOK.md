# RUNBOOK: a Ref reports the ActorPath it was spawned at (unit actor-853-ref-path)

Status: **prepared.** Sources pinned: Pekko `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (v1.7.1), kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`. Base of K: `develop` `db614b877a1c5ab7f00bf6364caa2c29ca1ac044`. Scope: `docs/scopes/853-ref-reports-actor-path.md` (PR #1287; G2 was granted on head `82c97fc7690d85c1b9cb01c85f0372d57f663595`; head `360951389190f36ea85fcad25f84e0970e9b6586` was reviewed before the post-G2 amendment below; the head cited now is `e6efd6f65848a433ac2b197e4d7439f309523a76`; the scope PR is **not merged**, so it is cited by SHA: `git show e6efd6f65848a433ac2b197e4d7439f309523a76:docs/scopes/853-ref-reports-actor-path.md`). Delivery is three draft PRs, each `Refs #853`, never `Closes`, no merge without the Owner. The PRs are stacked in the order **K, then C, then R**; C and R are not independent of K, and R is written by the orchestrator after independent QA on the exact SHAs.

## Owner G2 approval and conditions (verbatim, Spanish, message of 2026-10-29)

> Apruebo G2 de #853 sobre la propuesta presentada para #1287 en 82c97fc7690d85c1b9cb01c85f0372d57f663595, con estas condiciones:
> - D-1 aprobado.
> - D-6: apruebo evidencia solo de Go para P1-P4. Ejecutá también el escenario Classic adicional durante el kit. Si contradice el contrato propuesto, detené la parte afectada y reportalo.
> - D-7 aprobado: #853 antes de #894.
> - G-1: aprobado PathOf(h Handle) (ActorPath, bool).
> - G-2: aprobado devolver el path propio del handle de otro System. No presentes igualdad de Address como prueba de pertenencia a una instancia concreta de System sin verificar esa garantía.
> - G-3: aprobado, incluyendo guardianes. Acepto diferir el path de Terminated.Who() y del hijo fallido obsoleto; documentá explícitamente que esos handles devuelven cero, false.
> - G-4 y G-5 aprobados: kit y división K/C/R. Podés subdividir para cumplir el presupuesto autoral si el contrato no cambia.
> - G-6: acepto tests sin aislamiento de red en macOS y el fallo preexistente de headers, condicionado a comparar contra la base vigente y detenerte ante errores nuevos.
> Apruebo el path inmutable inicializado antes de publicar la celda, sujeto a verificar todas las vías de publicación y lectura. Antes de modificar idHandle, revisá sus comparaciones y usos como clave. Agregar el path no debe alterar identidad ni igualdad. No apruebo ocultar un fallo de construcción de path devolviendo cero para un actor válido. Demostrá la invariancia de los nombres aceptados y definí el tratamiento explícito del fallo defensivo; si exige cambiar el contrato, consultalo antes de implementarlo. Registrá esta aprobación y sus condiciones en #1287, obtené QA del delta y continuá con kit, implementación y QA independiente por SHA exacto. Sin otra autorización para los pasos aprobados. Sin merge, cierre de issues, CI remoto ni -race. No implementes #894 ni el índice de #1020.
**Post-G2 Owner decision (2026-10-29)**, made after G2 and not part of it, not an earlier authorization: "Autorizo ampliar Allowed files para incluir las dos líneas de cell_release_internal_test.go, exclusivamente para reforzar el guard relacionado con esta unidad. Registrá la ampliación como decisión posterior a G2, sin afirmar autorización previa." Recorded in the scope at `e6efd6f65848a433ac2b197e4d7439f309523a76`; PR C's two-line edit of that file is now within the Allowed files.
Accepted exceptions (G-6), each conditioned on a comparison with the CURRENT base at the final SHA and a stop on any new error: (1) unit tests run without network isolation on macOS (`unshare -rn` does not exist there; the exit code of the attempt is recorded); (2) `.github/scripts/check-headers.sh` exits 1 on files that already failed on the base (seven at develop `db614b87`). The Dev runs the script on the final tree and on a detached worktree of the current `origin/develop`, and compares the outputs; any new error stops the work.
The three conditions that are STOP gates for the code are analysed in `RULEBOOK.md` sections 5 (a), 6 (b) and 7 (c); the extra Classic scenario (D-6) is in `pekko-repro/`.
Owner instruction given in the working session on 2026-10-29 (not a GitHub comment; recorded here): the Dev may split the work into PRs so that each stays at or under 400 authored changed lines, keeping criteria and traceability. A split that changes the contract requires a new G2; this split keeps the scope contract unchanged.

## PRs, order and bases

| PR | Branch (base) | Tasks | Content |
|---|---|---|---|
| K | `docs/853-kit-package` (`develop`) | T1 | this package: `manifest.tsv`, `RULEBOOK.md`, `RUNBOOK.md`, `pekko-repro/{Probe.scala,RUN.md,observed.txt}` |
| C | `feat/853-ref-path` (`docs/853-kit-package`) | T2, T3, T4 | `ref.go` (`PathOf`), `cell.go`, `spawn.go`, `hierarchy.go` (`path`, `idHandle.path`, `childPath`), `stop.go` comment, `ref_path_internal_test.go` (N1-N4, P1-P12), `pekko-actor/README.md` (N7, limits) |
| R | `docs/853-execution-record` (`feat/853-ref-path`) | T5 | `EXECUTION-RECORD.md`, one `migration/cost-log.tsv` row, gate evidence bound to the SHA of C after independent QA (written by the orchestrator, not by this Dev) |

Order K -> C -> R; each later PR is stacked on the earlier branch and states it in its body. When an earlier PR merges the later one is updated by a normal merge of the target (no rebase of a published branch, no force-push).

## Procedure per PR
1. TDD: write each scenario, watch it fail (RED output kept), implement the minimum, inside the allowed files. RED is observed for N1-N4, P1-P12 and N7 before the implementation; R1-R3 are existing tests and stay unchanged.
2. Gates (commands in `AGENTS.md`, "Go"), once per final tree, for the affected module `pekko-actor`:
   - `cd pekko-actor && go test -count=1 ./...` with network isolation where the platform allows (`unshare -rn` is Linux only: state it, record the exit code of the attempt, run normally).
   - `gofmt -l pekko-actor`.
   - `golangci-lint` (gocyclo at most 10), installed per `AGENTS.md`: `GOBIN=$PWD/bin go install github.com/golangci/golangci-lint/v2/cmd/golangci-lint@$(<.golangci-version)` and run from `bin/` as `cd pekko-actor && ../bin/golangci-lint run --config ../.golangci.yml ./...`; check `bin/golangci-lint version` reports the `.golangci-version` tag built with the `.go-version` Go.
   - `scripts/check_deps.sh`.
   - `.github/scripts/check-headers.sh`, compared with a detached worktree of the CURRENT `origin/develop` (G-6).
   - Coverage by the `AGENTS.md` per-module recipe over every workspace module (`scripts/each_module.sh`, profiles in a `mktemp -d` directory outside the repository), then `scripts/check_coverage.sh "$COV" 85`.
   - Diff coverage at least 85%: `pipx run --spec diff-cover==11.0.0 diff-cover "$COV"/*.xml --compare-branch=origin/develop --fail-under=85`.
   - `.github/scripts/go-sdk-update.sh --check`.
   - `make ci-fast` (what CI runs per `AGENTS.md`; run locally, never remotely).
   - Additions from `docs/quality-gates.md`, labelled as additions to the `AGENTS.md` list: `go vet ./...` and `go mod tidy -diff` in `pekko-actor`; `git diff --check`.
   - `make parity-guard` is not applicable (persistence adapters only). `-race` only when the Owner explicitly asks: not run.
3. Differential check for shared code: `Spawn`, `SpawnAnonymous`, restart and watch behavior share `commitCell`, `cell.identity()` and `idHandle` with the new path. The same inputs run on a scratch copy of the base commit (`mktemp -d`, outside the repository, never committed) and on the change; stdout, stderr and exit code are compared; any unrequested difference is a regression to fix and pin with a test.
4. Record each gate with the pushed SHA, command, environment, exit code and output tail. Remote CI is not run; attribute no remote result.
5. QA reviews each PR on its final SHA; every commit after the gates ran invalidates the evidence.
6. PR size: authored changed lines (additions plus deletions, excluding `go.sum` and generated files) at most 400 per PR, stated in the PR body.

## Stops
A file outside the allowed list must change; `pekko-actor` needs another export or a method on `Ref[T]` or `Handle`; a comparison or keyed use of `idHandle` that the path would change (`RULEBOOK.md` section 6: did not fire); a path found missing or written after publication (section 5: did not fire); a defensive-failure treatment that changes the contract (section 7: did not fire); a Classic result that contradicts the contract (`pekko-repro/RUN.md`: none did); a scenario fails and is not a Go defect (run it on Pekko first, in a disposable copy as in PR #1292 `RUN.md`, never inside `upstream/pekko`); three red iterations; a new `check-headers.sh` error against the current base; PR C projects over 400 authored lines (stop and propose a split of the same contract); a split that would change the contract (needs a new G2). Report to the Owner; never skip, delete or weaken a test.
