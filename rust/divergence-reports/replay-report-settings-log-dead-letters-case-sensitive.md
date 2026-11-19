# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-zlxslczp/target-settings-log-dead-letters-case-sensitive/release/examples/classic-parity`

Passed: 2216/2222

Fallback rate: not reported by this build

## Failing cases

### cs-0012

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDL log-dead-letters = ON -> ok LogDeadLetters=2147483647
+ SET-LDL log-dead-letters = ON -> ERR WrongType [String: 1: pekko.log-dead-letters has type STRING rather than NUMBER]
```

### cs-0013

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDL log-dead-letters = Off -> ok LogDeadLetters=0
+ SET-LDL log-dead-letters = Off -> ERR WrongType [String: 1: pekko.log-dead-letters has type STRING rather than NUMBER]
```

### cs-0014

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDL log-dead-letters = TRUE -> ok LogDeadLetters=2147483647
+ SET-LDL log-dead-letters = TRUE -> ERR WrongType [String: 1: pekko.log-dead-letters has type STRING rather than NUMBER]
```

### cs-0015

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDL log-dead-letters = False -> ok LogDeadLetters=0
+ SET-LDL log-dead-letters = False -> ERR WrongType [String: 1: pekko.log-dead-letters has type STRING rather than NUMBER]
```

### cs-0016

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDL log-dead-letters = "ON" -> ok LogDeadLetters=2147483647
+ SET-LDL log-dead-letters = "ON" -> ERR WrongType [String: 1: pekko.log-dead-letters has type STRING rather than NUMBER]
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
