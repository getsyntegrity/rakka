# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-1f9lsxlk/target-lower-case-ascii-only/release/examples/classic-parity`

Passed: 1038/1047

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

### uc-0059

Arguments: `util-core-case`

stdout:

```diff
- LC \u041F\u0420\u0418\u0412\u0415\u0422 -> ok [\u043F\u0440\u0438\u0432\u0435\u0442]
+ LC \u041F\u0420\u0418\u0412\u0415\u0422 -> ok [\u041F\u0420\u0418\u0412\u0415\u0422]
```

### uc-0060

Arguments: `util-core-case`

stdout:

```diff
- LC \uFF21\uFF22 -> ok [\uFF41\uFF42]
+ LC \uFF21\uFF22 -> ok [\uFF21\uFF22]
```

### uc-0061

Arguments: `util-core-case`

stdout:

```diff
- LC \uD801\uDC00 -> ok [\uD801\uDC28]
+ LC \uD801\uDC00 -> ok [\uD801\uDC00]
```

### uc-0063

Arguments: `util-core-case`

stdout:

```diff
- LC \u0130 -> ok [i\u0307]
+ LC \u0130 -> ok [\u0130]
```

### uc-0064

Arguments: `util-core-case`

stdout:

```diff
- LC \u0391\u03A3 -> ok [\u03B1\u03C2]
+ LC \u0391\u03A3 -> ok [\u0391\u03A3]
```

### uc-0065

Arguments: `util-core-case`

stdout:

```diff
- LC \u0391\u03A3\u0391 -> ok [\u03B1\u03C3\u03B1]
+ LC \u0391\u03A3\u0391 -> ok [\u0391\u03A3\u0391]
```

### uc-0066

Arguments: `util-core-case`

stdout:

```diff
- LC \u03A3 -> ok [\u03C3]
+ LC \u03A3 -> ok [\u03A3]
```

### uc-0083

Arguments: `util-core-case`

stdout:

```diff
- LC \u1C90 -> ok [\u10D0]
+ LC \u1C90 -> ok [\u1C90]
```
