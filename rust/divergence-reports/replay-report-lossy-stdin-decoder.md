# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-c5f9c6on/target-lossy-stdin-decoder/release/examples/classic-parity`

Passed: 1044/1045

Fallback rate: not reported by this build

## Failing cases

### ok-user-info-lone-surrog

Arguments: `path-from-string`

stdout:

```diff
- {"ok":true,"toString":"pekko://sys\ud800@host:2552/user","name":"user","elements":["user"],"uid":0,"toStringWithoutAddress":"/user","toSerializationFormat":"pekko://sys\ud800@host:2552/user","address":{"protocol":"pekko","system":"sys\ud800","host":"host","port":2552,"toString":"pekko://sys\ud800@host:2552"}}
+ {"ok":true,"toString":"pekko://sys\ufffd@host:2552/user","name":"user","elements":["user"],"uid":0,"toStringWithoutAddress":"/user","toSerializationFormat":"pekko://sys\ufffd@host:2552/user","address":{"protocol":"pekko","system":"sys\ufffd","host":"host","port":2552,"toString":"pekko://sys\ufffd@host:2552"}}
```
