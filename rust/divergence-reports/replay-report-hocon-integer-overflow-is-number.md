# App replay

Build: `$TMPDIR/pekko-mutants-4l9e16mc/target-hocon-integer-overflow-is-number/release/examples/classic-parity`

Passed: 2299/2304

Fallback rate: not reported by this build

## Failing cases

### ch-0547

Arguments: `config-hocon-case`

stdout:

```diff
- NUMLEX 99999999999999999999 -> ok STRING "99999999999999999999" 99999999999999999999
+ NUMLEX 99999999999999999999 -> ok NUMBER 1.0E20 99999999999999999999
```

### ch-0548

Arguments: `config-hocon-case`

stdout:

```diff
- NUMLEX -99999999999999999999 -> ok STRING "-99999999999999999999" -99999999999999999999
+ NUMLEX -99999999999999999999 -> ok NUMBER -1.0E20 -99999999999999999999
```

### ch-0550

Arguments: `config-hocon-case`

stdout:

```diff
- NUMLEX 9223372036854775808 -> ok STRING "9223372036854775808" 9223372036854775808
+ NUMLEX 9223372036854775808 -> ok NUMBER 9223372036854775807 9223372036854775808
```

### ch-0552

Arguments: `config-hocon-case`

stdout:

```diff
- NUMLEX -9223372036854775809 -> ok STRING "-9223372036854775809" -9223372036854775809
+ NUMLEX -9223372036854775809 -> ok NUMBER -9223372036854775808 -9223372036854775809
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
