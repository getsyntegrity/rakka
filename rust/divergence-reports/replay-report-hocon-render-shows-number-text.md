# App replay

Build: `$TMPDIR/pekko-mutants-5pjgu_cj/target-hocon-render-shows-number-text/release/examples/classic-parity`

Passed: 2224/2258

Fallback rate: not reported by this build

## Failing cases

### ch-0183

Arguments: `config-hocon-case`

stdout:

```diff
- RW-01 render of scalars, nested objects, lists, null and empty values -> ok {\u000A    # String: 1\u000A    "a" : 1,\u000A    # String: 2\u000A    "b" : "hello",\u000A    # String: 8\u000A    "bool" : true,\u000A    # String: 3\u000A    "c" : {\u000A        # String: 3\u000A        "d" : [\u000A            # String: 3\u000A            1,\u000A            # String: 3\u000A            2\u000A        ],\u000A        # String: 3\u000A        "e" : "x y"\u000A    },\u000A    # String: 7\u000A    "f" : 1.5,\u000A    # String: 4\u000A    "l" : [],\u000A    # String: 6\u000A    "n" : null,\u000A    # String: 5\u000A    "o" : {},\u000A    # String: 9\u000A    "w" : 1\u000A}\u000A
+ RW-01 render of scalars, nested objects, lists, null and empty values -> ok {\u000A    # String: 1\u000A    "a" : 1,\u000A    # String: 2\u000A    "b" : "hello",\u000A    # String: 8\u000A    "bool" : true,\u000A    # String: 3\u000A    "c" : {\u000A        # String: 3\u000A        "d" : [\u000A            # String: 3\u000A            1,\u000A            # String: 3\u000A            2\u000A        ],\u000A        # String: 3\u000A        "e" : "x y"\u000A    },\u000A    # String: 7\u000A    "f" : 1.5,\u000A    # String: 4\u000A    "l" : [],\u000A    # String: 6\u000A    "n" : null,\u000A    # String: 5\u000A    "o" : {},\u000A    # String: 9\u000A    "w" : 1.0\u000A}\u000A
```

### ch-0217

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.default-dispatcher.fork-join-executor.parallelism-factor -> ok NUMBER 1
+ KEY actor pekko.actor.default-dispatcher.fork-join-executor.parallelism-factor -> ok NUMBER 1.0
```

### ch-0226

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.default-dispatcher.thread-pool-executor.core-pool-size-factor -> ok NUMBER 3
+ KEY actor pekko.actor.default-dispatcher.thread-pool-executor.core-pool-size-factor -> ok NUMBER 3.0
```

### ch-0231

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.default-dispatcher.thread-pool-executor.max-pool-size-factor -> ok NUMBER 3
+ KEY actor pekko.actor.default-dispatcher.thread-pool-executor.max-pool-size-factor -> ok NUMBER 3.0
```

### ch-0285

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.internal-dispatcher.fork-join-executor.parallelism-factor -> ok NUMBER 1
+ KEY actor pekko.actor.internal-dispatcher.fork-join-executor.parallelism-factor -> ok NUMBER 1.0
```

### ch-0359

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.circuit-breaker.default.exponential-backoff -> ok NUMBER 1
+ KEY actor pekko.circuit-breaker.default.exponential-backoff -> ok NUMBER 1.0
```

### ch-0362

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.circuit-breaker.default.random-factor -> ok NUMBER 0
+ KEY actor pekko.circuit-breaker.default.random-factor -> ok NUMBER 0.0
```

### ch-0497

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER 1.50 -> ok 1.5
+ NUM-RENDER 1.50 -> ok 1.50
```

### ch-0498

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER 1e-7 -> ok 1.0E-7
+ NUM-RENDER 1e-7 -> ok 1e-7
```

### ch-0499

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER 12345678.9 -> ok 1.23456789E7
+ NUM-RENDER 12345678.9 -> ok 12345678.9
```

### ch-0500

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER -0 -> ok 0
+ NUM-RENDER -0 -> ok -0
```

### ch-0501

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER 010 -> ok 10
+ NUM-RENDER 010 -> ok 010
```

### ch-0502

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER 1e400 -> ok Infinity
+ NUM-RENDER 1e400 -> ok 1e400
```

### ch-0503

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER 1.5e-3 -> ok 0.0015
+ NUM-RENDER 1.5e-3 -> ok 1.5e-3
```

### ch-0504

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER 5e18 -> ok 5000000000000000000
+ NUM-RENDER 5e18 -> ok 5e18
```

### ch-0505

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER 1e18 -> ok 1000000000000000000
+ NUM-RENDER 1e18 -> ok 1e18
```

### ch-0506

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER 1.0 -> ok 1
+ NUM-RENDER 1.0 -> ok 1.0
```

### ch-0507

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER 100.0 -> ok 100
+ NUM-RENDER 100.0 -> ok 100.0
```

### ch-0509

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER 1E5 -> ok 100000
+ NUM-RENDER 1E5 -> ok 1E5
```

### ch-0510

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER 1e2 -> ok 100
+ NUM-RENDER 1e2 -> ok 1e2
```

### ch-0512

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER -1.50 -> ok -1.5
+ NUM-RENDER -1.50 -> ok -1.50
```

### ch-0513

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER 0.0 -> ok 0
+ NUM-RENDER 0.0 -> ok 0.0
```

### ch-0514

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER 1e+2 -> ok 100
+ NUM-RENDER 1e+2 -> ok 1e+2
```

### ch-0515

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER -0.0 -> ok 0
+ NUM-RENDER -0.0 -> ok -0.0
```

### ch-0517

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER 1e19 -> ok 1.0E19
+ NUM-RENDER 1e19 -> ok 1e19
```

### ch-0518

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER -1e19 -> ok -1.0E19
+ NUM-RENDER -1e19 -> ok -1e19
```

### ch-0519

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER 9223372036854775808.0 -> ok 9223372036854775807
+ NUM-RENDER 9223372036854775808.0 -> ok 9223372036854775808.0
```

### ch-0520

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER 1.7976931348623157e308 -> ok 1.7976931348623157E308
+ NUM-RENDER 1.7976931348623157e308 -> ok 1.7976931348623157e308
```

### ch-0521

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER 4.9e-324 -> ok 4.9E-324
+ NUM-RENDER 4.9e-324 -> ok 4.9e-324
```

### ch-0524

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER 0.0001 -> ok 1.0E-4
+ NUM-RENDER 0.0001 -> ok 0.0001
```

### ch-0525

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER 1234567.0 -> ok 1234567
+ NUM-RENDER 1234567.0 -> ok 1234567.0
```

### ch-0526

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER 12345678.5 -> ok 1.23456785E7
+ NUM-RENDER 12345678.5 -> ok 12345678.5
```

### ch-0527

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER -2.5e-5 -> ok -2.5E-5
+ NUM-RENDER -2.5e-5 -> ok -2.5e-5
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
