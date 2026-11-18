# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-0g2y5v7p/target-pekko-version-final-accepts-pre-release/release/examples/classic-parity`

Passed: 1108/1111

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

### uc-0136

Arguments: `util-core-case`

stdout:

```diff
- PV-03 spec: unsatisfied -> ok [2.5.6]/[2.5.6-RC1]=UnsupportedPekkoVersion[Current version of Pekko is [2.5.6-RC1], but lib requires version [2.5.6]] [2.5.6-RC2]/[2.5.6-RC1]=UnsupportedPekkoVersion[Current version of Pekko is [2.5.6-RC1], but lib requires version [2.5.6-RC2]] [2.5.6]/[2.5.6-M1]=UnsupportedPekkoVersion[Current version of Pekko is [2.5.6-M1], but lib requires version [2.5.6]] [2.5.7-M10]/[2.5.7-M9]=UnsupportedPekkoVersion[Current version of Pekko is [2.5.7-M9], but lib requires version [2.5.7-M10]] [2.5.7-RC1]/[2.5.7-M9]=UnsupportedPekkoVersion[Current version of Pekko is [2.5.7-M9], but lib requires version [2.5.7-RC1]] [2.5.6]/[3.0.0]=UnsupportedPekkoVersion[Current version of Pekko is [3.0.0], but lib requires version [2.5.6]] [2.5.6]/[1.0.0]=UnsupportedPekkoVersion[Current version of Pekko is [1.0.0], but lib requires version [2.5.6]] [2.5.6]/[2.4.19]=UnsupportedPekkoVersion[Current version of Pekko is [2.4.19], but lib requires version [2.5.6]] [2.5.6]/[2.5.5]=UnsupportedPekkoVersion[Current version of Pekko is [2.5.5], but lib requires version [2.5.6]]
+ PV-03 spec: unsatisfied -> ok [2.5.6]/[2.5.6-RC1]=ok [2.5.6-RC2]/[2.5.6-RC1]=UnsupportedPekkoVersion[Current version of Pekko is [2.5.6-RC1], but lib requires version [2.5.6-RC2]] [2.5.6]/[2.5.6-M1]=ok [2.5.7-M10]/[2.5.7-M9]=UnsupportedPekkoVersion[Current version of Pekko is [2.5.7-M9], but lib requires version [2.5.7-M10]] [2.5.7-RC1]/[2.5.7-M9]=UnsupportedPekkoVersion[Current version of Pekko is [2.5.7-M9], but lib requires version [2.5.7-RC1]] [2.5.6]/[3.0.0]=UnsupportedPekkoVersion[Current version of Pekko is [3.0.0], but lib requires version [2.5.6]] [2.5.6]/[1.0.0]=UnsupportedPekkoVersion[Current version of Pekko is [1.0.0], but lib requires version [2.5.6]] [2.5.6]/[2.4.19]=UnsupportedPekkoVersion[Current version of Pekko is [2.4.19], but lib requires version [2.5.6]] [2.5.6]/[2.5.5]=UnsupportedPekkoVersion[Current version of Pekko is [2.5.5], but lib requires version [2.5.6]]
```

### uc-0139

Arguments: `util-core-case`

stdout:

```diff
- PV-06 pre-release matrix -> ok 1000011111101111001110001
+ PV-06 pre-release matrix -> ok 1111111111101111001110001
```
