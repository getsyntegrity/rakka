# App replay

Build: `$TMPDIR/pekko-mutants-bzvnb__t/target-concatenation-fuses-objects-in-the-wrong-order/release/examples/classic-parity`

Passed: 2943/2945

Fallback rate: not reported by this build

## Failing cases

### dp-0092

Arguments: `deploy-case`

stdout:

```diff
- TS-48 toString with adjacent objects that repeat a key -> ok Deploy(/a, Config(SimpleConfigObject({"a":{"x":2}${z}})), NoRouter, NoScopeGiven, , , Set())
+ TS-48 toString with adjacent objects that repeat a key -> ok Deploy(/a, Config(SimpleConfigObject({"a":{"x":1}${z}})), NoRouter, NoScopeGiven, , , Set())
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
