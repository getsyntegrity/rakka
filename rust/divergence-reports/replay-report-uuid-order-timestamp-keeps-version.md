# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-_354v1s_/target-uuid-order-timestamp-keeps-version/release/examples/classic-parity`

Passed: 1171/1173

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

### uc-0190

Arguments: `util-core-case`

stdout:

```diff
- UU-02 timestamp of version 1 -> ok [0000000000001000]=0 [0000000100001000]=1 [0000000000011000]=4294967296 [0000000000001001]=281474976710656 [ffffffffffff1fff]=1152921504606846975 [8000000080001800]=576601491939262464 [123456789abc1def]=1004191374476859000 [ffffffffffff1000]=281474976710655 [0000000000001fff]=1152640029630136320
+ UU-02 timestamp of version 1 -> ok [0000000000001000]=1152921504606846976 [0000000100001000]=1152921504606846977 [0000000000011000]=1152921508901814272 [0000000000001001]=1153202979583557632 [ffffffffffff1fff]=2305843009213693951 [8000000080001800]=1729522996546109440 [123456789abc1def]=2157112879083705976 [ffffffffffff1000]=1153202979583557631 [0000000000001fff]=2305561534236983296
```
