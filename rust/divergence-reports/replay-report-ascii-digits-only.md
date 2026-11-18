# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-939l44d_/target-ascii-digits-only/release/examples/classic-parity`

Passed: 1039/1045

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

### pa-0117

Arguments: `path-address-case`

stdout:

```diff
- FS-52 [pekko://sys/a#\u0665] -> ok ChildActorPath str=[pekko://sys/a] ser=[pekko://sys/a#5] noAddr=[/a] addr=[pekko://sys] name=[a] uid=5 elems=([a])
+ FS-52 [pekko://sys/a#\u0665] -> ERR NumberFormatException [For input string: "\u0665"]
```

### pa-0118

Arguments: `path-address-case`

stdout:

```diff
- FS-53 [pekko://sys/a#\uFF15] -> ok ChildActorPath str=[pekko://sys/a] ser=[pekko://sys/a#5] noAddr=[/a] addr=[pekko://sys] name=[a] uid=5 elems=([a])
+ FS-53 [pekko://sys/a#\uFF15] -> ERR NumberFormatException [For input string: "\uFF15"]
```

### pa-0281

Arguments: `path-address-case`

stdout:

```diff
- CH [foo#\u0665] -> ok ChildActorPath str=[pekko://mysys/user/foo] ser=[pekko://mysys/user/foo#5] noAddr=[/user/foo] addr=[pekko://mysys] name=[foo] uid=5 elems=([user],[foo])
+ CH [foo#\u0665] -> ERR NumberFormatException [For input string: "\u0665"]
```

### uid-arabic-indic-digits

Arguments: `path-from-string`

stdout:

```diff
- {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":34,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a#34","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
+ {"ok":false,"exception":"java.lang.NumberFormatException","message":"For input string: \"\u0663\u0664\""}
```

### uid-fullwidth-digits

Arguments: `path-from-string`

stdout:

```diff
- {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":12,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a#12","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
+ {"ok":false,"exception":"java.lang.NumberFormatException","message":"For input string: \"\uff11\uff12\""}
```
