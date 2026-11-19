# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-zlxslczp/target-hocon-duration-floors-negative/release/examples/classic-parity`

Passed: 2219/2222

Fallback rate: not reported by this build

## Failing cases

### ch-0080

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-ROUND -1500 us in millis truncates toward zero -> ok -1
+ DUR-ROUND -1500 us in millis truncates toward zero -> ok -2
```

### cs-0048

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDSD log-dead-letters-suspend-duration = -9223372036854775808 d -> ok LogDeadLettersSuspendDuration=-9223372036854ms
+ SET-LDSD log-dead-letters-suspend-duration = -9223372036854775808 d -> ERR BadValue [Invalid value at 'pekko.log-dead-letters-suspend-duration': requirement failed: Duration is limited to +-(2^63-1)ns (ca. 292 years)]
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
