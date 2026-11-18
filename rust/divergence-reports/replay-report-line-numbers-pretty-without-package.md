# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-6v3_1z54/target-line-numbers-pretty-without-package/release/examples/classic-parity`

Passed: 1194/1198

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

### uc-0229

Arguments: `util-core-case`

stdout:

```diff
- LN-06 a one-line serializable lambda has a source file and lines -> ok kind=SourceFileLines fileIsBaseName=true linesPositive=true fromLeTo=true prettyIsPackageSlashResult=true single=true
+ LN-06 a one-line serializable lambda has a source file and lines -> ok kind=SourceFileLines fileIsBaseName=true linesPositive=true fromLeTo=true prettyIsPackageSlashResult=false single=true
```

### uc-0230

Arguments: `util-core-case`

stdout:

```diff
- LN-07 a multi-line serializable lambda has a source file and lines -> ok kind=SourceFileLines fileIsBaseName=true linesPositive=true fromLeTo=true prettyIsPackageSlashResult=true
+ LN-07 a multi-line serializable lambda has a source file and lines -> ok kind=SourceFileLines fileIsBaseName=true linesPositive=true fromLeTo=true prettyIsPackageSlashResult=false
```

### uc-0231

Arguments: `util-core-case`

stdout:

```diff
- LN-08 an anonymous class has a source file and lines -> ok kind=SourceFileLines fileIsBaseName=true linesPositive=true fromLeTo=true prettyIsPackageSlashResult=true
+ LN-08 an anonymous class has a source file and lines -> ok kind=SourceFileLines fileIsBaseName=true linesPositive=true fromLeTo=true prettyIsPackageSlashResult=false
```
