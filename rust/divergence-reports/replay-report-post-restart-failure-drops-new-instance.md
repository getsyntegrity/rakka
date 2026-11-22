Verdict path: full replay (2961 cases); outcome DETECTED

# App replay

Build: `$TMPDIR/pekko-mutants-36436-f3qktghn/target-post-restart-failure-drops-new-instance/release/examples/classic-parity`

Passed: 2958/2961

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

### postr-1t-text

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/k","name":"k","parent":"pekko://sys/user/p","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"received","msg":"boom","sender":"pekko://sys/deadLetters"},{"ev":"pre-restart","reason":"java.lang.IllegalStateException: boom","msg":"boom"},{"ev":"post-stop"},{"ev":"post-restart","reason":"java.lang.IllegalStateException: boom"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"decide","cause":"java.lang.IllegalStateException: boom","directive":"Restart"},{"ev":"decide","cause":"org.apache.pekko.actor.PostRestartException: pekko://sys/user/p/k: exception post restart (class java.lang.IllegalStateException)","directive":"Stop"}]},{"op":"await-child-gone","ok":true,"gone":true},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"received","msg":"boom","sender":"pekko://sys/deadLetters"},{"ev":"pre-restart","reason":"java.lang.IllegalStateException: boom","msg":"boom"},{"ev":"post-stop"},{"ev":"post-restart","reason":"java.lang.IllegalStateException: boom"},{"ev":"post-stop"}]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"decide","cause":"java.lang.IllegalStateException: boom","directive":"Restart"},{"ev":"decide","cause":"org.apache.pekko.actor.PostRestartException: pekko://sys/user/p/k: exception post restart (class java.lang.IllegalStateException)","directive":"Stop"}]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/k","name":"k","parent":"pekko://sys/user/p","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"received","msg":"boom","sender":"pekko://sys/deadLetters"},{"ev":"pre-restart","reason":"java.lang.IllegalStateException: boom","msg":"boom"},{"ev":"post-stop"},{"ev":"post-restart","reason":"java.lang.IllegalStateException: boom"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"decide","cause":"java.lang.IllegalStateException: boom","directive":"Restart"},{"ev":"decide","cause":"org.apache.pekko.actor.PostRestartException: pekko://sys/user/p/k: exception post restart (class java.lang.IllegalStateException)","directive":"Stop"}]},{"op":"await-child-gone","ok":true,"gone":true},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"received","msg":"boom","sender":"pekko://sys/deadLetters"},{"ev":"pre-restart","reason":"java.lang.IllegalStateException: boom","msg":"boom"},{"ev":"post-stop"},{"ev":"post-restart","reason":"java.lang.IllegalStateException: boom"}]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"decide","cause":"java.lang.IllegalStateException: boom","directive":"Restart"},{"ev":"decide","cause":"org.apache.pekko.actor.PostRestartException: pekko://sys/user/p/k: exception post restart (class java.lang.IllegalStateException)","directive":"Stop"}]}]}
```

### postr-text

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/k","name":"k","parent":"pekko://sys/user/p","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"received","msg":"boom","sender":"pekko://sys/deadLetters"},{"ev":"pre-restart","reason":"java.lang.IllegalStateException: boom","msg":"boom"},{"ev":"post-stop"},{"ev":"post-restart","reason":"java.lang.IllegalStateException: boom"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"decide","cause":"java.lang.IllegalStateException: boom","directive":"Restart"},{"ev":"decide","cause":"org.apache.pekko.actor.PostRestartException: pekko://sys/user/p/k: exception post restart (class java.lang.IllegalStateException)","directive":"Stop"}]},{"op":"await-child-gone","ok":true,"gone":true},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"received","msg":"boom","sender":"pekko://sys/deadLetters"},{"ev":"pre-restart","reason":"java.lang.IllegalStateException: boom","msg":"boom"},{"ev":"post-stop"},{"ev":"post-restart","reason":"java.lang.IllegalStateException: boom"},{"ev":"post-stop"}]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"decide","cause":"java.lang.IllegalStateException: boom","directive":"Restart"},{"ev":"decide","cause":"org.apache.pekko.actor.PostRestartException: pekko://sys/user/p/k: exception post restart (class java.lang.IllegalStateException)","directive":"Stop"}]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/k","name":"k","parent":"pekko://sys/user/p","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"received","msg":"boom","sender":"pekko://sys/deadLetters"},{"ev":"pre-restart","reason":"java.lang.IllegalStateException: boom","msg":"boom"},{"ev":"post-stop"},{"ev":"post-restart","reason":"java.lang.IllegalStateException: boom"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"decide","cause":"java.lang.IllegalStateException: boom","directive":"Restart"},{"ev":"decide","cause":"org.apache.pekko.actor.PostRestartException: pekko://sys/user/p/k: exception post restart (class java.lang.IllegalStateException)","directive":"Stop"}]},{"op":"await-child-gone","ok":true,"gone":true},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"received","msg":"boom","sender":"pekko://sys/deadLetters"},{"ev":"pre-restart","reason":"java.lang.IllegalStateException: boom","msg":"boom"},{"ev":"post-stop"},{"ev":"post-restart","reason":"java.lang.IllegalStateException: boom"}]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"decide","cause":"java.lang.IllegalStateException: boom","directive":"Restart"},{"ev":"decide","cause":"org.apache.pekko.actor.PostRestartException: pekko://sys/user/p/k: exception post restart (class java.lang.IllegalStateException)","directive":"Stop"}]}]}
```
