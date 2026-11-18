# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-i2z9eagu/target-utf8-lossy-decoder/release/examples/classic-parity`

Passed: 564/566

Fallback rate: not reported by this build

## Failing cases

### cov41-ui-utf8-3byte-surr

Arguments: `path-from-string`

stdout:

```diff
- {"ok":true,"toString":"pekko://s\ufffd@host:1/user","name":"user","elements":["user"],"uid":0,"toStringWithoutAddress":"/user","toSerializationFormat":"pekko://s\ufffd@host:1/user","address":{"protocol":"pekko","system":"s\ufffd","host":"host","port":1,"toString":"pekko://s\ufffd@host:1"}}
+ {"ok":true,"toString":"pekko://s\ufffd\ufffd\ufffd@host:1/user","name":"user","elements":["user"],"uid":0,"toStringWithoutAddress":"/user","toSerializationFormat":"pekko://s\ufffd\ufffd\ufffd@host:1/user","address":{"protocol":"pekko","system":"s\ufffd\ufffd\ufffd","host":"host","port":1,"toString":"pekko://s\ufffd\ufffd\ufffd@host:1"}}
```

### ok-user-info-surrogate-e

Arguments: `path-from-string`

stdout:

```diff
- {"ok":true,"toString":"pekko://\ufffd@host:2552/user","name":"user","elements":["user"],"uid":0,"toStringWithoutAddress":"/user","toSerializationFormat":"pekko://\ufffd@host:2552/user","address":{"protocol":"pekko","system":"\ufffd","host":"host","port":2552,"toString":"pekko://\ufffd@host:2552"}}
+ {"ok":true,"toString":"pekko://\ufffd\ufffd\ufffd@host:2552/user","name":"user","elements":["user"],"uid":0,"toStringWithoutAddress":"/user","toSerializationFormat":"pekko://\ufffd\ufffd\ufffd@host:2552/user","address":{"protocol":"pekko","system":"\ufffd\ufffd\ufffd","host":"host","port":2552,"toString":"pekko://\ufffd\ufffd\ufffd@host:2552"}}
```
