# App replay

Build: `$TMPDIR/pekko-mutants-tigrcsc1/target-hocon-duration-nan-is-an-error/release/examples/classic-parity`

Passed: 2479/2484

Fallback rate: not reported by this build

## Failing cases

### ch-0639

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT NS NaN ms -> ok 0
+ DUR-SAT NS NaN ms -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number 'NaN']
```

### ch-0640

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT MS NaN ms -> ok 0
+ DUR-SAT MS NaN ms -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number 'NaN']
```

### ch-0641

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT SEC NaN ms -> ok 0
+ DUR-SAT SEC NaN ms -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number 'NaN']
```

### ch-0642

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT DAY NaN ms -> ok 0
+ DUR-SAT DAY NaN ms -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number 'NaN']
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
