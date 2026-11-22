Verdict path: full replay (2961 cases); outcome DETECTED

# App replay

Build: `$TMPDIR/pekko-mutants-36436-f3qktghn/target-prestart-failure-fixed-text/release/examples/classic-parity`

Passed: 2956/2961

Fallback rate: not reported by this build

## Failing cases

### init-1t-grandchild-text

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/g","name":"g","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/g/p","name":"p","parent":"pekko://sys/user/g","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/g/p/k","name":"k","parent":"pekko://sys/user/g/p","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"decide","cause":"org.apache.pekko.actor.ActorInitializationException: pekko://sys/user/g/p/k: exception during creation, root cause message: [init]","directive":"Stop"}]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"decide","cause":"org.apache.pekko.actor.ActorInitializationException: pekko://sys/user/g/p/k: exception during creation, root cause message: [init]","directive":"Stop"}]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/g","name":"g","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/g/p","name":"p","parent":"pekko://sys/user/g","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/g/p/k","name":"k","parent":"pekko://sys/user/g/p","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"decide","cause":"org.apache.pekko.actor.ActorInitializationException: exception during creation","directive":"Stop"}]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"decide","cause":"org.apache.pekko.actor.ActorInitializationException: exception during creation","directive":"Stop"}]}]}
```

### init-1t-prestart-text

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/k","name":"k","parent":"pekko://sys/user/p","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"decide","cause":"org.apache.pekko.actor.ActorInitializationException: pekko://sys/user/p/k: exception during creation, root cause message: [init]","directive":"Stop"}]},{"op":"await-child-gone","ok":true,"gone":true},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"decide","cause":"org.apache.pekko.actor.ActorInitializationException: pekko://sys/user/p/k: exception during creation, root cause message: [init]","directive":"Stop"}]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/k","name":"k","parent":"pekko://sys/user/p","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"decide","cause":"org.apache.pekko.actor.ActorInitializationException: exception during creation","directive":"Stop"}]},{"op":"await-child-gone","ok":true,"gone":true},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"decide","cause":"org.apache.pekko.actor.ActorInitializationException: exception during creation","directive":"Stop"}]}]}
```

### init-grandchild-text

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/g","name":"g","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/g/p","name":"p","parent":"pekko://sys/user/g","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/g/p/k","name":"k","parent":"pekko://sys/user/g/p","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"decide","cause":"org.apache.pekko.actor.ActorInitializationException: pekko://sys/user/g/p/k: exception during creation, root cause message: [init]","directive":"Stop"}]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"decide","cause":"org.apache.pekko.actor.ActorInitializationException: pekko://sys/user/g/p/k: exception during creation, root cause message: [init]","directive":"Stop"}]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/g","name":"g","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/g/p","name":"p","parent":"pekko://sys/user/g","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/g/p/k","name":"k","parent":"pekko://sys/user/g/p","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"decide","cause":"org.apache.pekko.actor.ActorInitializationException: exception during creation","directive":"Stop"}]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"decide","cause":"org.apache.pekko.actor.ActorInitializationException: exception during creation","directive":"Stop"}]}]}
```

### init-prestart-text

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/k","name":"k","parent":"pekko://sys/user/p","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"decide","cause":"org.apache.pekko.actor.ActorInitializationException: pekko://sys/user/p/k: exception during creation, root cause message: [init]","directive":"Stop"}]},{"op":"await-child-gone","ok":true,"gone":true},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"decide","cause":"org.apache.pekko.actor.ActorInitializationException: pekko://sys/user/p/k: exception during creation, root cause message: [init]","directive":"Stop"}]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/k","name":"k","parent":"pekko://sys/user/p","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"decide","cause":"org.apache.pekko.actor.ActorInitializationException: exception during creation","directive":"Stop"}]},{"op":"await-child-gone","ok":true,"gone":true},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"decide","cause":"org.apache.pekko.actor.ActorInitializationException: exception during creation","directive":"Stop"}]}]}
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
