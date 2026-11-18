# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-i2z9eagu/target-no-set-terminated-without-children/release/examples/classic-parity`

Passed: 562/566

Fallback rate: not reported by this build

## Failing cases

### h16-post-stop-spawn-no-c

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"stop","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"post-stop"},{"ev":"post-stop-spawn","ok":false,"exception":"java.lang.IllegalStateException","message":"cannot create children while terminating or terminated"}]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"post-stop"},{"ev":"post-stop-spawn","ok":false,"exception":"java.lang.IllegalStateException","message":"cannot create children while terminating or terminated"}]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"stop","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"post-stop"},{"ev":"post-stop-spawn","ok":true,"path":"pekko://sys/user/p/late","name":"late","parent":"pekko://sys/user/p","hasUid":true}]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"post-stop"},{"ev":"post-stop-spawn","ok":true,"path":"pekko://sys/user/p/late","name":"late","parent":"pekko://sys/user/p","hasUid":true}]}]}
```

### h18-post-stop-spawn-in-c

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/c","name":"c","parent":"pekko://sys/user/p","hasUid":true},{"op":"await-life","ok":true,"events":["c:pre-start","p:pre-start"],"before":[]},{"op":"do","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"post-stop"},{"ev":"post-stop-spawn","ok":false,"exception":"java.lang.IllegalStateException","message":"cannot create children while terminating or terminated"}]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"post-stop"},{"ev":"post-stop-spawn","ok":false,"exception":"java.lang.IllegalStateException","message":"cannot create children while terminating or terminated"}]},{"op":"do","ok":true,"children":[]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/c","name":"c","parent":"pekko://sys/user/p","hasUid":true},{"op":"await-life","ok":true,"events":["c:pre-start","p:pre-start"],"before":[]},{"op":"do","ok":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"},{"ev":"post-stop"},{"ev":"post-stop-spawn","ok":true,"path":"pekko://sys/user/p/c/late","name":"late","parent":"pekko://sys/user/p/c","hasUid":true}]},{"op":"events-now","ok":true,"events":[{"ev":"pre-start"},{"ev":"post-stop"},{"ev":"post-stop-spawn","ok":true,"path":"pekko://sys/user/p/c/late","name":"late","parent":"pekko://sys/user/p/c","hasUid":true}]},{"op":"do","ok":true,"children":[]}]}
```

### terminate-then-spawn-ano

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"terminate","ok":true},{"op":"await-system-terminated","ok":true,"actor":"pekko://sys/","existenceConfirmed":true,"addressTerminated":true},{"op":"spawn","ok":false,"exception":"java.lang.IllegalStateException","message":"cannot create children while terminating or terminated"}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"terminate","ok":true},{"op":"await-system-terminated","ok":true,"actor":"pekko://sys/","existenceConfirmed":true,"addressTerminated":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/$a","name":"$a","parent":"pekko://sys/user","hasUid":true}]}
```

### terminate-then-spawn

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"terminate","ok":true},{"op":"await-system-terminated","ok":true,"actor":"pekko://sys/","existenceConfirmed":true,"addressTerminated":true},{"op":"spawn","ok":false,"exception":"java.lang.IllegalStateException","message":"cannot create children while terminating or terminated"}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"terminate","ok":true},{"op":"await-system-terminated","ok":true,"actor":"pekko://sys/","existenceConfirmed":true,"addressTerminated":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/late","name":"late","parent":"pekko://sys/user","hasUid":true}]}
```
