# App replay

Build: `$TMPDIR/pekko-mutants-bzvnb__t/target-substitution-path-quotes-digit-elements/release/examples/classic-parity`

Passed: 2940/2945

Fallback rate: not reported by this build

## Failing cases

### dp-0094

Arguments: `deploy-case`

stdout:

```diff
- TS-50 toString with substitution paths of digits -> ok Deploy(/a, Config(SimpleConfigObject({"a":${1},"b":${a.007},"c":${a.1.2}})), NoRouter, NoScopeGiven, , , Set())
+ TS-50 toString with substitution paths of digits -> ok Deploy(/a, Config(SimpleConfigObject({"a":${"1"},"b":${a."007"},"c":${a."1"."2"}})), NoRouter, NoScopeGiven, , , Set())
```

### dp-0095

Arguments: `deploy-case`

stdout:

```diff
- TS-51 toString with substitution paths that start with a hyphen or look like keywords -> ok Deploy(/a, Config(SimpleConfigObject({"a":${a.-1}" "${-}" "${-a}" "${1-a}" "${a.1e5}" "${true}" "${null.x}})), NoRouter, NoScopeGiven, , , Set())
+ TS-51 toString with substitution paths that start with a hyphen or look like keywords -> ok Deploy(/a, Config(SimpleConfigObject({"a":${a."-1"}" "${-}" "${-a}" "${"1-a"}" "${a."1e5"}" "${true}" "${null.x}})), NoRouter, NoScopeGiven, , , Set())
```

### dp-0096

Arguments: `deploy-case`

stdout:

```diff
- TS-52 toString with substitution paths that need quotes -> ok Deploy(/a, Config(SimpleConfigObject({"a":${"q k"}" "${""}" "${"\u00E9.x"}" "${1}" "${a-b}" "${_c}" "${"a b".c}})), NoRouter, NoScopeGiven, , , Set())
+ TS-52 toString with substitution paths that need quotes -> ok Deploy(/a, Config(SimpleConfigObject({"a":${"q k"}" "${""}" "${"\u00E9.x"}" "${"1"}" "${a-b}" "${_c}" "${"a b".c}})), NoRouter, NoScopeGiven, , , Set())
```

### dp-0097

Arguments: `deploy-case`

stdout:

```diff
- TS-53 toString with substitution paths of non-ASCII letters -> ok Deploy(/a, Config(SimpleConfigObject({"a":${\u00E9}" "${\u03A91}})), NoRouter, NoScopeGiven, , , Set())
+ TS-53 toString with substitution paths of non-ASCII letters -> ok Deploy(/a, Config(SimpleConfigObject({"a":${\u00E9}" "${"\u03A91"}})), NoRouter, NoScopeGiven, , , Set())
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
