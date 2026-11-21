# App replay

Build: `$TMPDIR/pekko-mutants-7u16pnpo/target-hash-map-order-reverses-colliding-keys/release/examples/classic-parity`

Passed: 2884/2887

Fallback rate: not reported by this build

## Failing cases

### dr-0131

Arguments: `deployer-case`

stdout:

```diff
- DL-132 two prefixes with the same hash code, short first -> ok ok | ok | ok | some(/a/bvmxiku*@d2)
+ DL-132 two prefixes with the same hash code, short first -> ok ok | ok | ok | some(/a/bvmxikudrlnvgcneunetqmtswnkyfbflscxdksgeqwbcxgzthmnhmuntxiajllwoxnwfvdg*@d3)
```

### dr-0132

Arguments: `deployer-case`

stdout:

```diff
- DL-133 two prefixes with the same hash code, long first -> ok ok | ok | ok | some(/a/bvmxikudrlnvgcneunetqmtswnkyfbflscxdksgeqwbcxgzthmnhmuntxiajllwoxnwfvdg*@d2)
+ DL-133 two prefixes with the same hash code, long first -> ok ok | ok | ok | some(/a/bvmxiku*@d3)
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
