# RESUME: Supervision U8, judge scenarios S13, S14 and S16 in the Go gate (#175)

State: **delivered and closed.** The package (`RULEBOOK.md`, `RUNBOOK.md`) was approved and merged (`d5b831b`, PR #473); the code was merged by squash `3054971` (PR #474); #175 was closed as completed by the Owner's decision after the matrix check passed on `develop`. Gates, campaigns, the S14 deviation, the GMoverlap investigation, the review, the token accounting and the post-merge verification: `EXECUTION-RECORD.md`. Base `develop` `d5b831b7cc85b4029212d26480087ffd92d4a671`; kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`; upstream `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (`v1.7.1`).

## Approved hashes (kept as the record of what ran)
`RUNBOOK.md` sha256 `ca0686ff530cb4cb35bab950a4b399d6e02c0f7658c530b4d79064e3f6613be9`. The other seven hashes are in RUNBOOK section 9.

## Evidence retention
The Go campaign commit `e543c6a2e1bdbff73cca4a202a41c3f2223aa37f` is retained by the tag `evidence/go-campaign-e543c6a`, so `scripts/check-judge-matrix.py` passes on `develop`.

## Pending (tracked in #461, no new issue)
GMoverlap (data races in `cell.go` under `-race`, predates U8); S22 not in the Go gate; Pekko mutations without a Go analog (M10, M11, M58, M13); no calibration row for S13, S14 and S16; the fixes after the single review were not re-reviewed.

## Not claimed
Pekko parity, S22 in the Go gate, U9, #450, the module-close campaigns of #461.
