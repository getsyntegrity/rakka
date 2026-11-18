# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-6v3_1z54/target-line-numbers-range-always/release/examples/classic-parity`

Passed: 1195/1198

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

### uc-0224

Arguments: `util-core-case`

stdout:

```diff
- LN-01 one line shows file:line -> ok Foo.scala:22 a/B.java:1
+ LN-01 one line shows file:line -> ok Foo.scala:22-22 a/B.java:1-1
```

### uc-0226

Arguments: `util-core-case`

stdout:

```diff
- LN-03 unordered or non-positive bounds are printed as given -> ok F.scala:9-5 F.scala:0 F.scala:-1-3 :7 F.scala:-2147483648-2147483647
+ LN-03 unordered or non-positive bounds are printed as given -> ok F.scala:9-5 F.scala:0-0 F.scala:-1-3 :7-7 F.scala:-2147483648-2147483647
```
