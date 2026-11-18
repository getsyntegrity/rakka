# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-zgjq93ra/target-glob-question-rejects-surrogate-unit/release/examples/classic-parity`

Passed: 1061/1063

Fallback rate: not reported by this build

## Failing cases

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

### uc-0088

Arguments: `util-core-case`

stdout:

```diff
- GL-U16-01 ? vs astral -> ok [?]/[\uD83D\uDE00]=false [??]/[\uD83D\uDE00]=true [???]/[\uD83D\uDE00]=false [*]/[\uD83D\uDE00]=true [a?b]/[a\uD83D\uDE00b]=false [a??b]/[a\uD83D\uDE00b]=true
+ GL-U16-01 ? vs astral -> ok [?]/[\uD83D\uDE00]=false [??]/[\uD83D\uDE00]=false [???]/[\uD83D\uDE00]=false [*]/[\uD83D\uDE00]=true [a?b]/[a\uD83D\uDE00b]=false [a??b]/[a\uD83D\uDE00b]=false
```
