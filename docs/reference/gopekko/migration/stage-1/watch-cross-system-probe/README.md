# H5b: cross-System watch probe against Pekko v1.7.1 (written, NOT compiled, NOT executed)

**Status: not executed and not compiled.** `H5bCrossWatch.scala` was written for the cross-System design (GOV-15 option B, issue [#1065](https://github.com/getsyntegrity/gopekko/issues/1065), design in [`docs/parity/cross-system-watch-design.md`](../../../docs/parity/cross-system-watch-design.md)). It runs through `sbt`, which `migration/settings.go.json` denies inside migration loops; nothing in this directory was run, so **no result exists and the oracle is not available**. It is evidence tooling in the style of [`../supervision/h4-repro/`](../supervision/h4-repro/INDEX.md), not part of the judge and not a parity claim. Who runs it is not decided here.

## What it covers (the cases `H5Probe.scala` does not)

Two `ActorSystem`s in one JVM, default config, no remoting: `A` holds the watcher, `B` holds the subject. It prints events; it asserts nothing. A negative observation ("nothing arrived") holds only within a mailbox round trip plus a 300 ms window; it is not a proof. `system.deadLetters` is never observed.

| Case | Operation | Hypothesis read from the Pekko source (not a result) |
|---|---|---|
| a | `watch`, live subject of B, then stopped | one `Terminated`, `existenceConfirmed=true` (already seen by `H5Probe`, `h5-sbt.out`) |
| b | `watch`, subject of B already terminated | one `Terminated`, `existenceConfirmed=false` (`ActorRef.scala:694-698, 745-751`) |
| c1 / c2 | `watchWith`, live-then-stopped / already terminated | one `Custom(name)` message and no `Terminated` (`DeathWatch.scala:99-119`) |
| d1 | `unwatch` after the notification was delivered | no further event, no failure (`ActorRef.scala:699`) |
| d2 | `unwatch` of a terminated, never-watched subject | no event, no failure (`ActorRef.scala:699`) |
| e | watcher stopped first, then the subject | no delivery, no failure; the release of the link inside B is not visible through the public API |

## Requirements and command

- `sbt` and a JDK 8 (`JAVA_8_HOME`, default `$HOME/.sdkman/candidates/java/8.0.504+1-zulu`); network or a populated sbt/Coursier cache (not verified); the build compiles Pekko `actor` and `testkit` from source (no prebuilt jar exists).
- An environment where the repository rules permit `sbt`. This was not satisfied when the file was written.

```bash
T=$(mktemp -d)
git -C upstream/pekko archive --format=tar --prefix=pekko-copy/ v1.7.1 | tar -x -C "$T"
git -C "$T/pekko-copy" init -q && git -C "$T/pekko-copy" add -A
git -C "$T/pekko-copy" -c user.name=probe -c user.email=probe@example.invalid commit -q -m "import v1.7.1"
git -C "$T/pekko-copy" tag v1.7.1
migration/stage-1/watch-cross-system-probe/run.sh "$T/pekko-copy" "$T/h5b-sbt.out"
```

The steps up to the tag were run in a throwaway directory on 2026-10-25 (archive 49 MB, tag `v1.7.1` resolved); the `run.sh` line was not. Never run it against `upstream/pekko` itself. To record a result, commit the raw `h5b-sbt.out` here and add an index row stating date, environment and that it is not a parity claim, as `h4-repro/INDEX.md` does.

## Pending until it is run

Compilation of `H5bCrossWatch.scala` (a Scala syntax or API error is possible); the observed output for cases a to e; whether `existenceConfirmed=false` appears in case b; whether the judge could host these scenarios (it cannot as-is: one `ActorSystem` per run and no `existenceConfirmed` in its event vocabulary).

## Native review (2026-10-25)

Candidate `sha256:99c8a63a3b2be594af5faff389a881661eeefae94421260485e802188825bcb4` (branch `docs/cross-system-watch-5-probe`, head `c777836ea07b921cab34b9fc2ca1dd3a696a7958`), lineage `review-696ad3e4cce1692d`, lenses `review-risk` and `review-reliability`. Result: approved and acknowledged (authority burned).

- Risk lens: no findings.
- Reliability lens: two informational WARNINGs, non-blocking, no correction. R3-001, `H5bCrossWatch.scala:69-73`: `roundTrip` discards the boolean result of the latch await, so a timeout can look like "no events". R3-002, `H5bCrossWatch.scala:55-61`: `newWatcher` ignores the ready await and can return null.

Precondition: before the probe is used as an oracle its timeouts must be distinguished from an absence of events (R3-001 and R3-002 addressed in a later change, then compiled and run). It remains NOT compiled and NOT executed; nothing is an oracle yet.
