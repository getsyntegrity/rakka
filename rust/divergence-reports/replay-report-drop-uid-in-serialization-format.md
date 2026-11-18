# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-3pr9tpx_/target-drop-uid-in-serialization-format/release/examples/classic-parity`

Passed: 1002/1045

Fallback rate: not reported by this build

## Failing cases

### cov53-uid-plus-sign

Arguments: `path-from-string`

stdout:

```diff
- {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":5,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a#5","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
+ {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":5,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
```

### cov54-uid-minus-sign

Arguments: `path-from-string`

stdout:

```diff
- {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":-5,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a#-5","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
+ {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":-5,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
```

### cov57-uid-min-int

Arguments: `path-from-string`

stdout:

```diff
- {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":-2147483648,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a#-2147483648","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
+ {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":-2147483648,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
```

### ok-fragment-empty-path-e

Arguments: `path-from-string`

stdout:

```diff
- {"ok":true,"toString":"pekko://sys/user/","name":"","elements":["user",""],"uid":5,"toStringWithoutAddress":"/user/","toSerializationFormat":"pekko://sys/user/#5","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
+ {"ok":true,"toString":"pekko://sys/user/","name":"","elements":["user",""],"uid":5,"toStringWithoutAddress":"/user/","toSerializationFormat":"pekko://sys/user/","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
```

### ok-fragment-on-root-slas

Arguments: `path-from-string`

stdout:

```diff
- {"ok":true,"toString":"pekko://sys/","name":"","elements":[""],"uid":5,"toStringWithoutAddress":"/","toSerializationFormat":"pekko://sys/#5","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
+ {"ok":true,"toString":"pekko://sys/","name":"","elements":[""],"uid":5,"toStringWithoutAddress":"/","toSerializationFormat":"pekko://sys/","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
```

### ok-local-uid

Arguments: `path-from-string`

stdout:

```diff
- {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":7,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a#7","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
+ {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":7,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
```

### ok-query-with-fragment

Arguments: `path-from-string`

stdout:

```diff
- {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":9,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a#9","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
+ {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":9,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
```

### ok-remote-uid

Arguments: `path-from-string`

stdout:

