# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-oxtumr43/target-wall-clock-saturates/release/examples/classic-parity`

Passed: 1185/1187

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

### uc-0216

Arguments: `util-core-case`

stdout:

```diff
- WC-07 the increment wraps around at the top of the Long range -> ok a=9223372036854775806 b=9223372036854775807 c=-9223372036854775808 wrapped=true nextIsWall=true
+ WC-07 the increment wraps around at the top of the Long range -> ok a=9223372036854775806 b=9223372036854775807 c=9223372036854775807 wrapped=false nextIsWall=false
```
