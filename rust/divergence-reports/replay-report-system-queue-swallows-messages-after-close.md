# App replay

Build: `$TMPDIR/pekko-mutants-famb5jti/target-system-queue-swallows-messages-after-close/release/examples/classic-parity`

Passed: 2795/2824

Fallback rate: not reported by this build

## Failing cases

### i02-prestart-failure-wat

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/k","name":"k","parent":"pekko://sys/user/p","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-stopped","ok":true,"terminated":true},{"op":"do","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/k","is":"k","existenceConfirmed":false,"addressTerminated":false,"sender":"pekko://sys/user/p/k"}]},{"op":"do","ok":true,"children":[]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"}]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/k","name":"k","parent":"pekko://sys/user/p","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-stopped","ok":true,"terminated":true},{"op":"do","ok":true},{"op":"await-events","ok":false,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"children":[]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"}]}]}
```

### i03-watch-old-after-new

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/w","name":"w","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/t","name":"t","parent":"pekko://sys/user/p","hasUid":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/t","name":"t","parent":"pekko://sys/user/p","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":false,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"do","ok":true,"children":[]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":false,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"received","msg":"alive","sender":"pekko://sys/deadLetters"}]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/w","name":"w","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/t","name":"t","parent":"pekko://sys/user/p","hasUid":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/t","name":"t","parent":"pekko://sys/user/p","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true},{"op":"await-events","ok":false,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"do","ok":true,"children":[]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"received","msg":"alive","sender":"pekko://sys/deadLetters"}]}]}
```

### i04-watch-old-and-new

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/w","name":"w","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/t","name":"t","parent":"pekko://sys/user/p","hasUid":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/t","name":"t","parent":"pekko://sys/user/p","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":false,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"do","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":false,"addressTerminated":false,"sender":"pekko://sys/user/p/t"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t2","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t2","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"do","ok":true,"children":[]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":false,"addressTerminated":false,"sender":"pekko://sys/user/p/t"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t2","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/w","name":"w","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/t","name":"t","parent":"pekko://sys/user/p","hasUid":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/t","name":"t","parent":"pekko://sys/user/p","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"await-events","ok":false,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"do","ok":true},{"op":"await-events","ok":false,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t2","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t2","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"do","ok":true,"children":[]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t2","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]}]}
```

### i08-old-ref-watch-existe

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/w","name":"w","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/t","name":"t","parent":"pekko://sys/user/p","hasUid":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/t","name":"t","parent":"pekko://sys/user/p","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":false,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":false,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"compare-refs","ok":true,"pathEqual":true,"refEqual":false,"uidEqual":false,"pathStringEqual":true,"aHasUid":true,"bHasUid":true},{"op":"do","ok":true,"found":true,"path":"pekko://sys/user/p/t","name":"t","parent":"pekko://sys/user/p","hasUid":true,"is":"t2"},{"op":"do","ok":true,"children":["pekko://sys/user/p/t"]},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"received","msg":"alive","sender":"pekko://sys/deadLetters"}]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/w","name":"w","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/t","name":"t","parent":"pekko://sys/user/p","hasUid":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/t","name":"t","parent":"pekko://sys/user/p","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true},{"op":"await-events","ok":false,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"compare-refs","ok":true,"pathEqual":true,"refEqual":false,"uidEqual":false,"pathStringEqual":true,"aHasUid":true,"bHasUid":true},{"op":"do","ok":true,"found":true,"path":"pekko://sys/user/p/t","name":"t","parent":"pekko://sys/user/p","hasUid":true,"is":"t2"},{"op":"do","ok":true,"children":["pekko://sys/user/p/t"]},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"received","msg":"alive","sender":"pekko://sys/deadLetters"}]}]}
```