```diff
- {"ok":true,"toString":"pekko://sys@host:2552/user/a/b","name":"b","elements":["user","a","b"],"uid":42,"toStringWithoutAddress":"/user/a/b","toSerializationFormat":"pekko://sys@host:2552/user/a/b#42","address":{"protocol":"pekko","system":"sys","host":"host","port":2552,"toString":"pekko://sys@host:2552"}}
+ {"ok":true,"toString":"pekko://sys@host:2552/user/a/b","name":"b","elements":["user","a","b"],"uid":42,"toStringWithoutAddress":"/user/a/b","toSerializationFormat":"pekko://sys@host:2552/user/a/b","address":{"protocol":"pekko","system":"sys","host":"host","port":2552,"toString":"pekko://sys@host:2552"}}
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

### pa-0081

Arguments: `path-address-case`

stdout:

```diff
- FS-16 [pekko://sys/user?x#5] -> ok ChildActorPath str=[pekko://sys/user] ser=[pekko://sys/user#5] noAddr=[/user] addr=[pekko://sys] name=[user] uid=5 elems=([user])
+ FS-16 [pekko://sys/user?x#5] -> ok ChildActorPath str=[pekko://sys/user] ser=[pekko://sys/user] noAddr=[/user] addr=[pekko://sys] name=[user] uid=5 elems=([user])
```

### pa-0101

Arguments: `path-address-case`

stdout:

```diff
- FS-36 [pekko://sys/a#5] -> ok ChildActorPath str=[pekko://sys/a] ser=[pekko://sys/a#5] noAddr=[/a] addr=[pekko://sys] name=[a] uid=5 elems=([a])
+ FS-36 [pekko://sys/a#5] -> ok ChildActorPath str=[pekko://sys/a] ser=[pekko://sys/a] noAddr=[/a] addr=[pekko://sys] name=[a] uid=5 elems=([a])
```

### pa-0102

Arguments: `path-address-case`

stdout:

```diff
- FS-37 [pekko://sys/a#+5] -> ok ChildActorPath str=[pekko://sys/a] ser=[pekko://sys/a#5] noAddr=[/a] addr=[pekko://sys] name=[a] uid=5 elems=([a])
+ FS-37 [pekko://sys/a#+5] -> ok ChildActorPath str=[pekko://sys/a] ser=[pekko://sys/a] noAddr=[/a] addr=[pekko://sys] name=[a] uid=5 elems=([a])
```

### pa-0103

Arguments: `path-address-case`

stdout:

```diff
- FS-38 [pekko://sys/a#-5] -> ok ChildActorPath str=[pekko://sys/a] ser=[pekko://sys/a#-5] noAddr=[/a] addr=[pekko://sys] name=[a] uid=-5 elems=([a])
+ FS-38 [pekko://sys/a#-5] -> ok ChildActorPath str=[pekko://sys/a] ser=[pekko://sys/a] noAddr=[/a] addr=[pekko://sys] name=[a] uid=-5 elems=([a])
```

### pa-0107

Arguments: `path-address-case`

stdout:

```diff
- FS-42 [pekko://sys/a#2147483647] -> ok ChildActorPath str=[pekko://sys/a] ser=[pekko://sys/a#2147483647] noAddr=[/a] addr=[pekko://sys] name=[a] uid=2147483647 elems=([a])
+ FS-42 [pekko://sys/a#2147483647] -> ok ChildActorPath str=[pekko://sys/a] ser=[pekko://sys/a] noAddr=[/a] addr=[pekko://sys] name=[a] uid=2147483647 elems=([a])
```

### pa-0109

Arguments: `path-address-case`

stdout:

```diff
- FS-44 [pekko://sys/a#-2147483648] -> ok ChildActorPath str=[pekko://sys/a] ser=[pekko://sys/a#-2147483648] noAddr=[/a] addr=[pekko://sys] name=[a] uid=-2147483648 elems=([a])
+ FS-44 [pekko://sys/a#-2147483648] -> ok ChildActorPath str=[pekko://sys/a] ser=[pekko://sys/a] noAddr=[/a] addr=[pekko://sys] name=[a] uid=-2147483648 elems=([a])
```

### pa-0114

Arguments: `path-address-case`

stdout:

```diff
- FS-49 [pekko://sys/#5] -> ok ChildActorPath str=[pekko://sys/] ser=[pekko://sys/#5] noAddr=[/] addr=[pekko://sys] name=[] uid=5 elems=([])
+ FS-49 [pekko://sys/#5] -> ok ChildActorPath str=[pekko://sys/] ser=[pekko://sys/] noAddr=[/] addr=[pekko://sys] name=[] uid=5 elems=([])
```

### pa-0115

Arguments: `path-address-case`

stdout:

```diff
- FS-50 [pekko://sys/user/#5] -> ok ChildActorPath str=[pekko://sys/user/] ser=[pekko://sys/user/#5] noAddr=[/user/] addr=[pekko://sys] name=[] uid=5 elems=([user],[])
+ FS-50 [pekko://sys/user/#5] -> ok ChildActorPath str=[pekko://sys/user/] ser=[pekko://sys/user/] noAddr=[/user/] addr=[pekko://sys] name=[] uid=5 elems=([user],[])
```

### pa-0116

Arguments: `path-address-case`

stdout:

```diff
- FS-51 [pekko://sys/a/b#7] -> ok ChildActorPath str=[pekko://sys/a/b] ser=[pekko://sys/a/b#7] noAddr=[/a/b] addr=[pekko://sys] name=[b] uid=7 elems=([a],[b])
+ FS-51 [pekko://sys/a/b#7] -> ok ChildActorPath str=[pekko://sys/a/b] ser=[pekko://sys/a/b] noAddr=[/a/b] addr=[pekko://sys] name=[b] uid=7 elems=([a],[b])
```

### pa-0117

Arguments: `path-address-case`

stdout:

```diff
- FS-52 [pekko://sys/a#\u0665] -> ok ChildActorPath str=[pekko://sys/a] ser=[pekko://sys/a#5] noAddr=[/a] addr=[pekko://sys] name=[a] uid=5 elems=([a])
+ FS-52 [pekko://sys/a#\u0665] -> ok ChildActorPath str=[pekko://sys/a] ser=[pekko://sys/a] noAddr=[/a] addr=[pekko://sys] name=[a] uid=5 elems=([a])
```

### pa-0118

Arguments: `path-address-case`

stdout:

```diff
- FS-53 [pekko://sys/a#\uFF15] -> ok ChildActorPath str=[pekko://sys/a] ser=[pekko://sys/a#5] noAddr=[/a] addr=[pekko://sys] name=[a] uid=5 elems=([a])
+ FS-53 [pekko://sys/a#\uFF15] -> ok ChildActorPath str=[pekko://sys/a] ser=[pekko://sys/a] noAddr=[/a] addr=[pekko://sys] name=[a] uid=5 elems=([a])
```

### pa-0207

Arguments: `path-address-case`

stdout:

```diff
- UTF-FS-01 [pekko://a%ED%A0%80@h:1/u%ED%A0%80/v%C0%80#5] -> ok ChildActorPath str=[pekko://a\uFFFD@h:1/u%ED%A0%80/v%C0%80] ser=[pekko://a\uFFFD@h:1/u%ED%A0%80/v%C0%80#5] noAddr=[/u%ED%A0%80/v%C0%80] addr=[pekko://a\uFFFD@h:1] name=[v%C0%80] uid=5 elems=([u%ED%A0%80],[v%C0%80])
+ UTF-FS-01 [pekko://a%ED%A0%80@h:1/u%ED%A0%80/v%C0%80#5] -> ok ChildActorPath str=[pekko://a\uFFFD@h:1/u%ED%A0%80/v%C0%80] ser=[pekko://a\uFFFD@h:1/u%ED%A0%80/v%C0%80] noAddr=[/u%ED%A0%80/v%C0%80] addr=[pekko://a\uFFFD@h:1] name=[v%C0%80] uid=5 elems=([u%ED%A0%80],[v%C0%80])
```

### pa-0271

Arguments: `path-address-case`

stdout:

```diff
- CH [foo#7] -> ok ChildActorPath str=[pekko://mysys/user/foo] ser=[pekko://mysys/user/foo#7] noAddr=[/user/foo] addr=[pekko://mysys] name=[foo] uid=7 elems=([user],[foo])
+ CH [foo#7] -> ok ChildActorPath str=[pekko://mysys/user/foo] ser=[pekko://mysys/user/foo] noAddr=[/user/foo] addr=[pekko://mysys] name=[foo] uid=7 elems=([user],[foo])
```

### pa-0272

Arguments: `path-address-case`

stdout:

```diff
- CH [foo#+7] -> ok ChildActorPath str=[pekko://mysys/user/foo] ser=[pekko://mysys/user/foo#7] noAddr=[/user/foo] addr=[pekko://mysys] name=[foo] uid=7 elems=([user],[foo])
+ CH [foo#+7] -> ok ChildActorPath str=[pekko://mysys/user/foo] ser=[pekko://mysys/user/foo] noAddr=[/user/foo] addr=[pekko://mysys] name=[foo] uid=7 elems=([user],[foo])
```

### pa-0273

Arguments: `path-address-case`

stdout:

```diff
- CH [foo#-7] -> ok ChildActorPath str=[pekko://mysys/user/foo] ser=[pekko://mysys/user/foo#-7] noAddr=[/user/foo] addr=[pekko://mysys] name=[foo] uid=-7 elems=([user],[foo])
+ CH [foo#-7] -> ok ChildActorPath str=[pekko://mysys/user/foo] ser=[pekko://mysys/user/foo] noAddr=[/user/foo] addr=[pekko://mysys] name=[foo] uid=-7 elems=([user],[foo])
```

### pa-0279

Arguments: `path-address-case`

stdout:

```diff
- CH [#5] -> ok ChildActorPath str=[pekko://mysys/user/] ser=[pekko://mysys/user/#5] noAddr=[/user/] addr=[pekko://mysys] name=[] uid=5 elems=([user],[])
+ CH [#5] -> ok ChildActorPath str=[pekko://mysys/user/] ser=[pekko://mysys/user/] noAddr=[/user/] addr=[pekko://mysys] name=[] uid=5 elems=([user],[])
```

### pa-0281

Arguments: `path-address-case`

stdout:

```diff
- CH [foo#\u0665] -> ok ChildActorPath str=[pekko://mysys/user/foo] ser=[pekko://mysys/user/foo#5] noAddr=[/user/foo] addr=[pekko://mysys] name=[foo] uid=5 elems=([user],[foo])
+ CH [foo#\u0665] -> ok ChildActorPath str=[pekko://mysys/user/foo] ser=[pekko://mysys/user/foo] noAddr=[/user/foo] addr=[pekko://mysys] name=[foo] uid=5 elems=([user],[foo])
```

### pa-0292

Arguments: `path-address-case`

stdout:

```diff
- CH root/'foo#7' -> ok ChildActorPath str=[pekko://mysys/foo] ser=[pekko://mysys/foo#7] noAddr=[/foo] addr=[pekko://mysys] name=[foo] uid=7 elems=([foo])
+ CH root/'foo#7' -> ok ChildActorPath str=[pekko://mysys/foo] ser=[pekko://mysys/foo] noAddr=[/foo] addr=[pekko://mysys] name=[foo] uid=7 elems=([foo])
```

### pa-0294

Arguments: `path-address-case`

stdout:

```diff
- CH java child('x#3') -> ok ChildActorPath str=[pekko://mysys/x] ser=[pekko://mysys/x#3] noAddr=[/x] addr=[pekko://mysys] name=[x] uid=3 elems=([x])
+ CH java child('x#3') -> ok ChildActorPath str=[pekko://mysys/x] ser=[pekko://mysys/x] noAddr=[/x] addr=[pekko://mysys] name=[x] uid=3 elems=([x])
```

### pa-0298

Arguments: `path-address-case`

stdout:

```diff
- DS-04 [a,b#5] -> ok ChildActorPath str=[pekko://mysys/a/b] ser=[pekko://mysys/a/b#5] noAddr=[/a/b] addr=[pekko://mysys] name=[b] uid=5 elems=([a],[b])
+ DS-04 [a,b#5] -> ok ChildActorPath str=[pekko://mysys/a/b] ser=[pekko://mysys/a/b] noAddr=[/a/b] addr=[pekko://mysys] name=[b] uid=5 elems=([a],[b])
```

### pa-0305

Arguments: `path-address-case`

stdout:

```diff
- UID-04 child.withUid(9) -> ok ChildActorPath str=[pekko://mysys/a] ser=[pekko://mysys/a#9] noAddr=[/a] addr=[pekko://mysys] name=[a] uid=9 elems=([a])
+ UID-04 child.withUid(9) -> ok ChildActorPath str=[pekko://mysys/a] ser=[pekko://mysys/a] noAddr=[/a] addr=[pekko://mysys] name=[a] uid=9 elems=([a])
```

### pa-0307

Arguments: `path-address-case`

stdout:

```diff
- UID-07 child.withUid(MIN) -> ok ChildActorPath str=[pekko://mysys/a] ser=[pekko://mysys/a#-2147483648] noAddr=[/a] addr=[pekko://mysys] name=[a] uid=-2147483648 elems=([a])
+ UID-07 child.withUid(MIN) -> ok ChildActorPath str=[pekko://mysys/a] ser=[pekko://mysys/a] noAddr=[/a] addr=[pekko://mysys] name=[a] uid=-2147483648 elems=([a])
```

### pa-0310

Arguments: `path-address-case`

stdout:

```diff
- STR-01 child#7 -> ok ChildActorPath str=[pekko://mysys/user/foo] ser=[pekko://mysys/user/foo#7] noAddr=[/user/foo] addr=[pekko://mysys] name=[foo] uid=7 elems=([user],[foo])
+ STR-01 child#7 -> ok ChildActorPath str=[pekko://mysys/user/foo] ser=[pekko://mysys/user/foo] noAddr=[/user/foo] addr=[pekko://mysys] name=[foo] uid=7 elems=([user],[foo])
```

### pa-0312

Arguments: `path-address-case`

stdout:

```diff
- STR-03 child#7 serWithAddr(remote) -> ok pekko://mysys@aaa:7355/user/foo#7
+ STR-03 child#7 serWithAddr(remote) -> ok pekko://mysys@aaa:7355/user/foo
```

### pa-0314

Arguments: `path-address-case`

stdout:

```diff
- STR-05 child#7 serWithAddr(local other) -> ok pekko://other/user/foo#7
+ STR-05 child#7 serWithAddr(local other) -> ok pekko://other/user/foo
```

### pa-0315

Arguments: `path-address-case`

stdout:

```diff
- STR-06 remote child#7 serWithAddr(other remote) -> ok pekko://mysys@aaa:7355/foo#7
+ STR-06 remote child#7 serWithAddr(other remote) -> ok pekko://mysys@aaa:7355/foo
```

### pa-0319

Arguments: `path-address-case`

stdout:

```diff
- STR-10 child#-3 ser -> ok pekko://mysys/a#-3
+ STR-10 child#-3 ser -> ok pekko://mysys/a
```

### uid-arabic-indic-digits

Arguments: `path-from-string`

stdout:

```diff
- {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":34,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a#34","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
+ {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":34,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
```

### uid-fullwidth-digits

Arguments: `path-from-string`

stdout:

```diff
- {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":12,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a#12","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
+ {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":12,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
```

### uid-int-max

Arguments: `path-from-string`

stdout:

```diff
- {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":2147483647,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a#2147483647","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
+ {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":2147483647,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
```

### uid-int-min

Arguments: `path-from-string`

stdout:

```diff
- {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":-2147483648,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a#-2147483648","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
+ {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":-2147483648,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
```

### uid-leading-zeros

Arguments: `path-from-string`

stdout:

```diff
- {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":7,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a#7","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
+ {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":7,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
```

### uid-negative

Arguments: `path-from-string`

stdout:

```diff
- {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":-1,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a#-1","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
+ {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":-1,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
```

### uid-plus-sign

Arguments: `path-from-string`

stdout:

```diff
- {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":1,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a#1","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
+ {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":1,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
```
