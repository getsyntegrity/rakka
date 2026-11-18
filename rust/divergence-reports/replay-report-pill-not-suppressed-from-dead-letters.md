# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-i2z9eagu/target-pill-not-suppressed-from-dead-letters/release/examples/classic-parity`

Passed: 565/566

Fallback rate: not reported by this build

## Failing cases

### dl-pill-to-dead-actor-su

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"subscribe","ok":true},{"op":"tell","ok":true},{"op":"await-stopped","ok":true,"terminated":true},{"op":"tell","ok":true},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"dead-letter","msg":"after","sender":"pekko://sys/deadLetters","recipient":"pekko://sys/user/a"}]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"subscribe","ok":true},{"op":"tell","ok":true},{"op":"await-stopped","ok":true,"terminated":true},{"op":"tell","ok":true},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"dead-letter","msg":{"class":"?"},"sender":"pekko://sys/deadLetters","recipient":"pekko://sys/user/a"}]}]}
```
