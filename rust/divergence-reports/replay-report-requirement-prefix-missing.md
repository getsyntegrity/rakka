# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-1f9lsxlk/target-requirement-prefix-missing/release/examples/classic-parity`

Passed: 1042/1047

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

### uc-0068

Arguments: `util-core-case`

stdout:

```diff
- RQ-02 5.requiring(false, msg) -> ERR IllegalArgumentException [requirement failed: idle-cpu-level must be between 1 and 10]
+ RQ-02 5.requiring(false, msg) -> ERR IllegalArgumentException [idle-cpu-level must be between 1 and 10]
```

### uc-0069

Arguments: `util-core-case`

stdout:

```diff
- RQ-03 1.requiring(false, 42) -> ERR IllegalArgumentException [requirement failed: 42]
+ RQ-03 1.requiring(false, 42) -> ERR IllegalArgumentException [42]
```

### uc-0070

Arguments: `util-core-case`

stdout:

```diff
- RQ-04 1.requiring(false, empty) -> ERR IllegalArgumentException [requirement failed: ]
+ RQ-04 1.requiring(false, empty) -> ERR IllegalArgumentException []
```

### uc-0072

Arguments: `util-core-case`

stdout:

```diff
- RQ-06 0.requiring(_ > 0) -> ERR IllegalArgumentException [requirement failed: nr-of-selectors must be > 0]
+ RQ-06 0.requiring(_ > 0) -> ERR IllegalArgumentException [nr-of-selectors must be > 0]
```
