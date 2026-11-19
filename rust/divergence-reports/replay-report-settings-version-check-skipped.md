# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-zlxslczp/target-settings-version-check-skipped/release/examples/classic-parity`

Passed: 2217/2222

Fallback rate: not reported by this build

## Failing cases

### cs-0156

Arguments: `config-settings-case`

stdout:

```diff
- SET-VERSION pekko.version = "1.7.1" -> ERR ConfigurationException [Pekko JAR version [2.0.0] does not match the provided config version [1.7.1]]
+ SET-VERSION pekko.version = "1.7.1" -> ok ConfigVersion=1.7.1
```

### cs-0157

Arguments: `config-settings-case`

stdout:

```diff
- SET-VERSION pekko.version = "" -> ERR ConfigurationException [Pekko JAR version [2.0.0] does not match the provided config version []]
+ SET-VERSION pekko.version = "" -> ok ConfigVersion=
```

### cs-0158

Arguments: `config-settings-case`

stdout:

```diff
- SET-VERSION pekko.version = "2.0.0-M1" -> ERR ConfigurationException [Pekko JAR version [2.0.0] does not match the provided config version [2.0.0-M1]]
+ SET-VERSION pekko.version = "2.0.0-M1" -> ok ConfigVersion=2.0.0-M1
```

### cs-0159

Arguments: `config-settings-case`

stdout:

```diff
- SET-VERSION pekko.version = 2 -> ERR ConfigurationException [Pekko JAR version [2.0.0] does not match the provided config version [2]]
+ SET-VERSION pekko.version = 2 -> ok ConfigVersion=2
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
