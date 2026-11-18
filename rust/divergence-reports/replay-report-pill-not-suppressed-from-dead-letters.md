# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-3pr9tpx_/target-pill-not-suppressed-from-dead-letters/release/examples/classic-parity`

Passed: 1043/1045

Fallback rate: not reported by this build

## Failing cases

### dl-pill-to-dead-actor-su

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"subscribe","ok":true},{"op":"tell","ok":true},{"op":"await-stopped","ok":true,"terminated":true},{"op":"tell","ok":true},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"dead-letter","msg":"after","sender":"pekko://sys/deadLetters","recipient":"pekko://sys/user/a"}]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"subscribe","ok":true},{"op":"tell","ok":true},{"op":"await-stopped","ok":true,"terminated":true},{"op":"tell","ok":true},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"dead-letter","msg":{"class":"?"},"sender":"pekko://sys/deadLetters","recipient":"pekko://sys/user/a"}]}]}
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
