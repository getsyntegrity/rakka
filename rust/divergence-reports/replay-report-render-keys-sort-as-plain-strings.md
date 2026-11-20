# App replay

Build: `$TMPDIR/pekko-mutants-1ve7sybd/target-render-keys-sort-as-plain-strings/release/examples/classic-parity`

Passed: 2919/2922

Fallback rate: not reported by this build

## Failing cases

### dp-0051

Arguments: `deploy-case`

stdout:

```diff
- TS-07 toString with keys that need quotes -> ok Deploy(/a, Config(SimpleConfigObject({"1":4,"":2,"a b":1,"x.y":3})), NoRouter, NoScopeGiven, , , Set())
+ TS-07 toString with keys that need quotes -> ok Deploy(/a, Config(SimpleConfigObject({"":2,"a b":1,"x.y":3,"1":4})), NoRouter, NoScopeGiven, , , Set())
```

### dp-0067

Arguments: `deploy-case`

stdout:

```diff
- TS-23 toString with numeric and mixed keys -> ok Deploy(/a, Config(SimpleConfigObject({"2":3,"007":4,"9":2,"10":1,"123456789012345678901234567890":8,"-1":9,"B":6,"_":7,"a":5})), NoRouter, NoScopeGiven, , , Set())
+ TS-23 toString with numeric and mixed keys -> ok Deploy(/a, Config(SimpleConfigObject({"-1":9,"B":6,"_":7,"a":5,"2":3,"007":4,"9":2,"10":1,"123456789012345678901234567890":8})), NoRouter, NoScopeGiven, , , Set())
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
