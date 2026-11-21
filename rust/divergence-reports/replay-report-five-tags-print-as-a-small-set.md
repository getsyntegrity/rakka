# App replay

Build: `$TMPDIR/pekko-mutants-1ve7sybd/target-five-tags-print-as-a-small-set/release/examples/classic-parity`

Passed: 2919/2922

Fallback rate: not reported by this build

## Failing cases

### dp-0060

Arguments: `deploy-case`

stdout:

```diff
- TS-16 toString with five tags -> ok Deploy(/a, Config(SimpleConfigObject({})), NoRouter, NoScopeGiven, , , HashSet(t3, t2, t1, t4, t5))
+ TS-16 toString with five tags -> ok Deploy(/a, Config(SimpleConfigObject({})), NoRouter, NoScopeGiven, , , Set(t3, t1, t4, t2, t5))
```

### dp-0061

Arguments: `deploy-case`

stdout:

```diff
- TS-17 toString with five tags in another insertion order -> ok Deploy(/a, Config(SimpleConfigObject({})), NoRouter, NoScopeGiven, , , HashSet(t3, t2, t1, t4, t5))
+ TS-17 toString with five tags in another insertion order -> ok Deploy(/a, Config(SimpleConfigObject({})), NoRouter, NoScopeGiven, , , Set(t5, t2, t4, t1, t3))
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
