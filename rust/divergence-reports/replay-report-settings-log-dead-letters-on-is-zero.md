# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-zlxslczp/target-settings-log-dead-letters-on-is-zero/release/examples/classic-parity`

Passed: 2217/2222

Fallback rate: not reported by this build

## Failing cases

### cs-0008

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDL log-dead-letters = on -> ok LogDeadLetters=2147483647
+ SET-LDL log-dead-letters = on -> ok LogDeadLetters=0
```

### cs-0012

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDL log-dead-letters = ON -> ok LogDeadLetters=2147483647
+ SET-LDL log-dead-letters = ON -> ok LogDeadLetters=0
```

### cs-0014

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDL log-dead-letters = TRUE -> ok LogDeadLetters=2147483647
+ SET-LDL log-dead-letters = TRUE -> ok LogDeadLetters=0
```

### cs-0016

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDL log-dead-letters = "ON" -> ok LogDeadLetters=2147483647
+ SET-LDL log-dead-letters = "ON" -> ok LogDeadLetters=0
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