### i12-new-instance-watches

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/w","name":"w","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/t","name":"t","parent":"pekko://sys/user/p","hasUid":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/t","name":"t","parent":"pekko://sys/user/p","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":false,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":false,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":false,"addressTerminated":false,"sender":"pekko://sys/user/p/t"},{"ev":"received","msg":"alive","sender":"pekko://sys/deadLetters"}]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/w","name":"w","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/t","name":"t","parent":"pekko://sys/user/p","hasUid":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/t","name":"t","parent":"pekko://sys/user/p","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true},{"op":"await-events","ok":false,"events":[{"ev":"pre-start"}]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"}]},{"op":"tell","ok":true},{"op":"await-events","ok":false,"events":[{"ev":"pre-start"},{"ev":"received","msg":"alive","sender":"pekko://sys/deadLetters"}]}]}
```

### i13-new-instance-unwatch

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/w","name":"w","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/t","name":"t","parent":"pekko://sys/user/p","hasUid":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/t","name":"t","parent":"pekko://sys/user/p","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":false,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t2","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":false,"addressTerminated":false,"sender":"pekko://sys/user/p/t"},{"ev":"post-stop"}]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/w","name":"w","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/t","name":"t","parent":"pekko://sys/user/p","hasUid":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/t","name":"t","parent":"pekko://sys/user/p","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true},{"op":"await-events","ok":false,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t2","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"post-stop"}]}]}
```

### i15-stale-terminated-kee

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/w","name":"w","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/t","name":"t","parent":"pekko://sys/user/p","hasUid":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/t","name":"t","parent":"pekko://sys/user/p","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":false,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":false,"addressTerminated":false,"sender":"pekko://sys/user/p/t"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t2","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"await-child-gone","ok":true,"gone":true},{"op":"do","ok":true,"children":[]},{"op":"do","ok":true,"path":"pekko://sys/user/p/t","name":"t","parent":"pekko://sys/user/p","hasUid":true},{"op":"do","ok":true,"found":true,"path":"pekko://sys/user/p/t","name":"t","parent":"pekko://sys/user/p","hasUid":true,"is":"t3"}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/w","name":"w","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/t","name":"t","parent":"pekko://sys/user/p","hasUid":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/t","name":"t","parent":"pekko://sys/user/p","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true},{"op":"await-events","ok":false,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"await-events","ok":false,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t2","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"await-child-gone","ok":true,"gone":true},{"op":"do","ok":true,"children":[]},{"op":"do","ok":true,"path":"pekko://sys/user/p/t","name":"t","parent":"pekko://sys/user/p","hasUid":true},{"op":"do","ok":true,"found":true,"path":"pekko://sys/user/p/t","name":"t","parent":"pekko://sys/user/p","hasUid":true,"is":"t3"}]}
```

### mb-0001

Arguments: `mailbox-case`

status:

```diff
expected exit code 0, got exit code 101
```

stdout:

```diff
- MBX-ST-01 open to scheduled -> first=true second=false open scheduled=true suspendCount=0 raw=2
```

stderr:

```diff
+ 
+ thread 'main' (118327084) panicked at rakka-actor/examples/classic-parity/mailbox_cases.rs:240:5:
+ assertion `left == right` failed
+   left: Ok(())
+  right: Err("Resume")
+ note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### mb-0002

Arguments: `mailbox-case`

status:

```diff
expected exit code 0, got exit code 101
```

stdout:

```diff
- MBX-ST-02 scheduled to idle -> idle=true open scheduled=false suspendCount=0 raw=0 | again=true open scheduled=false suspendCount=0 raw=0 | reschedule=true
```

stderr:

```diff
+ 
+ thread 'main' (118327088) panicked at rakka-actor/examples/classic-parity/mailbox_cases.rs:240:5:
+ assertion `left == right` failed
+   left: Ok(())
+  right: Err("Resume")
+ note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### mb-0003

Arguments: `mailbox-case`

status:

```diff
expected exit code 0, got exit code 101
```

stdout:

```diff
- MBX-ST-03 open to suspended and resumed -> suspend1=true suspend2=false open scheduled=false suspendCount=2 raw=8 | resume1=false open scheduled=false suspendCount=1 raw=4 resume2=true open scheduled=false suspendCount=0 raw=0 | resumeAtZero=true open scheduled=false suspendCount=0 raw=0
```

stderr:

```diff
+ 
+ thread 'main' (118327092) panicked at rakka-actor/examples/classic-parity/mailbox_cases.rs:240:5:
+ assertion `left == right` failed
+   left: Ok(())
+  right: Err("Resume")
+ note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### mb-0004

