# App replay

Build: `$TMPDIR/pekko-mutants-bzvnb__t/target-substitution-path-leaves-astral-elements-bare/release/examples/classic-parity`

Passed: 2943/2945

Fallback rate: not reported by this build

## Failing cases

### dp-0102

Arguments: `deploy-case`

stdout:

```diff
- TS-58 toString with a substitution path of an astral letter -> ok Deploy(/a, Config(SimpleConfigObject({"a":${"\uD835\uDC00"}" "${"\u00E9\uD835\uDC00"}})), NoRouter, NoScopeGiven, , , Set())
+ TS-58 toString with a substitution path of an astral letter -> ok Deploy(/a, Config(SimpleConfigObject({"a":${\uD835\uDC00}" "${\u00E9\uD835\uDC00}})), NoRouter, NoScopeGiven, , , Set())
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
