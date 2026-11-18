# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-e7_dglox/target-hash-code-nan-payload-kept/release/examples/classic-parity`

Passed: 1095/1097

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

### uc-0126

Arguments: `util-core-case`

stdout:

```diff
- HC-05 float -> ok 851 -2147482797 1069548371 -1077935277 2143290195 2143290195 2143290195 2139095891 -8387757 852 2139095890 8389459
+ HC-05 float -> ok 851 -2147482797 1069548371 -1077935277 2143290195 2143290196 -4193453 2139095891 -8387757 852 2139095890 8389459
```