Arguments: `mailbox-case`

status:

```diff
expected exit code 0, got exit code 101
```

stdout:

```diff
- MBX-ST-04 suspended and scheduled -> schedule=true open scheduled=true suspendCount=1 raw=6 canBe(f,f)=false canBe(f,sys)=true | resume=true open scheduled=true suspendCount=0 raw=2
```

stderr:

```diff
+ 
+ thread 'main' (118327096) panicked at rakka-actor/examples/classic-parity/mailbox_cases.rs:240:5:
+ assertion `left == right` failed
+   left: Ok(())
+  right: Err("Resume")
+ note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### mb-0005

Arguments: `mailbox-case`

status:

```diff
expected exit code 0, got exit code 101
```

stdout:

```diff
- MBX-ST-05 open to closed -> before open scheduled=true suspendCount=2 raw=10 | close1=true closed scheduled=false suspendCount=0 raw=1 close2=false closed scheduled=false suspendCount=0 raw=1
```

stderr:

```diff
+ 
+ thread 'main' (118327100) panicked at rakka-actor/examples/classic-parity/mailbox_cases.rs:240:5:
+ assertion `left == right` failed
+   left: Ok(())
+  right: Err("Resume")
+ note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### mb-0006

Arguments: `mailbox-case`

status:

```diff
expected exit code 0, got exit code 101
```

stdout:

```diff
- MBX-ST-06 closed ignores resume and suspend -> resume=false suspend=false closed scheduled=false suspendCount=0 raw=1
```

stderr:

```diff
+ 
+ thread 'main' (118327104) panicked at rakka-actor/examples/classic-parity/mailbox_cases.rs:240:5:
+ assertion `left == right` failed
+   left: Ok(())
+  right: Err("Resume")
+ note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### mb-0007

Arguments: `mailbox-case`

status:

```diff
expected exit code 0, got exit code 101
```

stdout:

```diff
- MBX-ST-07 closed refuses scheduling -> schedule=false idle=true closed scheduled=false suspendCount=0 raw=1 canBe=false
```

stderr:

```diff
+ 
+ thread 'main' (118327108) panicked at rakka-actor/examples/classic-parity/mailbox_cases.rs:240:5:
+ assertion `left == right` failed
+   left: Ok(())
+  right: Err("Resume")
+ note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### mb-0008

Arguments: `mailbox-case`

status:

```diff
expected exit code 0, got exit code 101
```

stdout:

```diff
- MBX-ST-08 canBeScheduledForExecution matrix -> open: FTTT withSystemMessage=true; scheduled: FTTT withSystemMessage=true; suspended: FTFT withSystemMessage=true; suspended+scheduled: FTFT withSystemMessage=true; closed: FFFF withSystemMessage=false;
```

stderr:

```diff
+ 
+ thread 'main' (118327112) panicked at rakka-actor/examples/classic-parity/mailbox_cases.rs:240:5:
+ assertion `left == right` failed
+   left: Ok(())
+  right: Err("Resume")
+ note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### mb-0009

Arguments: `mailbox-case`

status:

```diff
expected exit code 0, got exit code 101
```

stdout:

```diff
- MBX-MQ-01 default queue is FIFO and counts -> new: n=0 has=false | enqueue n=1 has=true n=2 has=true n=3 has=true | dequeue [a, b, c] then null | after n=0 has=false
```

stderr:

```diff
+ 
+ thread 'main' (118327116) panicked at rakka-actor/examples/classic-parity/mailbox_cases.rs:240:5:
+ assertion `left == right` failed
+   left: Ok(())
+  right: Err("Resume")
+ note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### mb-0010

Arguments: `mailbox-case`

status:

```diff
expected exit code 0, got exit code 101
```

stdout:

```diff
- MBX-MQ-02 default queue stays FIFO when enqueue and dequeue interleave -> first=a rest=[b, c]
```

stderr:

