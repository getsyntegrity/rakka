Verdict path: fast-kill: case dl-pill-to-dead-actor-su confirmed by a single-case replay (`rust-anything app replay --case`); the unmutated build matches Pekko on it; no full replay ran

# App replay

Build: `$TMPDIR/pekko-mutants-79342-7ewgt65e/target-pill-not-suppressed-from-dead-letters/release/examples/classic-parity`

Passed: 0/1

Fallback rate: not reported by this build

## Failing cases

### dl-pill-to-dead-actor-su

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"subscribe","ok":true},{"op":"tell","ok":true},{"op":"await-stopped","ok":true,"terminated":true},{"op":"tell","ok":true},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"dead-letter","msg":"after","sender":"pekko://sys/deadLetters","recipient":"pekko://sys/user/a"}]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"subscribe","ok":true},{"op":"tell","ok":true},{"op":"await-stopped","ok":true,"terminated":true},{"op":"tell","ok":true},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"dead-letter","msg":{"class":"?"},"sender":"pekko://sys/deadLetters","recipient":"pekko://sys/user/a"}]}]}
```
