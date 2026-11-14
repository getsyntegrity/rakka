# Pekko reproduction for ADR 0030

`BoundedTerminatedRepro.java` runs the scenarios D-1a, D-1b, D-5, D-6 (twice: with and without the re-watch), D-7, D-8, D-9 (bounded and unbounded), D-10 (three orderings) and D-11 (a failed PreStart) against the real Apache Pekko classes (v1.7.1, `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`) and prints what the actor received and which dead letters were published. It uses the mailbox id `bounded-capacity:2`, the one Typed's `MailboxSelector.bounded(2)` builds.

It is not part of any build and is not run by CI. To run it you need JDK 11+ to run, the compiled `actor` module of the pin, `scala-library` 2.13 and `config` 1.4 on the classpath, the module's `src/main/resources` (for `reference.conf`), and a `version.conf` with `pekko.version` equal to the version string of the build (a build without `.git` reports `HEAD+<date>-<time>`; a release build reports `1.7.1`).

To compile `actor` from the pin in a throwaway copy (JDK 8 is required by the Pekko build; the dependencies must be in the local cache for `-Dsbt.offline=true`):

```sh
cp -R upstream/pekko "$TMP/pekko" && rm "$TMP/pekko/.git"
cd "$TMP/pekko"
JAVA_8_HOME=/path/to/jdk8 sbt -Dsbt.offline=true actor/compile
```

Then run the reproducer:

```sh
javac -cp "$CLASSES:$SCALA:$CONFIG" -d out BoundedTerminatedRepro.java
java  -cp "out:$CLASSES:$SCALA:$CONFIG" BoundedTerminatedRepro
```

Recorded output and its provenance caveat are in [ADR 0030](../0030-bounded-mailbox.md#reproduction-against-the-pin). The goPekko tests that mirror D-1a and D-1b are `bounded_mailbox_unwatch_test.go` in `pekko-actor/actor`.
