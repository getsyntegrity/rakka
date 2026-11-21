# App replay

Build: `$TMPDIR/pekko-mutants-1ve7sybd/target-surrogate-escape-pair-becomes-two-replacement-characters/release/examples/classic-parity`

Passed: 2920/2922

Fallback rate: not reported by this build

## Failing cases

### dp-0069

Arguments: `deploy-case`

stdout:

```diff
- TS-25 toString with keys that differ in UTF-16 order and code point order -> ok Deploy(/a, Config(SimpleConfigObject({"\uD83D\uDE00":2,"\uE000":3,"\uFFFF":1})), NoRouter, NoScopeGiven, , , Set())
+ TS-25 toString with keys that differ in UTF-16 order and code point order -> ok Deploy(/a, Config(SimpleConfigObject({"\uE000":3,"\uFFFD\uFFFD":2,"\uFFFF":1})), NoRouter, NoScopeGiven, , , Set())
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
