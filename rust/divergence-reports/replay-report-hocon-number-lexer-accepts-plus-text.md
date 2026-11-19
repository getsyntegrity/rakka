# App replay

Build: `$TMPDIR/pekko-mutants-4l9e16mc/target-hocon-number-lexer-accepts-plus-text/release/examples/classic-parity`

Passed: 2301/2304

Fallback rate: not reported by this build

## Failing cases

### ch-0536

Arguments: `config-hocon-case`

stdout:

```diff
- NUMLEX 1e+ -> ok error Parse
+ NUMLEX 1e+ -> ok STRING "1e+" 1e+
```

### ch-0553

Arguments: `config-hocon-case`

stdout:

```diff
- NUMLEX 1+2 -> ok error Parse
+ NUMLEX 1+2 -> ok STRING "1+2" 1+2
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
