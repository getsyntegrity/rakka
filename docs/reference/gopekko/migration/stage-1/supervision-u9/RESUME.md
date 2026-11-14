# RESUME: Supervision U9, `loggingEnabled` and `logFailure` (#451)

Status: PROPOSED package, **awaiting the Owner's approval**. Nothing is implemented; no agent has run; `pekko-actor/`, the judge and `upstream/pekko` are unchanged by this package. Base `develop` `ad98f7fc3e7a529d5ebc156daf1eda0407ff774d`; kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`; upstream `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (`v1.7.1`).

## Read in this order
1. `RULEBOOK.md` (the Pekko contract with line evidence, the Go facts, decisions D1-D9, tests L1-L12, mutants ML1-ML6, the dependency footprint and the seed, the effect on the matrix).
2. `RUNBOOK.md` (flow, budget, procedure, surfaces, commands, hashes).
3. This file for the RUNBOOK's own hash.

## Hash of the RUNBOOK (cannot be inside it)
`RUNBOOK.md` sha256 `877877864a90340a652376de09d95ad3723de3c09344acb339b6ba79758c8870`. The other eight hashes are in RUNBOOK section 9.

## Next step
The Owner approves the hashes and says to run. Then: merge the package PR, create `feat/actor-logging-u9`, seed the dependency, follow RUNBOOK section 5. Budget: stop at 450,000 tokens, ceiling 560,000, 45 minutes of agent time; the central estimate (about 440,000) is under the stop; one review at most, launched only if the implementer used at most 250,000 tokens.

## Not authorized by this package
Merge, native review, a second review, closing #451, pushing an evidence tag, #450, or the module-close campaigns of #461.
