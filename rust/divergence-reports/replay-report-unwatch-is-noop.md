# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-3pr9tpx_/target-unwatch-is-noop/release/examples/classic-parity`

Passed: 1041/1045

Fallback rate: not reported by this build

## Failing cases

### i06-unwatch-new-keeps-no

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/w","name":"w","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/t","name":"t","parent":"pekko://sys/user/p","hasUid":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/t","name":"t","parent":"pekko://sys/user/p","hasUid":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t2","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"},{"ev":"received","msg":"ping","sender":"pekko://sys/deadLetters"}]},{"op":"do","ok":true,"children":[]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"},{"ev":"received","msg":"ping","sender":"pekko://sys/deadLetters"}]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/w","name":"w","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/t","name":"t","parent":"pekko://sys/user/p","hasUid":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/t","name":"t","parent":"pekko://sys/user/p","hasUid":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t2","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t2","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"do","ok":true,"children":[]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t2","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"},{"ev":"received","msg":"ping","sender":"pekko://sys/deadLetters"}]}]}
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

### w07-unwatch-before-stop

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/w","name":"w","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/w0","name":"w0","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/t","name":"t","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"stop","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/t","is":"t","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/t"}]},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"received","msg":"ping","sender":"pekko://sys/deadLetters"}]},{"op":"do","ok":true,"children":[]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"received","msg":"ping","sender":"pekko://sys/deadLetters"}]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/w","name":"w","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/w0","name":"w0","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/t","name":"t","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"stop","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/t","is":"t","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/t"}]},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/t","is":"t","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/t"}]},{"op":"do","ok":true,"children":[]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/t","is":"t","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/t"},{"ev":"received","msg":"ping","sender":"pekko://sys/deadLetters"}]}]}
```

### w09-unwatch-twice

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/w","name":"w","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/w0","name":"w0","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/t","name":"t","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"stop","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/t","is":"t","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/t"}]},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"received","msg":"ping","sender":"pekko://sys/deadLetters"}]},{"op":"do","ok":true,"children":[]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"received","msg":"ping","sender":"pekko://sys/deadLetters"}]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/w","name":"w","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/w0","name":"w0","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/t","name":"t","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"stop","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/t","is":"t","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/t"}]},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/t","is":"t","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/t"}]},{"op":"do","ok":true,"children":[]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/t","is":"t","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/t"},{"ev":"received","msg":"ping","sender":"pekko://sys/deadLetters"}]}]}
```
