# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-c5f9c6on/target-children-match-by-path/release/examples/classic-parity`

Passed: 1043/1045

Fallback rate: not reported by this build

## Failing cases

### i08-old-ref-watch-existe

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/w","name":"w","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/t","name":"t","parent":"pekko://sys/user/p","hasUid":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/t","name":"t","parent":"pekko://sys/user/p","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":false,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":false,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"compare-refs","ok":true,"pathEqual":true,"refEqual":false,"uidEqual":false,"pathStringEqual":true,"aHasUid":true,"bHasUid":true},{"op":"do","ok":true,"found":true,"path":"pekko://sys/user/p/t","name":"t","parent":"pekko://sys/user/p","hasUid":true,"is":"t2"},{"op":"do","ok":true,"children":["pekko://sys/user/p/t"]},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"received","msg":"alive","sender":"pekko://sys/deadLetters"}]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/w","name":"w","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/t","name":"t","parent":"pekko://sys/user/p","hasUid":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/t","name":"t","parent":"pekko://sys/user/p","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":false,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/t"},{"ev":"terminated","actor":"pekko://sys/user/p/t","is":"t1","existenceConfirmed":false,"addressTerminated":false,"sender":"pekko://sys/user/p/t"}]},{"op":"compare-refs","ok":true,"pathEqual":true,"refEqual":false,"uidEqual":false,"pathStringEqual":true,"aHasUid":true,"bHasUid":true},{"op":"do","ok":true,"found":false},{"op":"do","ok":true,"children":[]},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"received","msg":"alive","sender":"pekko://sys/deadLetters"}]}]}
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
