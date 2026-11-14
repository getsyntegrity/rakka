# Differential run: ActorPath, address parsing and stale-uid resolution (identity units)

- Date: 2026-10-29. Evidence class: **Executed** (225 rows produced by running Pekko code: 196 in revisions 1 and 2, 29 more in revision 3, see the last section; nothing is read from source here).
- Purpose: a differential oracle for the identity units #852, #812, #1089, #853, #894 and #1092. It records what Pekko v1.7.1 `ActorPath`, `ActorPath.fromString`, `AddressFromURIString`, `ActorPathExtractor`, `RelativeActorPath`, `ActorPath.isValidPathElement` and `provider.resolveActorRef` return for a fixed set of inputs.
- Pekko: v1.7.1, `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`. The submodule HEAD equals the pin: `git -C upstream/pekko rev-parse HEAD` printed `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`, and `git ls-tree origin/develop upstream/pekko` lists `160000 commit 9a54d1294c3ac7821bbed3c940769c3d5027d1ff`.
- `upstream/pekko` was never modified: `git -C upstream/pekko status --short` was empty, and nothing was run inside it. The raw probe (`Probe.scala`) is an artifact of a disposable copy; it is stored here only as evidence and is not built in this repository.
- Revision 2: element sequences are printed with size and quoted elements because `toString` cannot distinguish `List("")` from `Nil` (both print `List()`). See "Changes from revision 1".
- Output: `observed.txt` is the 225 `ROW|...` lines of the run (196 of revision 2 plus 29 of revision 3), verbatim, with the `ROW|` prefix removed (`<label>|<value>`).

## Environment

- macOS (Darwin 25.3.0), JDK Eclipse Adoptium (Temurin) 11.0.32 (active), sbt 1.12.14 (`project/build.properties`), Scala 2.13 (the `actor` module).
- `JAVA_8_HOME=~/.sdkman/candidates/java/8.0.504+1-zulu` must be set for BOTH sbt commands (steps 3 and 4): without it the Pekko sbt build stops with "A JDK 8 installation was not found" (an independent reproduction hit exactly this). The probe itself ran on the active JDK 11.

## Commands, in order

1. Export the pinned tree into a disposable directory (cwd: the goPekko repository root; `<copy>` is the directory printed by `mktemp -d`):

   ```
   <copy>=$(mktemp -d)
   git -C upstream/pekko archive --format=tar 9a54d1294c3ac7821bbed3c940769c3d5027d1ff | tar -x -C <copy>
   ```

2. Add the probe to the copy (cwd: the goPekko repository root; the file declares `package org.apache.pekko.actor.probe`):

   ```
   mkdir -p <copy>/actor/src/main/scala/probe
   cp migration/stage-1/identity-oracle/pekko-repro/Probe.scala <copy>/actor/src/main/scala/probe/Probe.scala
   ```

   The revision 2 run compiled the stored `Probe.scala` as is (the comment header is part of the file).

3. Compile the actor module (cwd: `<copy>`):

   ```
   JAVA_8_HOME=~/.sdkman/candidates/java/8.0.504+1-zulu sbt -batch -Dsbt.offline=true "actor/compile"
   ```

   Exit 0. 22 s wall; sbt reported `[success] Total time: 16 s` (revision 2 run, fresh copy).

4. Run the probe (cwd: `<copy>`):

   ```
   JAVA_8_HOME=~/.sdkman/candidates/java/8.0.504+1-zulu sbt -batch -Dsbt.offline=true "actor/runMain org.apache.pekko.actor.probe.Probe"
   ```

   Exit 0. sbt reported `[success] Total time: 5 s` (12 s wall). The `ROW|` lines of the output are `observed.txt`.

## Offline mode

`-offline` is not an sbt command (the first attempt used it as one; sbt 1.x still loaded the build, but the flag is invalid). The offline switch is the system property `-Dsbt.offline=true`. Offline resolution succeeded with no download. Cache sizes in KB, before and after:

| Cache | Before | After |
|---|---|---|
| `~/Library/Caches/Coursier` | 192020 | 192020 |
| `~/.sbt` | 190928 | 190876 |
| `~/.ivy2` | 0 | 0 |

