# App replay

Build: `$TMPDIR/pekko-mutants-bzvnb__t/target-concatenation-number-loses-trailing-zeros/release/examples/classic-parity`

Passed: 2942/2945

Fallback rate: not reported by this build

## Failing cases

### dp-0080

Arguments: `deploy-case`

stdout:

```diff
- TS-36 toString with a number with a trailing zero before a substitution -> ok Deploy(/a, Config(SimpleConfigObject({"a":"1.50 "${x}})), NoRouter, NoScopeGiven, , , Set())
+ TS-36 toString with a number with a trailing zero before a substitution -> ok Deploy(/a, Config(SimpleConfigObject({"a":"1.5 "${x}})), NoRouter, NoScopeGiven, , , Set())
```

### dp-0084

Arguments: `deploy-case`

stdout:

```diff
- TS-40 toString with a number after a substitution -> ok Deploy(/a, Config(SimpleConfigObject({"a":${x}" 1.50"})), NoRouter, NoScopeGiven, , , Set())
+ TS-40 toString with a number after a substitution -> ok Deploy(/a, Config(SimpleConfigObject({"a":${x}" 1.5"})), NoRouter, NoScopeGiven, , , Set())
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