```diff
+ 
+ thread 'main' (118327120) panicked at rakka-actor/examples/classic-parity/mailbox_cases.rs:240:5:
+ assertion `left == right` failed
+   left: Ok(())
+  right: Err("Resume")
+ note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### mb-0011

Arguments: `mailbox-case`

status:

```diff
expected exit code 0, got exit code 101
```

stdout:

```diff
- MBX-MQ-03 cleanUp moves everything to the dead-letters queue in order -> source n=0 has=false | target n=3 has=true order [a, b, c]
```

stderr:

```diff
+ 
+ thread 'main' (118327124) panicked at rakka-actor/examples/classic-parity/mailbox_cases.rs:240:5:
+ assertion `left == right` failed
+   left: Ok(())
+  right: Err("Resume")
+ note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### mb-0012

Arguments: `mailbox-case`

status:

```diff
expected exit code 0, got exit code 101
```

stdout:

```diff
- MBX-SQ-01 system queue drains earliest first -> has=true drain=[Suspend, Resume, Terminate] | after has=false drain=[]
```

stderr:

```diff
+ 
+ thread 'main' (118327128) panicked at rakka-actor/examples/classic-parity/mailbox_cases.rs:240:5:
+ assertion `left == right` failed
+   left: Ok(())
+  right: Err("Resume")
+ note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### mb-0013

Arguments: `mailbox-case`

status:

```diff
expected exit code 0, got exit code 101
```

stdout:

```diff
- MBX-SQ-02 sentinel NoMessage closes the system queue -> drain=[Suspend] | afterSentinel has=false drain=[] (actor==null: dropped, not forwarded)
```

stderr:

```diff
+ 
+ thread 'main' (118327132) panicked at rakka-actor/examples/classic-parity/mailbox_cases.rs:240:5:
+ assertion `left == right` failed
+   left: Ok(())
+  right: Err("Resume")
+ note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### ok-user-info-lone-surrog

Arguments: `path-from-string`

status:

```diff
expected exit code 0, got exit code 2
```

stdout:

```diff
- {"ok":true,"toString":"pekko://sys\ud800@host:2552/user","name":"user","elements":["user"],"uid":0,"toStringWithoutAddress":"/user","toSerializationFormat":"pekko://sys\ud800@host:2552/user","address":{"protocol":"pekko","system":"sys\ud800","host":"host","port":2552,"toString":"pekko://sys\ud800@host:2552"}}
```

stderr:

```diff
+ input outside the API domain: lone UTF-16 surrogate (public inputs are &str; decision Q-X-17)
```

### terminate-await-twice

Arguments: `scenario`

status:

```diff
expected exit code 0, got exit code 2
```

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"terminate","ok":true},{"op":"await-system-terminated","ok":true,"actor":"pekko://sys/","existenceConfirmed":true,"addressTerminated":true},{"op":"await-system-terminated","ok":true,"actor":"pekko://sys/","existenceConfirmed":true,"addressTerminated":true}]}
```

stderr:

```diff
+ timed out waiting for system termination
```

### terminate-empty

Arguments: `scenario`

status:

```diff
expected exit code 0, got exit code 2
```

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"terminate","ok":true},{"op":"await-system-terminated","ok":true,"actor":"pekko://sys/","existenceConfirmed":true,"addressTerminated":true}]}
```

stderr:

```diff
+ timed out waiting for system termination
```

### terminate-then-spawn-bad

Arguments: `scenario`

status:

```diff
expected exit code 0, got exit code 2
```

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"terminate","ok":true},{"op":"await-system-terminated","ok":true,"actor":"pekko://sys/","existenceConfirmed":true,"addressTerminated":true},{"op":"spawn","ok":false,"exception":"org.apache.pekko.actor.InvalidActorNameException","message":"Invalid actor path element [a b], illegal character [ ] at position: 1. Actor paths MUST: not start with `$`, include only ASCII letters and can only contain these special characters: -_.*$+:@&=,!~';."},{"op":"spawn","ok":false,"exception":"org.apache.pekko.actor.InvalidActorNameException","message":"actor name must not be empty"}]}
```

stderr:

```diff
+ timed out waiting for system termination
```

### terminate-then-spawn

Arguments: `scenario`

status:

```diff
expected exit code 0, got exit code 2
```

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"terminate","ok":true},{"op":"await-system-terminated","ok":true,"actor":"pekko://sys/","existenceConfirmed":true,"addressTerminated":true},{"op":"spawn","ok":false,"exception":"java.lang.IllegalStateException","message":"cannot create children while terminating or terminated"}]}
```

