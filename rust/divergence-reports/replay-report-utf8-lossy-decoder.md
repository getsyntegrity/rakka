# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-939l44d_/target-utf8-lossy-decoder/release/examples/classic-parity`

Passed: 1035/1045

Fallback rate: not reported by this build

## Failing cases

### cov41-ui-utf8-3byte-surr

Arguments: `path-from-string`

stdout:

```diff
- {"ok":true,"toString":"pekko://s\ufffd@host:1/user","name":"user","elements":["user"],"uid":0,"toStringWithoutAddress":"/user","toSerializationFormat":"pekko://s\ufffd@host:1/user","address":{"protocol":"pekko","system":"s\ufffd","host":"host","port":1,"toString":"pekko://s\ufffd@host:1"}}
+ {"ok":true,"toString":"pekko://s\ufffd\ufffd\ufffd@host:1/user","name":"user","elements":["user"],"uid":0,"toStringWithoutAddress":"/user","toSerializationFormat":"pekko://s\ufffd\ufffd\ufffd@host:1/user","address":{"protocol":"pekko","system":"s\ufffd\ufffd\ufffd","host":"host","port":1,"toString":"pekko://s\ufffd\ufffd\ufffd@host:1"}}
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

### ok-user-info-surrogate-e

Arguments: `path-from-string`

stdout:

```diff
- {"ok":true,"toString":"pekko://\ufffd@host:2552/user","name":"user","elements":["user"],"uid":0,"toStringWithoutAddress":"/user","toSerializationFormat":"pekko://\ufffd@host:2552/user","address":{"protocol":"pekko","system":"\ufffd","host":"host","port":2552,"toString":"pekko://\ufffd@host:2552"}}
+ {"ok":true,"toString":"pekko://\ufffd\ufffd\ufffd@host:2552/user","name":"user","elements":["user"],"uid":0,"toStringWithoutAddress":"/user","toSerializationFormat":"pekko://\ufffd\ufffd\ufffd@host:2552/user","address":{"protocol":"pekko","system":"\ufffd\ufffd\ufffd","host":"host","port":2552,"toString":"pekko://\ufffd\ufffd\ufffd@host:2552"}}
```

### pa-0177

Arguments: `path-address-case`

stdout:

```diff
- UTF-AD-01 [pekko://a%ED%A0%80b@h:1] -> ok str=[pekko://a\uFFFDb@h:1] protocol=[pekko] system=[a\uFFFDb] host=[h] port=[1] local=false global=true hostPort=[a\uFFFDb@h:1]
+ UTF-AD-01 [pekko://a%ED%A0%80b@h:1] -> ok str=[pekko://a\uFFFD\uFFFD\uFFFDb@h:1] protocol=[pekko] system=[a\uFFFD\uFFFD\uFFFDb] host=[h] port=[1] local=false global=true hostPort=[a\uFFFD\uFFFD\uFFFDb@h:1]
```

### pa-0178

Arguments: `path-address-case`

stdout:

```diff
- UTF-AD-02 [pekko://a%ED%B0%80b@h:1] -> ok str=[pekko://a\uFFFDb@h:1] protocol=[pekko] system=[a\uFFFDb] host=[h] port=[1] local=false global=true hostPort=[a\uFFFDb@h:1]
+ UTF-AD-02 [pekko://a%ED%B0%80b@h:1] -> ok str=[pekko://a\uFFFD\uFFFD\uFFFDb@h:1] protocol=[pekko] system=[a\uFFFD\uFFFD\uFFFDb] host=[h] port=[1] local=false global=true hostPort=[a\uFFFD\uFFFD\uFFFDb@h:1]
```

### pa-0179

Arguments: `path-address-case`

stdout:

```diff
- UTF-AD-03 [pekko://a%ED%A0%80%ED%B0%80b@h:1] -> ok str=[pekko://a\uFFFD\uFFFDb@h:1] protocol=[pekko] system=[a\uFFFD\uFFFDb] host=[h] port=[1] local=false global=true hostPort=[a\uFFFD\uFFFDb@h:1]
+ UTF-AD-03 [pekko://a%ED%A0%80%ED%B0%80b@h:1] -> ok str=[pekko://a\uFFFD\uFFFD\uFFFD\uFFFD\uFFFD\uFFFDb@h:1] protocol=[pekko] system=[a\uFFFD\uFFFD\uFFFD\uFFFD\uFFFD\uFFFDb] host=[h] port=[1] local=false global=true hostPort=[a\uFFFD\uFFFD\uFFFD\uFFFD\uFFFD\uFFFDb@h:1]
```

### pa-0180

Arguments: `path-address-case`

stdout:

```diff
- UTF-AD-04 [pekko://a%ED%A0%41b@h:1] -> ok str=[pekko://a\uFFFDAb@h:1] protocol=[pekko] system=[a\uFFFDAb] host=[h] port=[1] local=false global=true hostPort=[a\uFFFDAb@h:1]
+ UTF-AD-04 [pekko://a%ED%A0%41b@h:1] -> ok str=[pekko://a\uFFFD\uFFFDAb@h:1] protocol=[pekko] system=[a\uFFFD\uFFFDAb] host=[h] port=[1] local=false global=true hostPort=[a\uFFFD\uFFFDAb@h:1]
```

### pa-0181

Arguments: `path-address-case`

stdout:

```diff
- UTF-AD-05 [pekko://a%ED%A0b@h:1] -> ok str=[pekko://a\uFFFDb@h:1] protocol=[pekko] system=[a\uFFFDb] host=[h] port=[1] local=false global=true hostPort=[a\uFFFDb@h:1]
+ UTF-AD-05 [pekko://a%ED%A0b@h:1] -> ok str=[pekko://a\uFFFD\uFFFDb@h:1] protocol=[pekko] system=[a\uFFFD\uFFFDb] host=[h] port=[1] local=false global=true hostPort=[a\uFFFD\uFFFDb@h:1]
```

### pa-0200

Arguments: `path-address-case`

stdout:

```diff
- UTF-AD-24 [pekko://a%E2%82%AC%ED%A0%E2%82%ACb@h:1] -> ok str=[pekko://a\u20AC\uFFFD\u20ACb@h:1] protocol=[pekko] system=[a\u20AC\uFFFD\u20ACb] host=[h] port=[1] local=false global=true hostPort=[a\u20AC\uFFFD\u20ACb@h:1]
+ UTF-AD-24 [pekko://a%E2%82%AC%ED%A0%E2%82%ACb@h:1] -> ok str=[pekko://a\u20AC\uFFFD\uFFFD\u20ACb@h:1] protocol=[pekko] system=[a\u20AC\uFFFD\uFFFD\u20ACb] host=[h] port=[1] local=false global=true hostPort=[a\u20AC\uFFFD\uFFFD\u20ACb@h:1]
```

### pa-0207

Arguments: `path-address-case`

stdout:

```diff
- UTF-FS-01 [pekko://a%ED%A0%80@h:1/u%ED%A0%80/v%C0%80#5] -> ok ChildActorPath str=[pekko://a\uFFFD@h:1/u%ED%A0%80/v%C0%80] ser=[pekko://a\uFFFD@h:1/u%ED%A0%80/v%C0%80#5] noAddr=[/u%ED%A0%80/v%C0%80] addr=[pekko://a\uFFFD@h:1] name=[v%C0%80] uid=5 elems=([u%ED%A0%80],[v%C0%80])
+ UTF-FS-01 [pekko://a%ED%A0%80@h:1/u%ED%A0%80/v%C0%80#5] -> ok ChildActorPath str=[pekko://a\uFFFD\uFFFD\uFFFD@h:1/u%ED%A0%80/v%C0%80] ser=[pekko://a\uFFFD\uFFFD\uFFFD@h:1/u%ED%A0%80/v%C0%80#5] noAddr=[/u%ED%A0%80/v%C0%80] addr=[pekko://a\uFFFD\uFFFD\uFFFD@h:1] name=[v%C0%80] uid=5 elems=([u%ED%A0%80],[v%C0%80])
```
