# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-3pr9tpx_/target-unwatch-keeps-queued-terminated/release/examples/classic-parity`

Passed: 1042/1045

Fallback rate: not reported by this build

## Failing cases

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

### w21-unwatch-cancels-queu

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/w","name":"w","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/w0","name":"w0","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/t","name":"t","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"received","msg":"block","sender":"pekko://sys/deadLetters"}]},{"op":"do","ok":true,"queued":true},{"op":"stop","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/t","is":"t","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/t"}]},{"op":"release","ok":true},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"received","msg":"block","sender":"pekko://sys/deadLetters"},{"ev":"received","msg":"ping","sender":"pekko://sys/deadLetters"}]},{"op":"do","ok":true,"children":[]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"received","msg":"block","sender":"pekko://sys/deadLetters"},{"ev":"received","msg":"ping","sender":"pekko://sys/deadLetters"}]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/w","name":"w","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/w0","name":"w0","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/t","name":"t","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"received","msg":"block","sender":"pekko://sys/deadLetters"}]},{"op":"do","ok":true,"queued":true},{"op":"stop","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/t","is":"t","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/t"}]},{"op":"release","ok":true},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"received","msg":"block","sender":"pekko://sys/deadLetters"},{"ev":"received","msg":"ping","sender":"pekko://sys/deadLetters"}]},{"op":"do","ok":true,"children":[]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"received","msg":"block","sender":"pekko://sys/deadLetters"},{"ev":"received","msg":"ping","sender":"pekko://sys/deadLetters"},{"ev":"terminated","actor":"pekko://sys/user/t","is":"t","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/t"}]}]}
```

### w23-parent-unwatch-cance

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/w","name":"w","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/w0","name":"w0","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/w/t","name":"t","parent":"pekko://sys/user/w","hasUid":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"received","msg":"block","sender":"pekko://sys/deadLetters"}]},{"op":"do","ok":true,"queued":true},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/w/t","is":"t","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/w/t"}]},{"op":"release","ok":true},{"op":"await-child-gone","ok":true,"gone":true},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"received","msg":"block","sender":"pekko://sys/deadLetters"},{"ev":"received","msg":"ping","sender":"pekko://sys/deadLetters"}]},{"op":"do","ok":true,"children":[]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"received","msg":"block","sender":"pekko://sys/deadLetters"},{"ev":"received","msg":"ping","sender":"pekko://sys/deadLetters"}]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/w","name":"w","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/w0","name":"w0","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/w/t","name":"t","parent":"pekko://sys/user/w","hasUid":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"received","msg":"block","sender":"pekko://sys/deadLetters"}]},{"op":"do","ok":true,"queued":true},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/w/t","is":"t","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/w/t"}]},{"op":"release","ok":true},{"op":"await-child-gone","ok":true,"gone":true},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"received","msg":"block","sender":"pekko://sys/deadLetters"},{"ev":"terminated","actor":"pekko://sys/user/w/t","is":"t","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/w/t"}]},{"op":"do","ok":true,"children":[]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"received","msg":"block","sender":"pekko://sys/deadLetters"},{"ev":"terminated","actor":"pekko://sys/user/w/t","is":"t","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/w/t"},{"ev":"received","msg":"ping","sender":"pekko://sys/deadLetters"}]}]}
```