Coursier and `.ivy2` did not change, and `.sbt` shrank by 52 KB: nothing was downloaded.

## Provenance of the prebuilt classes

Revision 2 re-verified this with a fresh copy: `diff -rq upstream/pekko/actor/target/scala-2.13/classes <copy>/actor/target/scala-2.13/classes` printed 5 lines. Exactly one is a differing file: `org/apache/pekko/Version$.class` (it embeds the build version string, `HEAD+<timestamp>` in the copy because the archive has no git metadata). The other four are entries present only in the copy: `META-INF`, `reference.conf`, `version.conf` and the probe classes under `org/apache/pekko/actor/probe`. Revision 1 had found 1893 classes in both trees, 1892 byte-identical. So the prebuilt classes were built from the pinned source.

## Results

Transcribed from `observed.txt` (line numbers refer to that file). `uid` values of the stale-uid rows are random per run, so only their relationships are stated.

### (a) ActorPath navigation (lines 1-26)

Let `root = RootActorPath(Address("pekko","sys"))` and `ua = root / "user" / "a"`.

| Expression | Observed |
|---|---|
| `root.parent == root` | `true` |
| `root.parent` | `pekko://sys/` |
| `ua.parent` | `pekko://sys/user` |
| `ua.parent.parent` | `pekko://sys/` |
| `root.descendant(["user","","a"])` | `pekko://sys/user/a`, elements `n=2 ["user","a"]` |
| `root / Seq("user","","a")` | `pekko://sys/user/a`, elements `n=2 ["user","a"]` |
| `root.descendant([""])` | `pekko://sys/`, elements `n=1 [""]` (the root's own elements) |
| `root.descendant([])` | `pekko://sys/`, same instance as `root` (`same=true`), elements `n=1 [""]` |
| `root / ""` | `pekko://sys/`, elements `n=1 [""]` |
| `root.elements` / `ua.elements` | `n=1 [""]` / `n=2 ["user","a"]` |
| `ua.name` and `root.name` | `a` and `/` |
| `ua.toString` / `ua.toStringWithoutAddress` | `pekko://sys/user/a` / `/user/a` |
| `ua.toSerializationFormat` | `pekko://sys/user/a` |
| `root.toString` / `root.toSerializationFormat` | `pekko://sys/` / `pekko://sys/` |
| `ua == root/user/a` (rebuilt) / `ua` vs system `sys2` | `true` / `false` |
| `ua.withUid(5) == ua` | `true` |
| `ua.withUid(5).toString` / `.toSerializationFormat` | `pekko://sys/user/a` / `pekko://sys/user/a#5` |
| `ua.uid` | `0` |
| `ua.compareTo(root)` | `-1` |
| `ua.hashCode == other.hashCode` (equal paths) | `true` |

### (b) ActorPath.fromString, AddressFromURIString, ActorPathExtractor (lines 27-152)

For each input, six rows: `fromString` (toString / serialization / uid / elements / address / isRoot), `Try(fromString).isSuccess`, `AddressFromURIString(s)`, `AddressFromURIString.parse(s)`, `AddressFromURIString.unapply(s)`, `ActorPathExtractor.unapply(s)`. Shown: `fromString` result, then the extractor. Sequences are `n=<size> ["e1","e2"]`.

| Input | `fromString` | `ActorPathExtractor.unapply` | Address parse |
|---|---|---|---|
| `pekko://sys/user/a` | `pekko://sys/user/a`, uid 0, `n=2 ["user","a"]` | `Some((pekko://sys,n=2 ["user","a"]))` | `pekko://sys` |
| `pekko://sys/` | root, `pekko://sys/`, `n=1 [""]` | `Some((pekko://sys,n=1 [""]))` | `pekko://sys` |
| `pekko://sys` | root, `pekko://sys/`, `n=1 [""]` | `Some((pekko://sys,n=0 []))` | `pekko://sys` |
| `pekko://sys//a` | `pekko://sys/a`, `n=1 ["a"]` | `Some((pekko://sys,n=2 ["","a"]))` | `pekko://sys` |
| `pekko://sys/a/` | `pekko://sys/a`, `n=1 ["a"]` | `Some((pekko://sys,n=2 ["a",""]))` | `pekko://sys` |
| `pekko://sys/a%41` | `pekko://sys/a%41`, `n=1 ["a%41"]` | `Some((pekko://sys,n=1 ["a%41"]))` | `pekko://sys` |
| `pekko://sys/a%4` | throws `java.net.MalformedURLException: cannot parse as ActorPath: pekko://sys/a%4` | `None` | throws `MalformedURLException: pekko://sys/a%4`; `unapply` `None` |
| `pekko://sys/a#5` | `pekko://sys/a`, serialization `pekko://sys/a#5`, uid 5 (elements `n=1 ["a"]`) | `Some((pekko://sys,n=1 ["a#5"]))` | `pekko://sys` |
| `pekko://sys/user/a?q` | `pekko://sys/user/a`, `n=2 ["user","a"]` | `Some((pekko://sys,n=2 ["user","a"]))` | `pekko://sys` |
| `pekko://my_sys/user/a` | throws `MalformedURLException: cannot parse as ActorPath: pekko://my_sys/user/a` | `None` | throws `MalformedURLException` |
| `pekko://s y/a` | throws `MalformedURLException` | `None` | throws `MalformedURLException` |
| `` (empty) | throws `MalformedURLException: cannot parse as ActorPath: ` | `None` | throws `MalformedURLException: ` |
| `://x` | throws `MalformedURLException` | `None` | throws `MalformedURLException` |
| `pekko://sys@h:1/a` | `pekko://sys@h:1/a`, addr `pekko://sys@h:1`, `n=1 ["a"]` | `Some((pekko://sys@h:1,n=1 ["a"]))` | `pekko://sys@h:1` |
| `/user/a` | throws `MalformedURLException` | `None` | throws `MalformedURLException` |
| `pekko://sys/a#0` | `pekko://sys/a`, serialization `pekko://sys/a`, uid 0 | `Some((pekko://sys,n=1 ["a#0"]))` | `pekko://sys` |
| `pekko://sys/a#x` | throws `java.lang.NumberFormatException: For input string: "x"` | `Some((pekko://sys,n=1 ["a#x"]))` | `pekko://sys` |
| `pekko://sys/a#-1` | `pekko://sys/a`, serialization `pekko://sys/a#-1`, uid -1 | `Some((pekko://sys,n=1 ["a#-1"]))` | `pekko://sys` |
| `pekko://sys/a#` | throws `java.lang.NumberFormatException: For input string: ""` | `Some((pekko://sys,n=1 ["a#"]))` | `pekko://sys` |
| `pekko://sys/a#5#6` | throws `MalformedURLException: cannot parse as ActorPath: pekko://sys/a#5#6` | `None` | throws `MalformedURLException` |
| `pekko://sys@h:1/user/a#7` | `pekko://sys@h:1/user/a`, serialization `...#7`, uid 7 | `Some((pekko://sys@h:1,n=2 ["user","a#7"]))` | `pekko://sys@h:1` |

`Try(fromString).isSuccess` is `false` exactly for the inputs where `fromString` throws. The extractor keeps the `#uid` suffix in the last element and does not parse it; `fromString` parses it.

### (c) RelativeActorPath.unapply (lines 153-164)

| Input | Observed |
|---|---|
| `foo/bar` | `Some(n=2 ["foo","bar"])` |
| `` (empty) | `Some(n=1 [""])` |
| `foo//bar` | `Some(n=3 ["foo","","bar"])` |
| `../sibling` | `Some(n=2 ["..","sibling"])` |
| `foo?x=1` | `Some(n=1 ["foo"])` |
| `a:b/c` | `None` |
| `foo/%4` | `None` |
| `foo/bar#1234` | `Some(n=2 ["foo","bar#1234"])` |
| `foo/bar/` | `Some(n=3 ["foo","bar",""])` |
| `/foo` | `Some(n=2 ["","foo"])` |
| `..` | `Some(n=1 [".."])` |
| `a/./b` | `Some(n=3 ["a",".","b"])` |

### (d) ActorPath.isValidPathElement (lines 165-182)

| Input | Observed | Input | Observed |
|---|---|---|---|
| `a` | `true` | `-` | `true` |
| `a b` | `false` | `_a` | `true` |
| `$a` | `false` | `a#1` | `false` |
| `a%41` | `true` | `a?` | `false` |
| `a%4` | `false` | `a/b` | `false` |
| `` (empty) | `false` | `ä` | `false` |
| `$$` | `false` | `a@b` | `true` |
| `.` | `true` | `..` | `true` |
| `a*` | `true` | `a:b` | `true` |

### (e) Stale-uid resolution (lines 183-196)

Setup: an actor `a` is started, its serialization format `old` is saved, it is stopped (500 ms wait), and a new actor `a` is started under the same name (`new`). `provider.resolveActorRef` is called on each string.

| Input | Observed |
|---|---|
| old and new serialization | `pekko://sys/user/a#<uidOld>` and `pekko://sys/user/a#<uidNew>`; `stale.uids differ` is `true` |
| `old` (with `#uidOld`) | class `EmptyLocalActorRef`, equals the old ref (`eqOld=true`), not the new (`eqNew=false`), path `...#<uidOld>`, `isTerminated=true` |
| `pekko://sys/user/a` (no uid) | class `RepointableActorRef`, equals the new ref, path `...#<uidNew>`, `isTerminated=false` |
| `pekko://sys/user/a#0` | same as the no-uid row: the new ref |
| `pekko://sys/user/a#x` | throws `java.lang.NumberFormatException: For input string: "x"` |
| current uid (`#<uidNew>`) | `RepointableActorRef`, the new ref, `isTerminated=false` |
| `pekko://sys/user/zzz` | `EmptyLocalActorRef`, path `pekko://sys/user/zzz`, `isTerminated=true` |
| `pekko://other/user/a` | `DeadLetterActorRef`, path `pekko://sys/deadLetters`, `isTerminated=true` |
| `pekko://sys@h:1/user/a` | `DeadLetterActorRef`, path `pekko://sys/deadLetters` |
| `foo` (relative) | `DeadLetterActorRef`, path `pekko://sys/deadLetters` |
| `` (empty) | `DeadLetterActorRef`, path `pekko://sys/deadLetters` |
| `sys.deadLetters.path.toSerializationFormat` | `pekko://sys/deadLetters` |

## Changes from revision 1

Revision 1 printed sequences with `toString`, where `List("")` and `Nil` are both `List()`. Revision 2 prints `n=<size> ["e1","e2"]` through the probe helpers `el` and `fmt`. A diff of the two `observed.txt` files shows only: (1) the new element formatting, (2) the random stale uids, (3) the renamed label `AddressFromURIString.apply('...')` (it was `ActorPath.isValidPathElement-ish AddressFromURIString('...')`, which named the wrong function). Every other row is identical. Rows whose MEANING changed (the old text hid an element):

| Row | Revision 1 | Revision 2 |
|---|---|---|
| `root.elements` | `List()` | `n=1 [""]` |
| `root.descendant([""])`, `root.descendant([])`, `root / ""` (elements) | `List()` | `n=1 [""]` |
| `ActorPath.fromString('pekko://sys/')` and `('pekko://sys')` (elements) | `List()` | `n=1 [""]` |
| `ActorPathExtractor.unapply('pekko://sys/')` | `Some((pekko://sys,List()))` | `Some((pekko://sys,n=1 [""]))` |
| `ActorPathExtractor.unapply('pekko://sys')` | `Some((pekko://sys,List()))` | `Some((pekko://sys,n=0 []))` (differs from the `pekko://sys/` row) |
| `RelativeActorPath.unapply('')` | `Some(List())` | `Some(n=1 [""])` |
| `ActorPathExtractor.unapply('pekko://sys//a')`, `('pekko://sys/a/')` | `List(, a)`, `List(a, )` | `n=2 ["","a"]`, `n=2 ["a",""]` (same elements, now unambiguous) |
| `RelativeActorPath.unapply('foo//bar')`, `('foo/bar/')`, `('/foo')` | `List(foo, , bar)` etc. | `n=3 ["foo","","bar"]`, `n=3 ["foo","bar",""]`, `n=2 ["","foo"]` (same elements, now unambiguous) |

## Limits

- Only the `actor` module was compiled; no other Pekko module was built or run.
- One run only: no repetition, so no flakiness assessment. The two stale uids are random per run; only their relationships (`differ`, `eqOld`, `eqNew`) are evidence.
- The sbt output contains many `Invalid Pekko version: HEAD+<timestamp> for OSGi import package versioning` warnings and one `[error] bnd: Invalid value for Bundle-Version, HEAD+... does not match ...` line, because the exported archive has no git metadata. They are harmless: both sbt commands exited 0.
- The probe file sits at `actor/src/main/scala/probe/Probe.scala` and declares `package org.apache.pekko.actor.probe`; an earlier note mentioned `.../org/apache/pekko/actor/probe/Probe.scala` as the directory, which is not where the file was.
- `observed.txt` is the probe output with trailing whitespace removed: three rows (empty inputs) ended in a space after the colon of Pekko's message, which `git diff --check` rejects. No other byte differs from the recorded `rows.txt`.
- Revision 2 state: the three files totalled 497 lines (RUN.md 200), of which 196 were recorded run output and 101 the verbatim probe source. After revision 3 they are RUN.md 208 lines, `observed.txt` 225 and `Probe.scala` 144 (the authored part is `RUN.md`).
- The judge harness (`judge/run-pekko.sh`) was not used: the probe ran through `actor/runMain`.
- This is not a kit execution record (no manifest, RULEBOOK or RUNBOOK of a kit run) and not a G2 gate artifact. No Go parity is claimed: the rows describe Pekko only.

## Revision 3 (2026-11-04): rows 197 to 225

- Owner decisions of 2026-11-04: G2 of #1286 (probe extended to rows 11-13 of the #1089 scope and to `%2F`, `%41`, `#`, `#x`, `foo#`, `?`, a non-ASCII text) and of #1289 (probe extended to a lone `#`, uid above 2^31, `path#-1`, a negative uid, a uid on an intermediate element, and the stop window of K3). Rows 1 to 196 are unchanged (byte-identical); the new rows are appended after them, so every earlier row number still holds.
- Same environment and commands as above; the new `Probe.scala` adds a `Blocker` actor and a second system `sys3`. Exit 0, 225 `ROW|` lines. Observed once; the stop window uses a latch, not a sleep (the `Thread.sleep(500)` after `gate.countDown()` only lets the stop finish before the last row).
- New relative rows (197-210), for the #1089 scope: `foo` -> `["foo"]`; the URL-encoded name -> one element, unchanged; `pekko://sys/user/a`, `foo bar`, `foo/a b`, `%`, `foo/%ZZ` -> `None`; `%2F`, `%41` -> one element, not decoded; `#`, `#x`, `foo#` -> one element holding the `#` text; `?` -> `[""]` (the path is empty, so one empty element; no query value was probed); `ä` -> `["ä"]` (non-ASCII accepted).
- New resolution rows (211-225), for the #1092 scope: `path#` (empty uid) and uids above 2^31-1 (`2147483648`, `4294967296`) **throw** `NumberFormatException`; `#` alone -> `DeadLetterActorRef`; `2147483647`, `-1`, `-5` (non-matching) -> `EmptyLocalActorRef` with that uid in the path; the live uid -> the live actor. A uid on an intermediate element (`/p#<uid>/c`) **throws** `NumberFormatException` (`java.net.URI` takes the fragment to the end of the string and the extractor appends it to the last element, so the uid text parsed is `<uid>/c`); with a uid on both, the result is `DeadLetterActorRef`. Stop window: after `stop` is told but not processed, resolving with the live uid or with no uid returns the **live** actor (`RepointableActorRef`, not terminated); after the stop is processed the old uid gives `EmptyLocalActorRef`, terminated.
- Status of the scope rows: the relative rows 11-13 of #1089 and the extra rows, and D-6 of #1092, now have executed Pekko lines. The Go side of every scenario remains "pending approval (ADR 0033 D13)".
