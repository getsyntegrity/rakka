Verdict path: fast-kill: case a05-restart-resets-state confirmed by a single-case replay (`rust-anything app replay --case`); the unmutated build matches Pekko on it; no full replay ran

# App replay

Build: `$TMPDIR/pekko-mutants-owirezs_/target-failure-does-not-suspend-mailbox/release/examples/classic-parity`

Passed: 0/1

Fallback rate: not reported by this build

## Failing cases

### a05-restart-resets-state

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/c","name":"c","parent":"pekko://sys/user/p","hasUid":true},{"op":"tell","ok":true},{"op":"tell","ok":true},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"received","msg":"m1","sender":"pekko://sys/deadLetters","n":1},{"ev":"received","msg":"brst","sender":"pekko://sys/deadLetters","n":2},{"ev":"pre-restart","reason":"java.lang.IllegalStateException: brst","msg":"brst"},{"ev":"post-stop"},{"ev":"post-restart","reason":"java.lang.IllegalStateException: brst"},{"ev":"pre-start"},{"ev":"received","msg":"m2","sender":"pekko://sys/deadLetters","n":1}]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"received","msg":"m1","sender":"pekko://sys/deadLetters","n":1},{"ev":"received","msg":"brst","sender":"pekko://sys/deadLetters","n":2},{"ev":"pre-restart","reason":"java.lang.IllegalStateException: brst","msg":"brst"},{"ev":"post-stop"},{"ev":"post-restart","reason":"java.lang.IllegalStateException: brst"},{"ev":"pre-start"},{"ev":"received","msg":"m2","sender":"pekko://sys/deadLetters","n":1}]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/c","name":"c","parent":"pekko://sys/user/p","hasUid":true},{"op":"tell","ok":true},{"op":"tell","ok":true},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"received","msg":"m1","sender":"pekko://sys/deadLetters","n":1},{"ev":"received","msg":"brst","sender":"pekko://sys/deadLetters","n":2},{"ev":"received","msg":"m2","sender":"pekko://sys/deadLetters","n":3},{"ev":"pre-restart","reason":"java.lang.IllegalStateException: brst","msg":"brst"},{"ev":"post-stop"},{"ev":"post-restart","reason":"java.lang.IllegalStateException: brst"},{"ev":"pre-start"}]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"received","msg":"m1","sender":"pekko://sys/deadLetters","n":1},{"ev":"received","msg":"brst","sender":"pekko://sys/deadLetters","n":2},{"ev":"received","msg":"m2","sender":"pekko://sys/deadLetters","n":3},{"ev":"pre-restart","reason":"java.lang.IllegalStateException: brst","msg":"brst"},{"ev":"post-stop"},{"ev":"post-restart","reason":"java.lang.IllegalStateException: brst"},{"ev":"pre-start"}]}]}
```
