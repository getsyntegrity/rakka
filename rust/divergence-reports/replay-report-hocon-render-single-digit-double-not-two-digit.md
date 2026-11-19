# App replay

Build: `$TMPDIR/pekko-mutants-5pjgu_cj/target-hocon-render-single-digit-double-not-two-digit/release/examples/classic-parity`

Passed: 2256/2258

Fallback rate: not reported by this build

## Failing cases

### ch-0521

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER 4.9e-324 -> ok 4.9E-324
+ NUM-RENDER 4.9e-324 -> ok 5.0E-324
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