stderr:

```diff
+ timed out waiting for system termination
```

### terminate-then-terminate

Arguments: `scenario`

status:

```diff
expected exit code 0, got exit code 2
```

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"terminate","ok":true},{"op":"await-system-terminated","ok":true,"actor":"pekko://sys/","existenceConfirmed":true,"addressTerminated":true},{"op":"terminate","ok":true},{"op":"await-system-terminated","ok":true,"actor":"pekko://sys/","existenceConfirmed":true,"addressTerminated":true}]}
```

stderr:

```diff
+ timed out waiting for system termination
```

### terminate-twice

Arguments: `scenario`

status:

```diff
expected exit code 0, got exit code 2
```

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"terminate","ok":true},{"op":"terminate","ok":true},{"op":"await-system-terminated","ok":true,"actor":"pekko://sys/","existenceConfirmed":true,"addressTerminated":true}]}
```

stderr:

```diff
+ timed out waiting for system termination
```

### w04-watch-already-termin

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"subscribe","ok":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/w0","name":"w0","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/t","name":"t","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/w","name":"w","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true},{"op":"stop","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/t","is":"t","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/t"}]},{"op":"do","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/t","is":"t","existenceConfirmed":false,"addressTerminated":false,"sender":"pekko://sys/user/t"}]},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"dead-letter","msg":"x","sender":"pekko://sys/deadLetters","recipient":"pekko://sys/user/t"}]},{"op":"events-now","ok":true,"events":[{"ev":"dead-letter","msg":"x","sender":"pekko://sys/deadLetters","recipient":"pekko://sys/user/t"}]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"subscribe","ok":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/w0","name":"w0","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/t","name":"t","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/w","name":"w","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true},{"op":"stop","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/t","is":"t","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/t"}]},{"op":"do","ok":true},{"op":"await-events","ok":false,"events":[{"ev":"pre-start"}]},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"dead-letter","msg":"x","sender":"pekko://sys/deadLetters","recipient":"pekko://sys/user/t"}]},{"op":"events-now","ok":true,"events":[{"ev":"dead-letter","msg":"x","sender":"pekko://sys/deadLetters","recipient":"pekko://sys/user/t"}]}]}
```

### w05-watch-dead-twice-two

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/w0","name":"w0","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/t","name":"t","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/w","name":"w","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true},{"op":"stop","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/t","is":"t","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/t"}]},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/t","is":"t","existenceConfirmed":false,"addressTerminated":false,"sender":"pekko://sys/user/t"}]},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/t","is":"t","existenceConfirmed":false,"addressTerminated":false,"sender":"pekko://sys/user/t"},{"ev":"terminated","actor":"pekko://sys/user/t","is":"t","existenceConfirmed":false,"addressTerminated":false,"sender":"pekko://sys/user/t"}]},{"op":"do","ok":true,"children":[]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/t","is":"t","existenceConfirmed":false,"addressTerminated":false,"sender":"pekko://sys/user/t"},{"ev":"terminated","actor":"pekko://sys/user/t","is":"t","existenceConfirmed":false,"addressTerminated":false,"sender":"pekko://sys/user/t"},{"ev":"received","msg":"ping","sender":"pekko://sys/deadLetters"}]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/w0","name":"w0","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/t","name":"t","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/w","name":"w","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true},{"op":"stop","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/t","is":"t","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/t"}]},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"await-events","ok":false,"events":[{"ev":"pre-start"}]},{"op":"tell","ok":true},{"op":"await-events","ok":false,"events":[{"ev":"pre-start"},{"ev":"received","msg":"ping","sender":"pekko://sys/deadLetters"}]},{"op":"do","ok":true,"children":[]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"received","msg":"ping","sender":"pekko://sys/deadLetters"}]}]}
```
