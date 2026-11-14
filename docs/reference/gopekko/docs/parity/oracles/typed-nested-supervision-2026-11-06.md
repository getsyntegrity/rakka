# Oracle: nested Typed supervisors, the innermost matching one and the equal-class replacement (Pekko v1.7.1)

- Date: 2026-11-06. Evidence only; this document recommends nothing and changes no ADR.
- Pekko `pekko-actor-typed` 1.7.1 and `pekko-slf4j` 1.7.1 (tag `v1.7.1`, `upstream/pekko` at `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`), Scala 2.13.18, sbt 1.10.7, JDK Temurin 11.0.32+9, default fork-join dispatcher, logback-classic 1.3.14 (logging off). The sbt project is outside the repository and is not committed.
- Purpose: the oracle that ADR 0033 D13 asks for before the scope of #1184 (TYP-50, the innermost matching strategy). It completes the [Resume oracle](typed-resume-2026-11-06.md) (R5a to R5e: nesting with disjoint classes and with overlapping classes) with what that oracle did not run: a limit exhausted inside an inner Restart (N1, N2, N7), a supervised behavior returned by a handler (N3, N4, N9, N11), a stack rebuilt by a restart (N10), the same layout flattened (N5a), the effect on children (N6) and on a failing first `Setup` (N8), each with equal classes and with different classes.
- Method: read the code (section 1), then run each scenario in its own JVM, three runs per scenario; the 16 scenarios (N11 was run later, on 2026-11-06, for the replacement by a returned `Restart`) gave the same events in the same order in the three runs (compared after removing the time prefix of each line; no actor path hash appears in these logs). Harness and failure classes as in the Resume oracle: a Typed guardian `P` spawns and watches the supervised actor `A`; `E1` extends `RuntimeException`, `E2` extends `E1`, `E3` extends `RuntimeException`; the state is a counter in a `var` created inside `Setup`. `switcher(next)` is a behavior whose handler answers the message `Switch` with `next` (a freshly supervised behavior), as `SupervisionSpec.scala:1270` does.

## 1. What the source says (code reading, Pekko 1.7.1)

`I` = `actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/InterceptorImpl.scala`; `S` = `actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala`; `B` = `actor-typed/src/main/scala/org/apache/pekko/actor/typed/Behavior.scala`; `SS` = `actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala` (the first line of each is `/*`).

| # | Claim | Location |
|---|---|---|
| C1 | Each `onFailure` wraps the behavior in another `Supervisor` | `B:76-86` |
| C2 | A supervisor catches a throwable only if its class is assignable from the unwrapped exception's class | `S:75-76` |
| C3 | Two supervisors are the same, and the enclosing one disappears, when their throwable classes are equal (the strategy is not compared) | `S:78-83` |
| C4 | `preStart` first runs `aroundStart` (the start of the whole stack, so a failing `Setup` meets every supervisor in turn) and only then `deduplicate` | `I:82-85` |
| C5 | `deduplicate` runs after the start, after every message and after every signal: it starts the returned behavior and, **if an interceptor equal to this one already exists inside the started stack, returns the started stack without wrapping it in this interceptor** (the enclosing one disappears); otherwise it wraps the started stack in this interceptor (the test is `existsInStack`) | `I:90-98`, `I:101-103`, `I:106-122`, `B:218-232` |
| C6 | A `Restart` supervisor that has used its limit rethrows (`throw t`); the enclosing supervisor then sees the exception | `S:335-337` |
| C7 | At the **first** start a Restart supervisor rethrows without retrying when its restart is unlimited or `restartCount + 1 >= maxRestarts` (`S:300-303`); a failure while a **restart** is being completed is handled by the rule of C6 instead (`restartCount >= maxRestarts` rethrows, `S:335-337`), so an outer `withLimit(3)` gives one initial run and three retries | `S:294-306`, `S:335-337` |
| C8 | The specs: nesting `SS:201-206` (`supervise(supervise(..))`) and flattening `SS:208-212` (`.onFailure.onFailure`) with Resume and Restart; nesting `SS:448-453`, `SS:462-468`, `SS:477-483` and flattening `SS:455-460`, `SS:470-475`, `SS:485-490` with different strategies and with backoff; `SS:849-880` titled "nesting" and written flattened; the replacement by a returned supervise `SS:1270-1307` | as listed |

## 2. Scenarios and results

**How to read the tables.** "Observed" cites the lines of the raw log by their sequence number `[nnn]` in `nn/<scenario>.1.txt` (runs 2 and 3 have the same lines); a sentence in that column states only what those lines show. "Inferred" is what the lines suggest through the code of section 1 and no line shows. `Get` prints the counter `n` and the number of `Setup` runs; "restart" is `A.signal PreRestart` followed by a new `A.setup`; "stop" is `A.signal PostStop` followed by `P.signal ChildFailed(E)`.

