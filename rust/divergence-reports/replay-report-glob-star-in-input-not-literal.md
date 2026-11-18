# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-zgjq93ra/target-glob-star-in-input-not-literal/release/examples/classic-parity`

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

### uc-0099

Arguments: `util-core-case`

stdout:

```diff
- GL-05 wildcards in the input -> ok [*]/[*]=true [?]/[?]=true [*]/[?]=true [a*]/[a*]=true [a*b]/[a*b]=true [a*b]/[a*]=false [*a]/[**a]=false [?*]/[**]=true [*?]/[?*]=true [**]/[*]=true
+ GL-05 wildcards in the input -> ok [*]/[*]=true [?]/[?]=true [*]/[?]=true [a*]/[a*]=true [a*b]/[a*b]=true [a*b]/[a*]=false [*a]/[**a]=true [?*]/[**]=true [*?]/[?*]=true [**]/[*]=true
```
