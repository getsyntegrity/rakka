Verdict path: fast-kill: case a11-stop-then-late-lette confirmed by a single-case replay (`rust-anything app replay --case`); the unmutated build matches Pekko on it; no full replay ran

# App replay

Build: `$TMPDIR/pekko-mutants-21018-0jf977yq/target-mailbox-dead-user-messages-not-published/release/examples/classic-parity`

Passed: 0/1

Fallback rate: not reported by this build

## Failing cases

### a11-stop-then-late-lette

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"subscribe","ok":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/c","name":"c","parent":"pekko://sys/user/p","hasUid":true},{"op":"do","ok":true},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/c","is":"c","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/c"}]},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"dead-letter","msg":"late","sender":"pekko://sys/deadLetters","recipient":"pekko://sys/user/p/c"}]},{"op":"events-now","ok":true,"events":[{"ev":"dead-letter","msg":"late","sender":"pekko://sys/deadLetters","recipient":"pekko://sys/user/p/c"}]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"subscribe","ok":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/c","name":"c","parent":"pekko://sys/user/p","hasUid":true},{"op":"do","ok":true},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/c","is":"c","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/c"}]},{"op":"tell","ok":true},{"op":"await-events","ok":false,"events":[]},{"op":"events-now","ok":true,"events":[]}]}
```
