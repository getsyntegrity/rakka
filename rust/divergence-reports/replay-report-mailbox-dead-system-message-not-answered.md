# App replay

Build: `$TMPDIR/pekko-mutants-_77o_7ue/target-mailbox-dead-system-message-not-answered/release/examples/classic-parity`

Passed: 2807/2822

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
