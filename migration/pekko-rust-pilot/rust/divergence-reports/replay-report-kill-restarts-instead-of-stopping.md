# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-i2z9eagu/target-kill-restarts-instead-of-stopping/release/examples/classic-parity`

Passed: 558/566

Fallback rate: not reported by this build

## Failing cases

### e07-g-default-stops-kill

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/g","name":"g","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/g/p","name":"p","parent":"pekko://sys/user/g","hasUid":true},{"op":"do","ok":true,"path":"pekko://sys/user/g/p/c","name":"c","parent":"pekko://sys/user/g/p","hasUid":true},{"op":"do","ok":true},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"decide","cause":"org.apache.pekko.actor.ActorKilledException: Kill","directive":"Stop"},{"ev":"terminated","actor":"pekko://sys/user/g/p","is":"p","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/g/p"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"decide","cause":"org.apache.pekko.actor.ActorKilledException: Kill","directive":"Escalate"},{"ev":"post-stop"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"post-stop"}]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"decide","cause":"org.apache.pekko.actor.ActorKilledException: Kill","directive":"Stop"},{"ev":"terminated","actor":"pekko://sys/user/g/p","is":"p","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/g/p"}]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"decide","cause":"org.apache.pekko.actor.ActorKilledException: Kill","directive":"Escalate"},{"ev":"post-stop"}]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"post-stop"}]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/g","name":"g","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/g/p","name":"p","parent":"pekko://sys/user/g","hasUid":true},{"op":"do","ok":true,"path":"pekko://sys/user/g/p/c","name":"c","parent":"pekko://sys/user/g/p","hasUid":true},{"op":"do","ok":true},{"op":"tell","ok":true},{"op":"await-events","ok":false,"events":[{"ev":"pre-start"},{"ev":"decide","cause":"org.apache.pekko.actor.ActorKilledException: Kill","directive":"Restart"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"decide","cause":"org.apache.pekko.actor.ActorKilledException: Kill","directive":"Escalate"},{"ev":"pre-restart","reason":"org.apache.pekko.actor.ActorKilledException: Kill","msg":{"class":"org.apache.pekko.dispatch.sysmsg.Failed"}}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"post-stop"}]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"decide","cause":"org.apache.pekko.actor.ActorKilledException: Kill","directive":"Restart"}]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"decide","cause":"org.apache.pekko.actor.ActorKilledException: Kill","directive":"Escalate"},{"ev":"pre-restart","reason":"org.apache.pekko.actor.ActorKilledException: Kill","msg":{"class":"org.apache.pekko.dispatch.sysmsg.Failed"}},{"ev":"post-stop"},{"ev":"post-restart","reason":"org.apache.pekko.actor.ActorKilledException: Kill"},{"ev":"pre-start"}]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"post-stop"}]}]}
```

### k01-kill-stops

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/w","name":"w","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/a","is":"a","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/a"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"post-stop"}]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"post-stop"}]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/a","is":"a","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/a"}]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/w","name":"w","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true},{"op":"tell","ok":true},{"op":"await-events","ok":false,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"pre-restart","reason":"org.apache.pekko.actor.ActorKilledException: Kill","msg":{"class":"org.apache.pekko.actor.Kill$"}}]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"pre-restart","reason":"org.apache.pekko.actor.ActorKilledException: Kill","msg":{"class":"org.apache.pekko.actor.Kill$"}},{"ev":"post-stop"},{"ev":"post-restart","reason":"org.apache.pekko.actor.ActorKilledException: Kill"},{"ev":"pre-start"}]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"}]}]}
```

### k02-kill-no-restart-hook

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"post-stop"}]},{"op":"await-stopped","ok":true,"terminated":true},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"post-stop"}]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"pre-restart","reason":"org.apache.pekko.actor.ActorKilledException: Kill","msg":{"class":"org.apache.pekko.actor.Kill$"}}]},{"op":"await-stopped","ok":true,"terminated":false},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"pre-restart","reason":"org.apache.pekko.actor.ActorKilledException: Kill","msg":{"class":"org.apache.pekko.actor.Kill$"}},{"ev":"post-stop"},{"ev":"post-restart","reason":"org.apache.pekko.actor.ActorKilledException: Kill"},{"ev":"pre-start"}]}]}
```

### k03-after-kill-dead-lett

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"subscribe","ok":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/w","name":"w","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/a","is":"a","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/a"}]},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"dead-letter","msg":"late","sender":"pekko://sys/deadLetters","recipient":"pekko://sys/user/a"}]},{"op":"events-now","ok":true,"events":[{"ev":"dead-letter","msg":"late","sender":"pekko://sys/deadLetters","recipient":"pekko://sys/user/a"}]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"subscribe","ok":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"spawn","ok":true,"path":"pekko://sys/user/w","name":"w","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true},{"op":"tell","ok":true},{"op":"await-events","ok":false,"events":[{"ev":"pre-start"}]},{"op":"tell","ok":true},{"op":"await-events","ok":false,"events":[]},{"op":"events-now","ok":true,"events":[]}]}
```

