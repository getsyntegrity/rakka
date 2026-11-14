# CI EXCEPTION: GitHub Actions credit exhausted (#161, PR #424; also PR #426)

Status: **temporary exception authorized by the Owner on 2026-10-12.** This file does **not** say that remote CI passed. It did not run.

## 1. What is excepted, and why

GitHub Actions does not start any job on this repository because the account's monthly credit is exhausted. Evidence (read, not inferred): every job of run `38308230550` (PR #424, HEAD `28e35b9`), including the single retry the Owner authorized (attempt 2, 2026-10-12T21:33Z), finished in about 2 seconds with no runner and the annotation "The job was not started because recent account payments have failed or your spending limit needs to be increased". No job executed any step, so no technical failure was hidden or attributed to billing.

The Owner authorized integrating on the basis of **reproducible local validation of the exact candidate**, keeping every other quality and review requirement. Remote CI on `develop` must be validated when the credit is renewed (section 5).

## 2. What replaces it, and what does not

| Item | Replaced by | Not replaced |
|---|---|---|
| Required checks `go`, `migration-kit-ext`, `ci-ok` of `develop` | local execution of the commands the workflow defines, with versions, logs, exit codes and hashes, kept outside the repository next to the evidence and summarized in the PR | a green remote check: there is none, and none is claimed |
| Linux runner | not replaced: all local runs are darwin/arm64 | Linux evidence (a Go campaign on Linux, the nightly workflow) |
| Pekko parity | not replaced and **not claimed**: no sbt oracle was run | any statement of parity with Pekko |

Three things are kept separate and none substitutes for another: **Go functional validation** (tests, coverage, lint, judge), **execution of the migration kit** (`EXECUTION-RECORD.md`: it ported `PromiseRefSpec`; the runtime is manual), and **Pekko parity** (not demonstrated).

## 3. Branch protection of `develop`

Configuration before the exception (saved verbatim as `develop-protection-BEFORE.json` and `develop-required-status-checks-BEFORE.json` next to the evidence; sha256 `9fb15e3083d06c52db7ffd6c891c8df949ccd7a253209641c35996d0455f3fb8` and `ee1ddb2b3ce1a8e24453dfaf06aa369fecb3e6bbb9d64d9daa13ee5f784b4811` (the required-status-checks object alone)):

- required status checks: `go`, `migration-kit-ext`, `ci-ok` (GitHub Actions app id `15368`), `strict: true`;
- pull request reviews: required count `0`, dismiss stale reviews `true`;
- required conversation resolution `true`; force pushes, deletions and signatures: not allowed or not required as saved; `enforce_admins: false`; no rulesets.

During the exception only the three required status checks are removed; every other protection is left as it was. The change and its restoration are executed with these calls (`gh api`, repository `getsyntegrity/gopekko`, branch `develop`):

- remove: `PATCH repos/.../branches/develop/protection/required_status_checks` with `strict: true` and `contexts: []`;
- restore: the same call with `contexts: ["go","migration-kit-ext","ci-ok"]` and `strict: true`.

## 4. Scope of the exception

Only the integration of PR #424 and of PR #426 by squash. It does not enable extra spending, does not retry Actions, and does not authorize skipping a technical failure or a pending gate. A gate that does not apply to a commit must say so in the validation record; a gate that was not executed is reported as not executed.

## 5. Closing the exception

When the credit is renewed: (1) restore the required checks of section 3, (2) run the CI of `develop` in the remote and compare with the local record, (3) record the result here or in the PR, (4) only then call the integration validated remotely. Until then the integration is "validated locally, remote CI not executed".