### N1, N7, N7d, N2: an inner Restart whose limit is used

| Run | Layout (outer ← inner) | Observed | Inferred |
|---|---|---|---|
| N1 | `resume[E1]` ← `restart[E1].withLimit(1, 10 s)` (equal classes) | `Inc`; `E1` [003]: `PreRestart` [004], `setup#2` [005], `n=0` [006]; `Inc`, `n=1` [007]; `E1` [008]: `PostStop` [009], `ChildFailed(E1)` [010]; `setups=2` [011]. The second `E1` was not resumed | the outer Resume had been deduplicated once the start succeeded, so the rethrow found no enclosing supervisor |
| N7 | `restart[E1]` ← `restart[E1].withLimit(1, 10 s)` (equal classes) | the same lines as N1 [001]-[011]: one restart, then `PostStop` and `ChildFailed(E1)`; `setups=2`. The second `E1` was not restarted | as N1, for an outer Restart |
| N7d | `restart[Exception]` ← `restart[E1].withLimit(1, 10 s)` (different classes) | `E1` [003]: `PreRestart` [004], `setup#2` [005]; `Inc`, `n=1` [007]; `E1` [008]: `PreRestart` [009], `setup#3` [010], `n=0` [011]; `E1` [012]: `PreRestart` [013], `setup#4` [014]; no `PostStop` until the end [017] | the second restart was done by the outer Restart, which received the rethrow of the exhausted inner limit |
| N2 | `resume[E1]` ← `restart[E2].withLimit(1, 10 s)` (different classes, inner narrower) | `E2` [003]: `PreRestart` [004], `setup#2` [005], `n=0` [006]; `Inc`, `n=1` [007]; `E2` [008]: no restart, `n=1` [009]; `E1` [010]: `n=1` [011]; `setups=2` [012] | the second `E2` reached the outer Resume as the rethrow of the exhausted inner limit |

### N3, N4, N9, N11: a supervised behavior returned by a handler

`switcher(next)` answers `Switch` with `next`. `S.` lines belong to the switcher, `A.` lines to the returned behavior.

| Run | Layout (outer ← returned) | Observed | Inferred |
|---|---|---|---|
| N3 | `restart[E1]` ← `switcher(resume[E1] ← target)` | before the switch `E1` [003] restarts the switcher (`S.setup#2` [005], `n=0` [006]); after `Switch` [007] and `A.setup#1` [008]: `Inc`; `E1` [010]: no `PreRestart`, `n=1` [011], `setups=1` [012] | the returned Resume replaced the enclosing Restart |
| N4 | `restart[E1]` ← `switcher(resume[E2] ← target)` | after `Switch` [002]: `E2` [005]: `n=1` [006]; `E1` [007]: `A.signal PreRestart` [008], `S.setup#2` [009], `n=0` [010] | both supervisors were kept; the restart rebuilt from the outer's initial behavior (the switcher) |
| N9 | the returned stack holds an equal pair: `switcher(resume[E1] ← restart[E1].withLimit(1, 10 s) ← target)` | after `Switch` [002]: `E1` [005]: `PreRestart` [006], `setup#2` [007], `n=0` [008]; `Inc`, `n=1` [009]; `E1` [010]: `PostStop` [011], `ChildFailed(E1)` [012]. The second `E1` was not resumed | the pair was deduplicated when the returned stack started (the turn site of C5) |
| N11 | `restart[E1]` ← `switcher(restart[E1].withLimit(1, 10 s) ← target)`: the returned supervise is a limited Restart of the same matcher | after `Switch` [002] and `A.setup#1` [003]: `Inc`; `E1` [005]: `PreRestart` [006], `setup#2` [007], `n=0` [008]; `Inc`, `n=1` [009]; `E1` [010]: `PostStop` [011], `ChildFailed(E1)` [012]; `setups=2` [013]. The second `E1` was not restarted by the enclosing Restart | the returned Restart replaced the enclosing Restart of the same matcher: otherwise the exhausted limit would have been rethrown to the enclosing Restart, which would have restarted the actor, as in N7d |

### N10: a stack rebuilt by a restart

`restart[Exception]` ← a `Setup` that returns `resume[E1] ← restart[E1].withLimit(1, 10 s) ← target`; script: `Inc, Get; E3; Inc, Get; E1; Inc, Get; E1; Get`.

| Run | Observed | Inferred |
|---|---|---|
| N10 | `E3` [003]: `PreRestart` [004], `setup#2` [005], `n=0` [006]; `Inc`, `n=1` [007]; `E1` [008]: `PreRestart` [009], `setup#3` [010], `n=0` [011]; `Inc`, `n=1` [012]; `E1` [013]: `PreRestart` [014], `setup#4` [015], `n=0` [016]. The second `E1` after the rebuild restarted the actor (`setup#4`) and did not leave the state at `n=1` | after the rebuild the Resume of the pair is gone (the restart site of C5): the exhausted inner limit reached the outer `restart[Exception]`. Without the deduplication the Resume would have kept `n=1` |

