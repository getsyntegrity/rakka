# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-b87a6ub3/target-hocon-boolean-on-ignores-case/release/examples/classic-parity`

Passed: 2007/2009

Fallback rate: not reported by this build

## Failing cases

### ch-0123

Arguments: `config-hocon-case`

stdout:

```diff
- BOOL "ON" -> ERR WrongType [String: 1: x has type STRING rather than BOOLEAN]
+ BOOL "ON" -> ok true
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
