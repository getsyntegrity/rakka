# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-3pr9tpx_/target-deathpact-restarts-instead-of-stopping/release/examples/classic-parity`

Passed: 1040/1045

Fallback rate: not reported by this build

## Failing cases

### d01-unhandled-terminated

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/x","name":"x","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/w","name":"w","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/v","name":"v","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"stop","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/w","is":"w","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/w"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"post-stop"}]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"post-stop"}]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/x","name":"x","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/w","name":"w","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/v","name":"v","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"stop","ok":true},{"op":"await-events","ok":false,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"pre-restart","reason":"org.apache.pekko.actor.DeathPactException: Monitored actor [Actor[pekko://sys/user/x]] terminated","msg":{"class":"?"}}]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"pre-restart","reason":"org.apache.pekko.actor.DeathPactException: Monitored actor [Actor[pekko://sys/user/x]] terminated","msg":{"class":"?"}},{"ev":"post-stop"},{"ev":"post-restart","reason":"org.apache.pekko.actor.DeathPactException: Monitored actor [Actor[pekko://sys/user/x]] terminated"},{"ev":"pre-start"}]}]}
```

### d03-no-restart-on-deathp

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/x","name":"x","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/w","name":"w","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true},{"op":"stop","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"post-stop"}]},{"op":"await-stopped","ok":true,"terminated":true},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"post-stop"}]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/x","name":"x","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/w","name":"w","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true},{"op":"stop","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"pre-restart","reason":"org.apache.pekko.actor.DeathPactException: Monitored actor [Actor[pekko://sys/user/x]] terminated","msg":{"class":"?"}}]},{"op":"await-stopped","ok":true,"terminated":false},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"pre-restart","reason":"org.apache.pekko.actor.DeathPactException: Monitored actor [Actor[pekko://sys/user/x]] terminated","msg":{"class":"?"}},{"ev":"post-stop"},{"ev":"post-restart","reason":"org.apache.pekko.actor.DeathPactException: Monitored actor [Actor[pekko://sys/user/x]] terminated"},{"ev":"pre-start"}]}]}
```

### d04-deathpact-in-child

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/x","name":"x","parent":"pekko://sys/user/p","hasUid":true},{"op":"do","ok":true,"path":"pekko://sys/user/p/c","name":"c","parent":"pekko://sys/user/p","hasUid":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/c","is":"c","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/c"}]},{"op":"do","ok":true,"children":[]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"post-stop"}]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/x","name":"x","parent":"pekko://sys/user/p","hasUid":true},{"op":"do","ok":true,"path":"pekko://sys/user/p/c","name":"c","parent":"pekko://sys/user/p","hasUid":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"do","ok":true},{"op":"await-events","ok":false,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"children":["pekko://sys/user/p/c"]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"pre-restart","reason":"org.apache.pekko.actor.DeathPactException: Monitored actor [Actor[pekko://sys/user/p/x]] terminated","msg":{"class":"?"}},{"ev":"post-stop"},{"ev":"post-restart","reason":"org.apache.pekko.actor.DeathPactException: Monitored actor [Actor[pekko://sys/user/p/x]] terminated"},{"ev":"pre-start"}]}]}
```

### d05-handles-list-ignores

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/x","name":"x","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/w","name":"w","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true},{"op":"stop","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"post-stop"}]},{"op":"await-stopped","ok":true,"terminated":true},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"post-stop"}]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/x","name":"x","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/w","name":"w","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true},{"op":"stop","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"pre-restart","reason":"org.apache.pekko.actor.DeathPactException: Monitored actor [Actor[pekko://sys/user/x]] terminated","msg":{"class":"?"}}]},{"op":"await-stopped","ok":true,"terminated":false},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"pre-restart","reason":"org.apache.pekko.actor.DeathPactException: Monitored actor [Actor[pekko://sys/user/x]] terminated","msg":{"class":"?"}},{"ev":"post-stop"},{"ev":"post-restart","reason":"org.apache.pekko.actor.DeathPactException: Monitored actor [Actor[pekko://sys/user/x]] terminated"},{"ev":"pre-start"}]}]}
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
