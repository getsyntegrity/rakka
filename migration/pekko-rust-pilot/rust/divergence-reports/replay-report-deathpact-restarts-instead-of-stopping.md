# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-i2z9eagu/target-deathpact-restarts-instead-of-stopping/release/examples/classic-parity`

Passed: 562/566

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
