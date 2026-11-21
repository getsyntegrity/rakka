# App replay

Build: `$TMPDIR/pekko-mutants-7u16pnpo/target-hash-map-order-puts-child-nodes-before-lone-entries/release/examples/classic-parity`

Passed: 2876/2887

Fallback rate: not reported by this build

## Failing cases

### dr-0069

Arguments: `deployer-case`

stdout:

```diff
- DL-70 5 overlapping prefixes of abczzz in a shuffled order -> ok ok | ok | ok | ok | ok | ok | some(/a/abczz*@d4) | some(/a/abczz*@d4)
+ DL-70 5 overlapping prefixes of abczzz in a shuffled order -> ok ok | ok | ok | ok | ok | ok | some(/a/abczzz*@d6) | some(/a/abczzz*@d6)
```

### dr-0070

Arguments: `deployer-case`

stdout:

```diff
- DL-71 5 overlapping prefixes of abczzz in the reverse order -> ok ok | ok | ok | ok | ok | ok | some(/a/abczz*@d4) | some(/a/abczz*@d4)
+ DL-71 5 overlapping prefixes of abczzz in the reverse order -> ok ok | ok | ok | ok | ok | ok | some(/a/abczzz*@d2) | some(/a/abczzz*@d2)
```

### dr-0087

Arguments: `deployer-case`

stdout:

```diff
- DL-88 5 overlapping prefixes of stzzz in a shuffled order -> ok ok | ok | ok | ok | ok | ok | some(/a/stz*@d5) | some(/a/stz*@d5)
+ DL-88 5 overlapping prefixes of stzzz in a shuffled order -> ok ok | ok | ok | ok | ok | ok | some(/a/st*@d4) | some(/a/st*@d4)
```

### dr-0088

Arguments: `deployer-case`

stdout:

```diff
- DL-89 5 overlapping prefixes of stzzz in the reverse order -> ok ok | ok | ok | ok | ok | ok | some(/a/stz*@d3) | some(/a/stz*@d3)
+ DL-89 5 overlapping prefixes of stzzz in the reverse order -> ok ok | ok | ok | ok | ok | ok | some(/a/st*@d4) | some(/a/st*@d4)
```

### dr-0091

Arguments: `deployer-case`

stdout:

```diff
- DL-92 3 overlapping prefixes of mnxyz in a shuffled order -> ok ok | ok | ok | ok | some(/a/m*@d3) | some(/a/m*@d3)
+ DL-92 3 overlapping prefixes of mnxyz in a shuffled order -> ok ok | ok | ok | ok | some(/a/mnxy*@d2) | some(/a/mnxy*@d2)
```

### dr-0092

Arguments: `deployer-case`

stdout:

```diff
- DL-93 3 overlapping prefixes of mnxyz in the reverse order -> ok ok | ok | ok | ok | some(/a/m*@d3) | some(/a/m*@d3)
+ DL-93 3 overlapping prefixes of mnxyz in the reverse order -> ok ok | ok | ok | ok | some(/a/mnxy*@d4) | some(/a/mnxy*@d4)
```

### dr-0093

Arguments: `deployer-case`

stdout:

```diff
- DL-94 5 overlapping prefixes of mnxyz in a shuffled order -> ok ok | ok | ok | ok | ok | ok | some(/a/mnx*@d5) | some(/a/mnx*@d5)
+ DL-94 5 overlapping prefixes of mnxyz in a shuffled order -> ok ok | ok | ok | ok | ok | ok | some(/a/mnxy*@d2) | some(/a/mnxy*@d2)
```

### dr-0094

Arguments: `deployer-case`

stdout:

```diff
- DL-95 5 overlapping prefixes of mnxyz in the reverse order -> ok ok | ok | ok | ok | ok | ok | some(/a/mnx*@d3) | some(/a/mnx*@d3)
+ DL-95 5 overlapping prefixes of mnxyz in the reverse order -> ok ok | ok | ok | ok | ok | ok | some(/a/mnxy*@d6) | some(/a/mnxy*@d6)
```

### dr-0133

Arguments: `deployer-case`

stdout:

```diff
- DL-134 two prefixes with the same hash code among others, short first -> ok ok | ok | ok | ok | ok | some(/a/bvm*@d2)
+ DL-134 two prefixes with the same hash code among others, short first -> ok ok | ok | ok | ok | ok | some(/a/bvmxiku*@d3)
```

### dr-0134

Arguments: `deployer-case`

stdout:

```diff
- DL-135 two prefixes with the same hash code among others, long first -> ok ok | ok | ok | ok | ok | some(/a/bvm*@d2)
+ DL-135 two prefixes with the same hash code among others, long first -> ok ok | ok | ok | ok | ok | some(/a/bvmxikudrlnvgcneunetqmtswnkyfbflscxdksgeqwbcxgzthmnhmuntxiajllwoxnwfvdg*@d3)
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
