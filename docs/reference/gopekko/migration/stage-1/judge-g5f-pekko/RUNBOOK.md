# RUNBOOK: M66, a Pekko mutation for `G5.f`

Status: this is the execution plan of a unit run by one responsible. It authorizes nothing beyond the Owner's brief (a bounded unit, draft PR, no merge, #461 stays open). No agent is launched, so there is no token budget, no launcher and no permission template; the kit's package for this unit is `RULEBOOK.md`, this file, `manifest.tsv` and `PACKAGE-HASHES.tsv`, and the execution record `EXECUTION-RECORD.md`.

## Order
1. **Baseline.** Clean tree at `f9a8838`; `git rev-parse HEAD HEAD^{tree}`; sha256 of the byte-identical files (RULEBOOK section 5); `python3 -I scripts/check-judge-matrix.py` result (CURRENT 18/18, `pending G5.f`).
2. **Disposable Pekko copy** (outside the repository): `git -C upstream/pekko archive` of `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` to a tar, extracted in `$D/pekko-copy`, then `git init`, one commit, tag `v1.7.1`. Check `git rev-parse HEAD:actor` and `HEAD:testkit` against `upstream/pekko` (the runner's preflight does it again).
3. **Define M66** in `judge/mutate.py` and generate the patch with `python3 -I judge/mutate.py patches <copy>`; the diff must add only `judge/mutations/M66.patch` (the 65 others unchanged). Check that the patch applies to a clean copy and that the edits match once.
4. **Compile check and a first look** of the patched Pekko is the run itself (the runner reports `compile-error` if it does not compile; that is a result, not retried by changing the design).
5. **Commit `judge/`** (`mutate.py`, the patch) so the run is not `+dirty`, and run `python3 -I judge/mutate.py run <copy> $D/work M66` in the background, in an empty work directory. One run only; no repetition looking for a different outcome; an unexpected outcome is investigated first.
6. **Verify** the row: scenario `S22`, mode `concurrent`, 30 runs, outcome, fired rules, `expected_hit`, patch sha256, comparator commit; read the kept logs (not only the row) and the `G5.f` message.
7. **Registration and checks:** `docs/judge.md`, `milestone-1.md`, `docs/actor-core-parity.md`, `judge/selftest_calibrate.py` (only the enumeration the row changes); the judge selftests; `check-judge-matrix.py` (expected: `G5.f` observed, the Pekko counts +1, the Go lines unchanged: CURRENT 18/18 and benign 1/1); byte-identical files against the baseline; `ci-fast`, gofmt/vet are unaffected (no Go change) but `go run .github/scripts/ci-fast.go` is run once.
8. **Delivery:** draft PR with the Pekko source, the mutation, the results and the limits; #461 updated, not closed; no merge; no tag push (a Pekko mutation row does not create a Go campaign tag; the PR says which commit produced the row).

## Environment
sbt with Java 11 on PATH and `JAVA_8_HOME` for the harness (`~/.sdkman/candidates/java/8.0.504+1-zulu`), as `judge/run-pekko.sh` and `mutate.py` set it; the dependency caches `~/.ivy2` and `~/.sbt` exist, so a first compile of the copy is local work (minutes). Everything is local; nothing runs in GitHub Actions beyond the repository's fast check.

## Stop conditions (reported, not worked around)
`compile-error`, `harness-error`, `timeout`, `incomplete` or `survived` for M66 (a result to report, with the logs read); a preflight refusal (pin, tree or `+dirty`); a changed byte-identical file; any need to touch the Go runtime or the Go campaign inputs. A new semantic choice (a second mutation design, an expectation other than `G5.f`) returns to the Owner.
