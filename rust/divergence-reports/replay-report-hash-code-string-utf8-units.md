# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-e7_dglox/target-hash-code-string-utf8-units/release/examples/classic-parity`

Passed: 1094/1097

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

### uc-0128

Arguments: `util-core-case`

stdout:

```diff
- HC-07 string -> ok []=851 [a]=948 [abc]=97205 [Hello, World!]=1498790760 [\u00E9]=1084 [\u20AC]=9215 [\uD83D\uDE00]=1773750 [a\uD83D\uDE00b]=57850545 [1000x]=-1715417261
+ HC-07 string -> ok []=851 [a]=948 [abc]=97205 [Hello, World!]=1498790760 [\u00E9]=7065 [\u20AC]=222239 [\uD83D\uDE00]=7308330 [a\uD83D\uDE00b]=-1291406851 [1000x]=-1715417261
```

### uc-0130

Arguments: `util-core-case`

stdout:

```diff
- HC-09 arrays of objects -> ok 23 892356006 43163537 133073878 1166465 1166465
+ HC-09 arrays of objects -> ok 23 892356006 43163537 133079859 1166465 1166465
```
