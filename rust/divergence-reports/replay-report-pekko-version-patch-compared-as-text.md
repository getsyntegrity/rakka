# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-0g2y5v7p/target-pekko-version-patch-compared-as-text/release/examples/classic-parity`

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

### uc-0140

Arguments: `util-core-case`

stdout:

```diff
- PV-07 numbers compared as text then as ints -> ok [2.5.6]/[2.05.6]=ok [2.05.6]/[2.5.6]=ok [2.5.6]/[2.5.06]=ok [2.5.06]/[2.5.6]=ok [02.5.6]/[2.5.6]=UnsupportedPekkoVersion[Current version of Pekko is [2.5.6], but lib requires version [02.5.6]] [2.5.6]/[02.5.6]=UnsupportedPekkoVersion[Current version of Pekko is [02.5.6], but lib requires version [2.5.6]] [2.6.0]/[2.05.9]=UnsupportedPekkoVersion[Current version of Pekko is [2.05.9], but lib requires version [2.6.0]] [2.05.9]/[2.5.8]=ok [2.5.007]/[2.5.7]=ok [2.5.7]/[2.5.0007]=ok [2.5.7]/[2.5.007-M1]=ok [2.5.7-M01]/[2.5.7-M1]=ok [2.5.7-M2]/[2.5.7-M02]=ok [2.5.7-M10]/[2.5.7-M9]=UnsupportedPekkoVersion[Current version of Pekko is [2.5.7-M9], but lib requires version [2.5.7-M10]] [2.5.7-M9]/[2.5.7-M10]=ok [2.5.2147483647]/[2.5.2147483646]=UnsupportedPekkoVersion[Current version of Pekko is [2.5.2147483646], but lib requires version [2.5.2147483647]] [2.5.2147483647]/[2.5.2147483647]=ok
+ PV-07 numbers compared as text then as ints -> ok [2.5.6]/[2.05.6]=ok [2.05.6]/[2.5.6]=ok [2.5.6]/[2.5.06]=UnsupportedPekkoVersion[Current version of Pekko is [2.5.06], but lib requires version [2.5.6]] [2.5.06]/[2.5.6]=ok [02.5.6]/[2.5.6]=UnsupportedPekkoVersion[Current version of Pekko is [2.5.6], but lib requires version [02.5.6]] [2.5.6]/[02.5.6]=UnsupportedPekkoVersion[Current version of Pekko is [02.5.6], but lib requires version [2.5.6]] [2.6.0]/[2.05.9]=UnsupportedPekkoVersion[Current version of Pekko is [2.05.9], but lib requires version [2.6.0]] [2.05.9]/[2.5.8]=ok [2.5.007]/[2.5.7]=ok [2.5.7]/[2.5.0007]=UnsupportedPekkoVersion[Current version of Pekko is [2.5.0007], but lib requires version [2.5.7]] [2.5.7]/[2.5.007-M1]=UnsupportedPekkoVersion[Current version of Pekko is [2.5.007-M1], but lib requires version [2.5.7]] [2.5.7-M01]/[2.5.7-M1]=ok [2.5.7-M2]/[2.5.7-M02]=ok [2.5.7-M10]/[2.5.7-M9]=UnsupportedPekkoVersion[Current version of Pekko is [2.5.7-M9], but lib requires version [2.5.7-M10]] [2.5.7-M9]/[2.5.7-M10]=ok [2.5.2147483647]/[2.5.2147483646]=UnsupportedPekkoVersion[Current version of Pekko is [2.5.2147483646], but lib requires version [2.5.2147483647]] [2.5.2147483647]/[2.5.2147483647]=ok
```

### uc-0141

Arguments: `util-core-case`

stdout:

```diff
- PV-08 numbers out of int range -> ok [2.5.6]/[2.5.99999999999]=NumberFormatException[For input string: "99999999999"] [2.5.99999999999]/[2.5.6]=NumberFormatException[For input string: "99999999999"] [2.99999999999.0]/[2.5.0]=NumberFormatException[For input string: "99999999999"] [2.5.0]/[2.99999999999.0]=NumberFormatException[For input string: "99999999999"] [99999999999.5.6]/[99999999999.5.7]=ok [99999999999.5.6]/[99999999998.5.7]=UnsupportedPekkoVersion[Current version of Pekko is [99999999998.5.7], but lib requires version [99999999999.5.6]] [2.5.99999999999]/[2.5.99999999999]=ok [2.5.99999999999]/[2.5.99999999998]=NumberFormatException[For input string: "99999999999"] [2.5.6-M99999999999]/[2.5.6-M1]=NumberFormatException[For input string: "99999999999"] [2.5.6-M1]/[2.5.6-M99999999999]=NumberFormatException[For input string: "99999999999"] [2.5.6-RC99999999999]/[2.5.6-RC99999999999]=ok [2.5.6-RC99999999999]/[2.5.6-RC99999999998]=NumberFormatException[For input string: "99999999999"] [2.5.2147483648]/[2.5.1]=NumberFormatException[For input string: "2147483648"] [2.5.1]/[2.5.2147483648]=NumberFormatException[For input string: "2147483648"] [2.2147483648.1]/[2.1.1]=NumberFormatException[For input string: "2147483648"] [2.5.6-M2147483648]/[2.5.6-M1]=NumberFormatException[For input string: "2147483648"]
+ PV-08 numbers out of int range -> ok [2.5.6]/[2.5.99999999999]=ok [2.5.99999999999]/[2.5.6]=UnsupportedPekkoVersion[Current version of Pekko is [2.5.6], but lib requires version [2.5.99999999999]] [2.99999999999.0]/[2.5.0]=NumberFormatException[For input string: "99999999999"] [2.5.0]/[2.99999999999.0]=NumberFormatException[For input string: "99999999999"] [99999999999.5.6]/[99999999999.5.7]=ok [99999999999.5.6]/[99999999998.5.7]=UnsupportedPekkoVersion[Current version of Pekko is [99999999998.5.7], but lib requires version [99999999999.5.6]] [2.5.99999999999]/[2.5.99999999999]=ok [2.5.99999999999]/[2.5.99999999998]=UnsupportedPekkoVersion[Current version of Pekko is [2.5.99999999998], but lib requires version [2.5.99999999999]] [2.5.6-M99999999999]/[2.5.6-M1]=NumberFormatException[For input string: "99999999999"] [2.5.6-M1]/[2.5.6-M99999999999]=NumberFormatException[For input string: "99999999999"] [2.5.6-RC99999999999]/[2.5.6-RC99999999999]=ok [2.5.6-RC99999999999]/[2.5.6-RC99999999998]=NumberFormatException[For input string: "99999999999"] [2.5.2147483648]/[2.5.1]=UnsupportedPekkoVersion[Current version of Pekko is [2.5.1], but lib requires version [2.5.2147483648]] [2.5.1]/[2.5.2147483648]=ok [2.2147483648.1]/[2.1.1]=NumberFormatException[For input string: "2147483648"] [2.5.6-M2147483648]/[2.5.6-M1]=NumberFormatException[For input string: "2147483648"]
```
