# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-zpsyxi4q/target-long-part-little-endian-shifts-as-long/release/examples/classic-parity`

Passed: 1498/1500

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

### uc-0512

Arguments: `util-core-case`

stdout:

```diff
- IT-07 getLongPart -> ok ok 66051 ok 197121 ok 72623859790382856 ok 201786885 ok 0 1 ok 1 1 ok 0 2 ok 144964032628459529 0 ok 65535
+ IT-07 getLongPart -> ok ok 66051 ok 197121 ok 72623859790382856 ok 578437695752307201 ok 0 1 ok 1 1 ok 0 2 ok 144964032628459529 0 ok 65535
```
