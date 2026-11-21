# App replay

Build: `$TMPDIR/pekko-mutants-1ve7sybd/target-render-shows-only-the-top-layer-of-a-key/release/examples/classic-parity`

Passed: 2914/2922

Fallback rate: not reported by this build

## Failing cases

### dp-0071

Arguments: `deploy-case`

stdout:

```diff
- TS-27 toString with a substitution after and before an object -> ok Deploy(/a, Config(SimpleConfigObject({"a":{"x":1},"a":${b},"c":${d},"c":{"y":2}})), NoRouter, NoScopeGiven, , , Set())
+ TS-27 toString with a substitution after and before an object -> ok Deploy(/a, Config(SimpleConfigObject({"a":${b},"c":{"y":2}})), NoRouter, NoScopeGiven, , , Set())
```

### dp-0073

Arguments: `deploy-case`

stdout:

```diff
- TS-29 toString with a substitution under an object -> ok Deploy(/a, Config(SimpleConfigObject({"a":${b},"a":{"y":2}})), NoRouter, NoScopeGiven, , , Set())
+ TS-29 toString with a substitution under an object -> ok Deploy(/a, Config(SimpleConfigObject({"a":{"y":2}})), NoRouter, NoScopeGiven, , , Set())
```

### dp-0074

Arguments: `deploy-case`

stdout:

```diff
- TS-30 toString with a substitution over a number -> ok Deploy(/a, Config(SimpleConfigObject({"a":1,"a":${b}})), NoRouter, NoScopeGiven, , , Set())
+ TS-30 toString with a substitution over a number -> ok Deploy(/a, Config(SimpleConfigObject({"a":${b}})), NoRouter, NoScopeGiven, , , Set())
```

### dp-0076

Arguments: `deploy-case`

stdout:

```diff
- TS-32 toString with objects around a substitution -> ok Deploy(/a, Config(SimpleConfigObject({"a":{"x":1},"a":${b},"a":{"z":3}})), NoRouter, NoScopeGiven, , , Set())
+ TS-32 toString with objects around a substitution -> ok Deploy(/a, Config(SimpleConfigObject({"a":{"z":3}})), NoRouter, NoScopeGiven, , , Set())
```

### dp-0077

Arguments: `deploy-case`

stdout:

```diff
- TS-33 toString with two substitutions -> ok Deploy(/a, Config(SimpleConfigObject({"a":${b},"a":${c}})), NoRouter, NoScopeGiven, , , Set())
+ TS-33 toString with two substitutions -> ok Deploy(/a, Config(SimpleConfigObject({"a":${c}})), NoRouter, NoScopeGiven, , , Set())
```

### dp-0078

Arguments: `deploy-case`

stdout:

```diff
- TS-34 toString with a substitution and a nested key -> ok Deploy(/a, Config(SimpleConfigObject({"a":{"b":${c},"b":{"d":1},"e":2}})), NoRouter, NoScopeGiven, , , Set())
+ TS-34 toString with a substitution and a nested key -> ok Deploy(/a, Config(SimpleConfigObject({"a":{"b":{"d":1},"e":2}})), NoRouter, NoScopeGiven, , , Set())
```

### dp-0079

Arguments: `deploy-case`

stdout:

```diff
- TS-35 toString with an optional substitution over an object -> ok Deploy(/a, Config(SimpleConfigObject({"a":{"x":1},"a":${?b}})), NoRouter, NoScopeGiven, , , Set())
+ TS-35 toString with an optional substitution over an object -> ok Deploy(/a, Config(SimpleConfigObject({"a":${?b}})), NoRouter, NoScopeGiven, , , Set())
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
