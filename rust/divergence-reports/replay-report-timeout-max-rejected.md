# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-1f9lsxlk/target-timeout-max-rejected/release/examples/classic-parity`

Passed: 1045/1047

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

### uc-0003

Arguments: `util-core-case`

stdout:

```diff
- TO-MAX NANOSECONDS -> ok length=9223372036854775807 nanos=9223372036854775807
+ TO-MAX NANOSECONDS -> ERR IllegalArgumentException [requirement failed: Duration is limited to +-(2^63-1)ns (ca. 292 years)]
```
