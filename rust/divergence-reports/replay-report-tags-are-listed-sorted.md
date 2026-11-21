# App replay

Build: `$TMPDIR/pekko-mutants-1ve7sybd/target-tags-are-listed-sorted/release/examples/classic-parity`

Passed: 2918/2922

Fallback rate: not reported by this build

## Failing cases

### dp-0058

Arguments: `deploy-case`

stdout:

```diff
- TS-14 toString with four tags in insertion order -> ok Deploy(/a, Config(SimpleConfigObject({})), NoRouter, NoScopeGiven, , , Set(t3, t1, t4, t2))
+ TS-14 toString with four tags in insertion order -> ok Deploy(/a, Config(SimpleConfigObject({})), NoRouter, NoScopeGiven, , , Set(t1, t2, t3, t4))
```

### dp-0059

Arguments: `deploy-case`

stdout:

```diff
- TS-15 toString with four tags in the opposite insertion order -> ok Deploy(/a, Config(SimpleConfigObject({})), NoRouter, NoScopeGiven, , , Set(t2, t4, t1, t3))
+ TS-15 toString with four tags in the opposite insertion order -> ok Deploy(/a, Config(SimpleConfigObject({})), NoRouter, NoScopeGiven, , , Set(t1, t2, t3, t4))
```

### dp-0063

Arguments: `deploy-case`

stdout:

```diff
- TS-19 toString with repeated tags -> ok Deploy(/a, Config(SimpleConfigObject({})), NoRouter, NoScopeGiven, , , Set(b, a, c))
+ TS-19 toString with repeated tags -> ok Deploy(/a, Config(SimpleConfigObject({})), NoRouter, NoScopeGiven, , , Set(a, b, c))
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
