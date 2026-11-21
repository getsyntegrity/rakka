Verdict path: full replay (2958 cases); outcome DETECTED

# App replay

Build: `$TMPDIR/pekko-mutants-14538-rtqx6ml7/target-actor-name-ending-in-an-escape-rejected/release/examples/classic-parity`

Passed: 2947/2958

Fallback rate: not reported by this build

## Failing cases

### dup-escape-differs-from

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/%61","name":"%61","parent":"pekko://sys/user","hasUid":true}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":false,"exception":"org.apache.pekko.actor.InvalidActorNameException","message":"Invalid actor path element [%61], illegal character [%] at position: 0. Actor paths MUST: not start with `$`, include only ASCII letters and can only contain these special characters: -_.*$+:@&=,!~';."}]}
```

### h06-child-name-09

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/ok%41","name":"ok%41","parent":"pekko://sys/user/p","hasUid":true},{"op":"do","ok":true,"children":["pekko://sys/user/p/ok%41"]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":false,"exception":"org.apache.pekko.actor.InvalidActorNameException","message":"Invalid actor path element [ok%41], illegal character [%] at position: 2. Actor paths MUST: not start with `$`, include only ASCII letters and can only contain these special characters: -_.*$+:@&=,!~';."},{"op":"do","ok":true,"children":[]}]}
```

### h07-names-case-and-escap

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/a","name":"a","parent":"pekko://sys/user/p","hasUid":true},{"op":"do","ok":true,"path":"pekko://sys/user/p/A","name":"A","parent":"pekko://sys/user/p","hasUid":true},{"op":"do","ok":true,"path":"pekko://sys/user/p/%61","name":"%61","parent":"pekko://sys/user/p","hasUid":true},{"op":"do","ok":false,"exception":"org.apache.pekko.actor.InvalidActorNameException","message":"actor name [a] is not unique!"},{"op":"do","ok":true,"children":["pekko://sys/user/p/%61","pekko://sys/user/p/A","pekko://sys/user/p/a"]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/p","name":"p","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"do","ok":true,"path":"pekko://sys/user/p/a","name":"a","parent":"pekko://sys/user/p","hasUid":true},{"op":"do","ok":true,"path":"pekko://sys/user/p/A","name":"A","parent":"pekko://sys/user/p","hasUid":true},{"op":"do","ok":false,"exception":"org.apache.pekko.actor.InvalidActorNameException","message":"Invalid actor path element [%61], illegal character [%] at position: 0. Actor paths MUST: not start with `$`, include only ASCII letters and can only contain these special characters: -_.*$+:@&=,!~';."},{"op":"do","ok":false,"exception":"org.apache.pekko.actor.InvalidActorNameException","message":"actor name [a] is not unique!"},{"op":"do","ok":true,"children":["pekko://sys/user/p/A","pekko://sys/user/p/a"]}]}
```

### name-ok-17

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/%41","name":"%41","parent":"pekko://sys/user","hasUid":true}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":false,"exception":"org.apache.pekko.actor.InvalidActorNameException","message":"Invalid actor path element [%41], illegal character [%] at position: 0. Actor paths MUST: not start with `$`, include only ASCII letters and can only contain these special characters: -_.*$+:@&=,!~';."}]}
```

### name-ok-19

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/a%4A","name":"a%4A","parent":"pekko://sys/user","hasUid":true}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":false,"exception":"org.apache.pekko.actor.InvalidActorNameException","message":"Invalid actor path element [a%4A], illegal character [%] at position: 1. Actor paths MUST: not start with `$`, include only ASCII letters and can only contain these special characters: -_.*$+:@&=,!~';."}]}
```

### name-ok-20

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/a%aF","name":"a%aF","parent":"pekko://sys/user","hasUid":true}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":false,"exception":"org.apache.pekko.actor.InvalidActorNameException","message":"Invalid actor path element [a%aF], illegal character [%] at position: 1. Actor paths MUST: not start with `$`, include only ASCII letters and can only contain these special characters: -_.*$+:@&=,!~';."}]}
```

### name-ok-27

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/a%41%42%43","name":"a%41%42%43","parent":"pekko://sys/user","hasUid":true}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":false,"exception":"org.apache.pekko.actor.InvalidActorNameException","message":"Invalid actor path element [a%41%42%43], illegal character [%] at position: 7. Actor paths MUST: not start with `$`, include only ASCII letters and can only contain these special characters: -_.*$+:@&=,!~';."}]}
```

### name-ok-28

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/%41","name":"%41","parent":"pekko://sys/user","hasUid":true}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":false,"exception":"org.apache.pekko.actor.InvalidActorNameException","message":"Invalid actor path element [%41], illegal character [%] at position: 0. Actor paths MUST: not start with `$`, include only ASCII letters and can only contain these special characters: -_.*$+:@&=,!~';."}]}
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

### pa-0370

Arguments: `path-address-case`

stdout:

```diff
- VAL-11 [%41] -> ok valid=true validate=ok
+ VAL-11 [%41] -> ok valid=false validate=ERR InvalidActorNameException [Invalid actor path element [%41], illegal character [%] at position: 0. Actor paths MUST: not start with `$`, include only ASCII letters and can only contain these special characters: -_.*$+:@&=,!~';.]
```

### pa-0371

Arguments: `path-address-case`

stdout:

```diff
- VAL-12 [A%aF] -> ok valid=true validate=ok
+ VAL-12 [A%aF] -> ok valid=false validate=ERR InvalidActorNameException [Invalid actor path element [A%aF], illegal character [%] at position: 1. Actor paths MUST: not start with `$`, include only ASCII letters and can only contain these special characters: -_.*$+:@&=,!~';.]
```