### N5a. Flattened

`supervise(target).onFailure[E2](resume).onFailure[E3](restart)` with the script of R5a of the Resume oracle. Observed: `nn/N5a.1.txt` and `rr/R5a.1.txt` of the harness directory are identical line for line (diff empty, 14 lines): `E2` keeps `n=1`, `E3` restarts (`setup#2`, `n=0`), `E2` keeps `n=1`, `E1` stops with `ChildFailed(E1)`.

### N6. Children and the rethrow

`A` has a child `k1`; script: `Kid, Get; E1; Get`.

| Run | Layout (outer ← inner) | Observed | Inferred |
|---|---|---|---|
| N6a | `restart.withStopChildren(false)[E1]` ← `restart[E1]` | `E1` [003]: `PreRestart` [004], `kid[k1].PostStop` [005], `setup#2 children=List()` [006] | the inner layer decided |
| N6b | `restart[E1]` ← `restart.withStopChildren(false)[E1]` | `E1` [003]: `PreRestart` [004], `setup#2 children=List(k1)` [005] | the inner layer decided |
| N6c | `restart[E1]` ← `restart.withStopChildren(false).withLimit(0, 1 s)[E1]` (equal classes) | `E1` [003]: `kid[k1].PostStop` [004], `PostStop` [005], `ChildFailed(E1)` [006]; `setups=1` [007]; no `PreRestart` | the inner layer rethrew and the outer Restart had been deduplicated |
| N6d | `restart[Exception]` ← `restart.withStopChildren(false).withLimit(0, 1 s)[E1]` (different classes) | `E1` [003]: `PreRestart` [004], `kid[k1].PostStop` [005], `setup#2 children=List()` [006] | the outer Restart took the rethrow and stopped the child |

### N8. A failing first `Setup` in nested layers

`target(failSetup)` throws `E1` on every `Setup`; the inner layer is an unlimited `restart`, the outer `restart.withLimit(3, 10 s)`.

| Run | Classes | Observed | Inferred |
|---|---|---|---|
| N8 | inner `E1`, outer `E1` (equal) | `A.setup#1` to `#4` [001]-[004], then `ChildFailed(ActorInitializationException ...)` [005]; `setups=4` [006] | the outer limit counted the retries; both layers were consulted during the start |
| N8d | inner `E1`, outer `Exception` (different) | the same lines | as N8 |

## 3. Summary

| Scenario | One line |
|---|---|
| N1, N7 | With equal classes, after one restart an inner Restart whose limit is used stops the actor; the outer Resume or Restart does not take it (inferred: it had been deduplicated) |
| N7d, N2 | With different classes the same exhausted limit ends in a restart (with an outer Restart) or in the state kept (with an outer Resume); *inferred:* the outer layer received the rethrow |
| N3, N4 | After a handler returns a supervised behavior: an equal-class Resume handles `E1` and no restart happens (inferred: it replaced the enclosing Restart); a different-class Resume handles `E2`, and `E1` restarts the switcher (inferred: both were kept) |
| N9 | A returned stack that holds an equal pair behaves as N1: the second `E1` stops the actor |
| N11 | A returned limited Restart of the same matcher as the enclosing Restart: its exhausted limit stops the actor (inferred: it replaced the enclosing Restart) |
| N10 | After a restart rebuilds a stack that holds an equal pair, the second `E1` restarts the actor (`setup#4`) instead of leaving the state kept; *inferred:* the pair had been deduplicated, as in N1 |
| N5a | Flattened and nested layouts give identical logs (R5a) |
| N6a-N6d | The child is stopped (N6a) or kept (N6b) after the restart; with equal classes and an inner limit 0 the actor stops (N6c); with different classes it restarts and the child is stopped (N6d); *inferred:* the inner layer decided (N6a, N6b) and the inner layer rethrew (N6c, N6d) |
| N8, N8d | A failing first `Setup` runs 4 times under an outer `withLimit(3)`, with equal or different classes; *inferred:* the outer limit counted the retries (C7) |

## 4. Limits of this evidence

- The deduplication is **inferred** from its effects (N1, N3, N6c, N7, N9, N10); no log line shows the stack of supervisors. That it is not applied while the start is failing is **inferred** from N8 and from `I:82-85`.
- The raw logs are `nn/<scenario>.{1,2,3}.txt` and `rr/R5a.1.txt` of the harness directory, outside the repository (see the header).
- C6 and C7 (what an enclosing supervisor receives when a Restart rethrows) are read from the code; no log line shows the rethrow.
- `SupervisorStrategy.stop` (explicit) was not run: it is a separate story (#1182).
- No scenario with `restartWithBackoff`.
