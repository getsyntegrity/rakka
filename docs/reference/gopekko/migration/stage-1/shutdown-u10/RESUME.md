# RESUME: ordered minimal termination of the system, U10 (design #192, implementation #193)

Status: PROPOSED package, **awaiting the Owner's approval**. Nothing is implemented; no agent has run; `pekko-actor/`, the judge and `upstream/pekko` are unchanged by this package. Base `develop` `ced37b9f97fdfd7786162e75ab2f4349861e8d32`; kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`; upstream `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (`v1.7.1`).

## Read in this order
1. `RULEBOOK.md` (the Pekko contract with line evidence, the goPekko facts, decisions D1-D9, invariants I1-I9, tests SH1-SH11, mutants MS1-MS8, the judge and matrix position, the differences from Pekko).
2. `RUNBOOK.md` (flow, budget, procedure, surfaces, commands, hashes).
3. This file for the RUNBOOK's own hash.

## Hash of the RUNBOOK (cannot be inside it)
`RUNBOOK.md` sha256 `8f297206f85715173af74146546b3dc4a8c5aca87ecb6cfd6dc630059f3a7f81`. The other eight hashes are in RUNBOOK section 9.

## Next step
The Owner approves the hashes and says to run. Then: merge the package PR, create `feat/actor-shutdown-u10`, follow RUNBOOK section 5. Budget: stop at 450,000 tokens, ceiling 560,000, 45 minutes of agent time; the estimate (295,000 to 435,000, central about 360,000) is under the stop; one review at most, launched only if all implementer runs together used at most 250,000 tokens.

## Not authorized by this package
Merge, native review, a second review, a judge run or campaign, an evidence tag, closing #192 or #193, Persistence, `CoordinatedShutdown`, cancellation of handlers, or the module-close campaigns of #461.