### k04-queued-after-kill-de

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"subscribe","ok":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"received","msg":"block","sender":"pekko://sys/deadLetters"}]},{"op":"tell","ok":true},{"op":"tell","ok":true},{"op":"tell","ok":true},{"op":"release","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"dead-letter","msg":"m1","sender":"pekko://sys/deadLetters","recipient":"pekko://sys/user/a"},{"ev":"dead-letter","msg":"m2","sender":"pekko://sys/deadLetters","recipient":"pekko://sys/user/a"}]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"received","msg":"block","sender":"pekko://sys/deadLetters"},{"ev":"post-stop"}]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"subscribe","ok":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"received","msg":"block","sender":"pekko://sys/deadLetters"}]},{"op":"tell","ok":true},{"op":"tell","ok":true},{"op":"tell","ok":true},{"op":"release","ok":true},{"op":"await-events","ok":false,"events":[]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"received","msg":"block","sender":"pekko://sys/deadLetters"},{"ev":"pre-restart","reason":"org.apache.pekko.actor.ActorKilledException: Kill","msg":{"class":"org.apache.pekko.actor.Kill$"}},{"ev":"post-stop"},{"ev":"post-restart","reason":"org.apache.pekko.actor.ActorKilledException: Kill"},{"ev":"pre-start"},{"ev":"received","msg":"m1","sender":"pekko://sys/deadLetters"},{"ev":"received","msg":"m2","sender":"pekko://sys/deadLetters"}]}]}
```

### k05-kill-child-parent-se

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/k","name":"k","parent":"pekko://sys/user/p","hasUid":true},{"op":"do","ok":true},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/k","is":"k","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/k"}]},{"op":"do","ok":true,"children":[]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p/k","is":"k","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p/k"}]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/k","name":"k","parent":"pekko://sys/user/p","hasUid":true},{"op":"do","ok":true},{"op":"tell","ok":true},{"op":"await-events","ok":false,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"children":["pekko://sys/user/p/k"]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"}]}]}
```

### k06-kill-parent-stops-ch

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/k","name":"k","parent":"pekko://sys/user/p","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/w","name":"w","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"terminated","actor":"pekko://sys/user/p","is":"p","existenceConfirmed":true,"addressTerminated":false,"sender":"pekko://sys/user/p"}]},{"op":"await-life","ok":true,"events":["k:post-stop","k:pre-start","p:post-stop","p:pre-start","w:pre-start","w:terminated:p"],"before":[true]},{"op":"do","ok":true,"children":[]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/k","name":"k","parent":"pekko://sys/user/p","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/w","name":"w","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true},{"op":"tell","ok":true},{"op":"await-events","ok":false,"events":[{"ev":"pre-start"}]},{"op":"await-life","ok":true,"events":["k:post-stop","k:pre-start","p:post-restart","p:post-stop","p:pre-restart","p:pre-start","p:pre-start","w:pre-start"],"before":[false]},{"op":"do","ok":true,"children":[]}]}
```

### k07-kill-twice

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"subscribe","ok":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"tell","ok":true},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"post-stop"}]},{"op":"await-events","ok":true,"events":[{"ev":"dead-letter","msg":{"class":"org.apache.pekko.actor.Kill$"},"sender":"pekko://sys/deadLetters","recipient":"pekko://sys/user/a"}]},{"op":"events-now","ok":true,"events":[{"ev":"dead-letter","msg":{"class":"org.apache.pekko.actor.Kill$"},"sender":"pekko://sys/deadLetters","recipient":"pekko://sys/user/a"}]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"subscribe","ok":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"tell","ok":true},{"op":"tell","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"pre-restart","reason":"org.apache.pekko.actor.ActorKilledException: Kill","msg":{"class":"org.apache.pekko.actor.Kill$"}}]},{"op":"await-events","ok":false,"events":[]},{"op":"events-now","ok":true,"events":[]}]}
```
