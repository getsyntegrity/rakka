# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-939l44d_/target-compare-by-code-point/release/examples/classic-parity`

Passed: 1042/1045

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

### pa-0341

Arguments: `path-address-case`

stdout:

```diff
- CMPA-14 UTF-16 order system U+FF61 vs U+10000 -> ok cmp=1 cmpRev=-1 eq=false
+ CMPA-14 UTF-16 order system U+FF61 vs U+10000 -> ok cmp=-1 cmpRev=1 eq=false
```

### pa-0356

Arguments: `path-address-case`

stdout:

```diff
- CMPP-13 UTF-16 child names U+FF61 vs U+10000 -> ok cmp=1 cmpRev=-1 eq=false eqRev=false
+ CMPP-13 UTF-16 child names U+FF61 vs U+10000 -> ok cmp=-1 cmpRev=1 eq=false eqRev=false
```
