# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-b87a6ub3/target-hocon-check-valid-string-reference-needs-string/release/examples/classic-parity`

Passed: 2006/2009

Fallback rate: not reported by this build

## Failing cases

### ch-0169

Arguments: `config-hocon-case`

stdout:

```diff
- CV-08 number where a string is expected -> ok valid
+ CV-08 number where a string is expected -> ERR ValidationFailed [String: 1: pekko.a: Wrong value type at 'pekko.a', expecting: string but got: number]
```

### ch-0182

Arguments: `config-hocon-case`

stdout:

```diff
- CV-21 the real reference.conf fails when a key it defines is an object in the application -> ok valid
+ CV-21 the real reference.conf fails when a key it defines is an object in the application -> ERR ValidationFailed [String: 1: pekko.loglevel: Wrong value type at 'pekko.loglevel', expecting: string but got: object with keys [x]]
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
