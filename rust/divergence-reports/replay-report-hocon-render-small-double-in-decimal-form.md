# App replay

Build: `$TMPDIR/pekko-mutants-5pjgu_cj/target-hocon-render-small-double-in-decimal-form/release/examples/classic-parity`

Passed: 2250/2258

Fallback rate: not reported by this build

## Failing cases

### ch-0498

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER 1e-7 -> ok 1.0E-7
+ NUM-RENDER 1e-7 -> ok 0.0000001
```

### ch-0499

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER 12345678.9 -> ok 1.23456789E7
+ NUM-RENDER 12345678.9 -> ok 12345678.9
```

### ch-0517

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER 1e19 -> ok 1.0E19
+ NUM-RENDER 1e19 -> ok 10000000000000000000.0
```

### ch-0518

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER -1e19 -> ok -1.0E19
+ NUM-RENDER -1e19 -> ok -10000000000000000000.0
```

### ch-0524

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER 0.0001 -> ok 1.0E-4
+ NUM-RENDER 0.0001 -> ok 0.0001
```

### ch-0526

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER 12345678.5 -> ok 1.23456785E7
+ NUM-RENDER 12345678.5 -> ok 12345678.5
```

### ch-0527

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER -2.5e-5 -> ok -2.5E-5
+ NUM-RENDER -2.5e-5 -> ok -0.000025
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
