# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-e7_dglox/target-hash-code-long-no-fold/release/examples/classic-parity`

Passed: 1090/1097

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

### uc-0125

Arguments: `util-core-case`

stdout:

```diff
- HC-04 long -> ok 851 -2147483648 852 -2147483647 851 -2147483648 705033556 -1442450943 -2147482797 0 -2147482797 0 852 -2147483647 850 2147483647
+ HC-04 long -> ok 851 -2147483648 852 -2147483647 850 2147483647 705033555 -1442450944 850 2147483647 851 -2147483648 851 -2147483648 850 2147483647
```

### uc-0127

Arguments: `util-core-case`

stdout:

```diff
- HC-06 double -> ok 851 -2147482797 1073218387 -1074265261 2146960211 2146960211 2146960211 2146435923 -1047725 852 -2146434221 1049427
+ HC-06 double -> ok 851 851 851 851 851 851 851 851 851 852 850 851
```

### uc-0129

Arguments: `util-core-case`

stdout:

```diff
- HC-08 arrays of primitives -> ok 23 1166389 100611 1133583 23 31526 1163649 970780212 -527317285 337757915 1166223 -2147473760 0
+ HC-08 arrays of primitives -> ok 23 1166389 100611 1133583 23 31526 1163649 -1176704842 -527317285 1165019 1166223 -2147473760 0
```

### uc-0130

Arguments: `util-core-case`

stdout:

```diff
- HC-09 arrays of objects -> ok 23 892356006 43163537 133073878 1166465 1166465
+ HC-09 arrays of objects -> ok 23 -875018842 43163537 133073878 1166465 1166465
```

### uc-0131

Arguments: `util-core-case`

stdout:

```diff
- HC-10 boxed values -> ok 856 852 705033556 97205 1073218387 948
+ HC-10 boxed values -> ok 856 852 705033555 97205 851 948
```

### uc-0132

Arguments: `util-core-case`

stdout:

```diff
- HC-11 chaining example -> ok 23 -1894357629
+ HC-11 chaining example -> ok 23 1302619523
```
