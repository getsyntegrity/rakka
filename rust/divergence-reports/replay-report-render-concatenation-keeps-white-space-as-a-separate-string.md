# App replay

Build: `$TMPDIR/pekko-mutants-1ve7sybd/target-render-concatenation-keeps-white-space-as-a-separate-string/release/examples/classic-parity`

Passed: 2919/2922

Fallback rate: not reported by this build

## Failing cases

### dp-0056

Arguments: `deploy-case`

stdout:

```diff
- TS-12 toString with an unresolved optional substitution and a concatenation -> ok Deploy(/a, Config(SimpleConfigObject({"a":${?b},"c":${d}" suffix","e":"x "${f}})), NoRouter, NoScopeGiven, , , Set())
+ TS-12 toString with an unresolved optional substitution and a concatenation -> ok Deploy(/a, Config(SimpleConfigObject({"a":${?b},"c":${d}"suffix","e":"x"${f}})), NoRouter, NoScopeGiven, , , Set())
```

### dp-0070

Arguments: `deploy-case`

stdout:

```diff
- TS-26 toString with concatenations of substitutions -> ok Deploy(/a, Config(SimpleConfigObject({"a":${x}" "${y},"b":${x}${y},"c":"a "${x}" b","d":"foo "${x}"bar","e":[${x},1],"f":{"g":${x}}})), NoRouter, NoScopeGiven, , , Set())
+ TS-26 toString with concatenations of substitutions -> ok Deploy(/a, Config(SimpleConfigObject({"a":${x}${y},"b":${x}${y},"c":"a"${x}"b","d":"foo"${x}"bar","e":[${x},1],"f":{"g":${x}}})), NoRouter, NoScopeGiven, , , Set())
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
