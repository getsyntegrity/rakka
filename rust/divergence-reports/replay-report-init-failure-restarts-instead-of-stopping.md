# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-3pr9tpx_/target-init-failure-restarts-instead-of-stopping/release/examples/classic-parity`

Passed: 1042/1045

Fallback rate: not reported by this build

## Failing cases

### i01-prestart-failure-sto

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-stopped","ok":true,"terminated":true},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"}]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-stopped","ok":true,"terminated":true},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"post-restart","reason":"org.apache.pekko.actor.ActorInitializationException: exception during creation"},{"ev":"pre-start"}]}]}
```

### i02-prestart-failure-wat

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/k","name":"k","parent":"pekko://sys/user/p","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-stopped","ok":true,"terminated":true},{"op":"do","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/k","is":"k","existenceConfirmed":false,"addressTerminated":false,"sender":"pekko://sys/user/p/k"}]},{"op":"do","ok":true,"children":[]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"}]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/k","name":"k","parent":"pekko://sys/user/p","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-stopped","ok":true,"terminated":true},{"op":"do","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/k","is":"k","existenceConfirmed":false,"addressTerminated":false,"sender":"pekko://sys/user/p/k"}]},{"op":"do","ok":true,"children":[]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"post-restart","reason":"org.apache.pekko.actor.ActorInitializationException: exception during creation"},{"ev":"pre-start"}]}]}
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
