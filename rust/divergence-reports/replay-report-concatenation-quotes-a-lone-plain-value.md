# App replay

Build: `$TMPDIR/pekko-mutants-bzvnb__t/target-concatenation-quotes-a-lone-plain-value/release/examples/classic-parity`

Passed: 2941/2945

Fallback rate: not reported by this build

## Failing cases

### dp-0086

Arguments: `deploy-case`

stdout:

```diff
- TS-42 toString with a number with leading zeros between substitutions -> ok Deploy(/a, Config(SimpleConfigObject({"a":${x}7${y}})), NoRouter, NoScopeGiven, , , Set())
+ TS-42 toString with a number with leading zeros between substitutions -> ok Deploy(/a, Config(SimpleConfigObject({"a":${x}"007"${y}})), NoRouter, NoScopeGiven, , , Set())
```

### dp-0099

Arguments: `deploy-case`

stdout:

```diff
- TS-55 toString with a lone number between substitutions -> ok Deploy(/a, Config(SimpleConfigObject({"a":${x}1${y}})), NoRouter, NoScopeGiven, , , Set())
+ TS-55 toString with a lone number between substitutions -> ok Deploy(/a, Config(SimpleConfigObject({"a":${x}"1"${y}})), NoRouter, NoScopeGiven, , , Set())
```

### dp-0100

Arguments: `deploy-case`

stdout:

```diff
- TS-56 toString with a lone boolean between substitutions -> ok Deploy(/a, Config(SimpleConfigObject({"a":${x}true${y}})), NoRouter, NoScopeGiven, , , Set())
+ TS-56 toString with a lone boolean between substitutions -> ok Deploy(/a, Config(SimpleConfigObject({"a":${x}"true"${y}})), NoRouter, NoScopeGiven, , , Set())
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
