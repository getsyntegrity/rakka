# App replay

Build: `$TMPDIR/pekko-mutants-bzvnb__t/target-concatenation-keeps-white-space-next-to-containers/release/examples/classic-parity`

Passed: 2937/2945

Fallback rate: not reported by this build

## Failing cases

### dp-0087

Arguments: `deploy-case`

stdout:

```diff
- TS-43 toString with adjacent lists before a substitution -> ok Deploy(/a, Config(SimpleConfigObject({"a":[1,2]${x}})), NoRouter, NoScopeGiven, , , Set())
+ TS-43 toString with adjacent lists before a substitution -> ok Deploy(/a, Config(SimpleConfigObject({"a":[1]" "[2]" "${x}})), NoRouter, NoScopeGiven, , , Set())
```

### dp-0088

Arguments: `deploy-case`

stdout:

```diff
- TS-44 toString with adjacent objects before a substitution -> ok Deploy(/a, Config(SimpleConfigObject({"a":{"x":1,"y":2}${z}})), NoRouter, NoScopeGiven, , , Set())
+ TS-44 toString with adjacent objects before a substitution -> ok Deploy(/a, Config(SimpleConfigObject({"a":{"x":1}" "{"y":2}" "${z}})), NoRouter, NoScopeGiven, , , Set())
```

### dp-0089

Arguments: `deploy-case`

stdout:

```diff
- TS-45 toString with lists around a substitution -> ok Deploy(/a, Config(SimpleConfigObject({"a":[1]${x}[2]})), NoRouter, NoScopeGiven, , , Set())
+ TS-45 toString with lists around a substitution -> ok Deploy(/a, Config(SimpleConfigObject({"a":[1]" "${x}" "[2]})), NoRouter, NoScopeGiven, , , Set())
```

### dp-0090

Arguments: `deploy-case`

stdout:

```diff
- TS-46 toString with adjacent objects after a substitution -> ok Deploy(/a, Config(SimpleConfigObject({"a":${x}{"b":1,"c":2}})), NoRouter, NoScopeGiven, , , Set())
+ TS-46 toString with adjacent objects after a substitution -> ok Deploy(/a, Config(SimpleConfigObject({"a":${x}" "{"b":1}" "{"c":2}})), NoRouter, NoScopeGiven, , , Set())
```

### dp-0091

Arguments: `deploy-case`

stdout:

```diff
- TS-47 toString with adjacent lists after a substitution -> ok Deploy(/a, Config(SimpleConfigObject({"a":${x}[1,2]})), NoRouter, NoScopeGiven, , , Set())
+ TS-47 toString with adjacent lists after a substitution -> ok Deploy(/a, Config(SimpleConfigObject({"a":${x}" "[1]" "[2]})), NoRouter, NoScopeGiven, , , Set())
```

### dp-0092

Arguments: `deploy-case`

stdout:

```diff
- TS-48 toString with adjacent objects that repeat a key -> ok Deploy(/a, Config(SimpleConfigObject({"a":{"x":2}${z}})), NoRouter, NoScopeGiven, , , Set())
+ TS-48 toString with adjacent objects that repeat a key -> ok Deploy(/a, Config(SimpleConfigObject({"a":{"x":1}" "{"x":2}" "${z}})), NoRouter, NoScopeGiven, , , Set())
```

### dp-0093

Arguments: `deploy-case`

stdout:

```diff
- TS-49 toString with adjacent objects that merge a nested key -> ok Deploy(/a, Config(SimpleConfigObject({"a":{"x":{"j":2,"k":1}}${z}})), NoRouter, NoScopeGiven, , , Set())
+ TS-49 toString with adjacent objects that merge a nested key -> ok Deploy(/a, Config(SimpleConfigObject({"a":{"x":{"k":1}}" "{"x":{"j":2}}" "${z}})), NoRouter, NoScopeGiven, , , Set())
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
