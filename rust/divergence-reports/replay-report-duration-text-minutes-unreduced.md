# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-1r57p1zd/target-duration-text-minutes-unreduced/release/examples/classic-parity`

Passed: 1509/1513

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

### uc-0033

Arguments: `util-core-case`

stdout:

```diff
- TO-NEG-04 create(ofSeconds(9223372036, 854775808)) -> ERR IllegalArgumentException [Java duration PT2562047H47M16.854775808S cannot be expressed as a Scala duration]
+ TO-NEG-04 create(ofSeconds(9223372036, 854775808)) -> ERR IllegalArgumentException [Java duration PT2562047H153722867M16.854775808S cannot be expressed as a Scala duration]
```

### uc-0545

Arguments: `util-core-case`

stdout:

```diff
- FR-DUR-09 create(ofSeconds(9223372037, 1)) -> ERR IllegalArgumentException [Java duration PT2562047H47M17.000000001S cannot be expressed as a Scala duration]
+ FR-DUR-09 create(ofSeconds(9223372037, 1)) -> ERR IllegalArgumentException [Java duration PT2562047H153722867M17.000000001S cannot be expressed as a Scala duration]
```

### uc-0547

Arguments: `util-core-case`

stdout:

```diff
- FR-DUR-11 create(ofSeconds(Long.MAX_VALUE, 999999999)) -> ERR IllegalArgumentException [Java duration PT2562047788015215H30M7.999999999S cannot be expressed as a Scala duration]
+ FR-DUR-11 create(ofSeconds(Long.MAX_VALUE, 999999999)) -> ERR IllegalArgumentException [Java duration PT2562047788015215H153722867280912930M7.999999999S cannot be expressed as a Scala duration]
```
