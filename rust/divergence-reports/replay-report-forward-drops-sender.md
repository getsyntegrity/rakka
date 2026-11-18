# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-i2z9eagu/target-forward-drops-sender/release/examples/classic-parity`

Passed: 563/566

Fallback rate: not reported by this build

## Failing cases

### dl-forward-to-stopped

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/f","name":"f","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/b","name":"b","parent":"pekko://sys/user","hasUid":true},{"op":"subscribe","ok":true},{"op":"stop","ok":true},{"op":"await-stopped","ok":true,"terminated":true},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"dead-letter","msg":"m","sender":"pekko://sys/user/b","recipient":"pekko://sys/user/a"}]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/f","name":"f","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/b","name":"b","parent":"pekko://sys/user","hasUid":true},{"op":"subscribe","ok":true},{"op":"stop","ok":true},{"op":"await-stopped","ok":true,"terminated":true},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"dead-letter","msg":"m","sender":"pekko://sys/deadLetters","recipient":"pekko://sys/user/a"}]}]}
```

### snd-forward-chain

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/g","name":"g","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/f","name":"f","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/b","name":"b","parent":"pekko://sys/user","hasUid":true},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"received","msg":"m","sender":"pekko://sys/user/b"}]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/g","name":"g","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/f","name":"f","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/b","name":"b","parent":"pekko://sys/user","hasUid":true},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"received","msg":"m","sender":"pekko://sys/deadLetters"}]}]}
```

### snd-forward-keeps-sender

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/f","name":"f","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/b","name":"b","parent":"pekko://sys/user","hasUid":true},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"received","msg":"m","sender":"pekko://sys/user/b"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"received","msg":"m","sender":"pekko://sys/user/b"}]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/f","name":"f","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/b","name":"b","parent":"pekko://sys/user","hasUid":true},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"received","msg":"m","sender":"pekko://sys/deadLetters"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"received","msg":"m","sender":"pekko://sys/user/b"}]}]}
```
