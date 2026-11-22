Verdict path: full replay (2961 cases); outcome DETECTED

# App replay

Build: `$TMPDIR/pekko-mutants-36436-f3qktghn/target-factory-failure-not-reported/release/examples/classic-parity`

Passed: 2956/2961

Fallback rate: not reported by this build

## Failing cases

### ctor-1t-child-stops

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/k","name":"k","parent":"pekko://sys/user/p","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"decide","cause":"org.apache.pekko.actor.ActorInitializationException: pekko://sys/user/p/k: exception during creation, root cause message: [ctor]","directive":"Stop"}]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"decide","cause":"org.apache.pekko.actor.ActorInitializationException: pekko://sys/user/p/k: exception during creation, root cause message: [ctor]","directive":"Stop"}]},{"op":"await-child-gone","ok":true,"gone":true},{"op":"do","ok":true,"children":[]},{"op":"events-now","ok":true,"events":[]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/k","name":"k","parent":"pekko://sys/user/p","hasUid":true},{"op":"await-events","ok":false,"events":[{"ev":"pre-start"}]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-child-gone","ok":true,"gone":false},{"op":"do","ok":true,"children":["pekko://sys/user/p/k"]},{"op":"events-now","ok":true,"events":[]}]}
```

### ctor-1t-then-works

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/k","name":"k","parent":"pekko://sys/user/p","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"decide","cause":"org.apache.pekko.actor.ActorInitializationException: pekko://sys/user/p/k: exception during creation, root cause message: [ctor]","directive":"Stop"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/ok","name":"ok","parent":"pekko://sys/user/p","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"decide","cause":"org.apache.pekko.actor.ActorInitializationException: pekko://sys/user/p/k: exception during creation, root cause message: [ctor]","directive":"Stop"},{"ev":"received","msg":"alive","sender":"pekko://sys/deadLetters"}]},{"op":"await-child-gone","ok":true,"gone":true},{"op":"do","ok":true,"children":["pekko://sys/user/p/ok"]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/k","name":"k","parent":"pekko://sys/user/p","hasUid":true},{"op":"await-events","ok":false,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/ok","name":"ok","parent":"pekko://sys/user/p","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"tell","ok":true},{"op":"await-events","ok":false,"events":[{"ev":"pre-start"},{"ev":"received","msg":"alive","sender":"pekko://sys/deadLetters"}]},{"op":"await-child-gone","ok":true,"gone":false},{"op":"do","ok":true,"children":["pekko://sys/user/p/k","pekko://sys/user/p/ok"]}]}
```

### ctor-child-stops

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/k","name":"k","parent":"pekko://sys/user/p","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"decide","cause":"org.apache.pekko.actor.ActorInitializationException: pekko://sys/user/p/k: exception during creation, root cause message: [ctor]","directive":"Stop"}]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"decide","cause":"org.apache.pekko.actor.ActorInitializationException: pekko://sys/user/p/k: exception during creation, root cause message: [ctor]","directive":"Stop"}]},{"op":"await-child-gone","ok":true,"gone":true},{"op":"do","ok":true,"children":[]},{"op":"events-now","ok":true,"events":[]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/k","name":"k","parent":"pekko://sys/user/p","hasUid":true},{"op":"await-events","ok":false,"events":[{"ev":"pre-start"}]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-child-gone","ok":true,"gone":false},{"op":"do","ok":true,"children":["pekko://sys/user/p/k"]},{"op":"events-now","ok":true,"events":[]}]}
```

### ctor-then-works

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/k","name":"k","parent":"pekko://sys/user/p","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"decide","cause":"org.apache.pekko.actor.ActorInitializationException: pekko://sys/user/p/k: exception during creation, root cause message: [ctor]","directive":"Stop"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/ok","name":"ok","parent":"pekko://sys/user/p","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"decide","cause":"org.apache.pekko.actor.ActorInitializationException: pekko://sys/user/p/k: exception during creation, root cause message: [ctor]","directive":"Stop"},{"ev":"received","msg":"alive","sender":"pekko://sys/deadLetters"}]},{"op":"await-child-gone","ok":true,"gone":true},{"op":"do","ok":true,"children":["pekko://sys/user/p/ok"]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/k","name":"k","parent":"pekko://sys/user/p","hasUid":true},{"op":"await-events","ok":false,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/ok","name":"ok","parent":"pekko://sys/user/p","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"tell","ok":true},{"op":"await-events","ok":false,"events":[{"ev":"pre-start"},{"ev":"received","msg":"alive","sender":"pekko://sys/deadLetters"}]},{"op":"await-child-gone","ok":true,"gone":false},{"op":"do","ok":true,"children":["pekko://sys/user/p/k","pekko://sys/user/p/ok"]}]}
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
