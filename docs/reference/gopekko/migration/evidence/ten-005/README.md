# TEN-005 evidence (#393)

> **Historical (Owner scope change, 2026-10-17):** goPekko has no tenancy module. References to tenancy, `tenancy.Owner` or the tenant guard in this file describe the scope at the time and are not current requirements.


Nature of the unit: design decision (ADR), own extension. Deliverable: `docs/adr/0016-tenant-identity-and-isolation-model.md`,
status proposed. No code, no contract of ADR 0015 or issue #161 was touched.

## Sources and pins

| Item | Value | Check |
|---|---|---|
| Base | `origin/develop` `77a86d5` | `git fetch origin`; worktree created from it |
| Base CI | run 38295885594 on `77a86d5`: failure in `judge-go`, `judge-selftests`, `ci-ok`; previous develop `ci` run on `9db4e76` succeeded | `gh run list --branch develop` |
| migration-kit | `cf91c9d5068d9aaf95a36164169f08c3e636c909` | `git submodule status` in the main checkout (the worktree submodule is not initialized, so it was read from the main checkout) |
| upstream/pekko | `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (v1.7.1) | same command |
| Pekko tenancy | none: no occurrence of `tenant` or `multitenant` in `upstream/pekko` | `rg -il 'tenant\|multi-?tenan'` over the tree returned no files |

Read, not edited: `AGENTS.md`, `CLAUDE.md`, issue #393, issues #48 and #123 (execution rule), ADRs 0000, 0008, 0010, 0014, 0015,
`docs/judge.md`, `migration/stage-1/milestone-1.md`, `pekko-tenancy/` (scaffold), `pekko-actor/actor/names.go`, kit `README.md`.

## Kit execution

Not executed. This unit ports no upstream code or semantics (Pekko has no tenancy). No kit run, model/role, manifest,
permission set or cost is registered for it. The session that wrote the ADR was a single Claude agent; its token and
wall-clock cost were not measured and are not recorded as if they were.

## Invariants and their verification

Seven proposed invariants (TI-1 to TI-7) and the conformance approach are in the ADR, sections "Isolation matrix" and
"Pending hypotheses and how to verify them later". None is implemented or has a scenario yet; no result is claimed.

## Pending

- Issues TEN-001, TEN-003, TEN-006, PER-001 and PER-007 must link to the ADR (closure evidence of #393). Not done here:
  it needs a reviewed and pushed ADR; coordinator action.
- Owner acceptance of ADR 0016 (all decisions D1 to D9 are proposals).
- Final-SHA review and CI: see the PR.
