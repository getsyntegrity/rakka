# App replay

Build: `$TMPDIR/pekko-mutants-tigrcsc1/target-hocon-duration-trailing-point-is-an-error/release/examples/classic-parity`

Passed: 2446/2484

Fallback rate: not reported by this build

## Failing cases

### ch-0075

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SCI scientific notation is accepted -> ok 1000
+ DUR-SCI scientific notation is accepted -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '1e3']
```

### ch-0599

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT NS 1e30 s -> ok 9223372036854775807
+ DUR-SAT NS 1e30 s -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '1e30']
```

### ch-0600

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT MS 1e30 s -> ok 9223372036854
+ DUR-SAT MS 1e30 s -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '1e30']
```

### ch-0601

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT SEC 1e30 s -> ok 9223372036
+ DUR-SAT SEC 1e30 s -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '1e30']
```

### ch-0602

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT DAY 1e30 s -> ok 106751
+ DUR-SAT DAY 1e30 s -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '1e30']
```

### ch-0603

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT NS -1e30 s -> ok -9223372036854775808
+ DUR-SAT NS -1e30 s -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '-1e30']
```

### ch-0604

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT MS -1e30 s -> ok -9223372036854
+ DUR-SAT MS -1e30 s -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '-1e30']
```

### ch-0605

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT SEC -1e30 s -> ok -9223372036
+ DUR-SAT SEC -1e30 s -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '-1e30']
```

### ch-0606

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT DAY -1e30 s -> ok -106751
+ DUR-SAT DAY -1e30 s -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '-1e30']
```

### ch-0607

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT NS 1e18 ns -> ok 1000000000000000000
+ DUR-SAT NS 1e18 ns -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '1e18']
```

### ch-0608

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT MS 1e18 ns -> ok 1000000000000
+ DUR-SAT MS 1e18 ns -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '1e18']
```

### ch-0609

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT SEC 1e18 ns -> ok 1000000000
+ DUR-SAT SEC 1e18 ns -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '1e18']
```

### ch-0610

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT DAY 1e18 ns -> ok 11574
+ DUR-SAT DAY 1e18 ns -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '1e18']
```

### ch-0619

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT NS .5 s -> ok 500000000
+ DUR-SAT NS .5 s -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '.5']
```

### ch-0620

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT MS .5 s -> ok 500
+ DUR-SAT MS .5 s -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '.5']
```

### ch-0621

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT SEC .5 s -> ok 0
+ DUR-SAT SEC .5 s -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '.5']
```

### ch-0622

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT DAY .5 s -> ok 0
+ DUR-SAT DAY .5 s -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '.5']
```

### ch-0631

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT NS -.5 s -> ok -500000000
+ DUR-SAT NS -.5 s -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '-.5']
```

### ch-0632

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT MS -.5 s -> ok -500
+ DUR-SAT MS -.5 s -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '-.5']
```

### ch-0633

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT SEC -.5 s -> ok 0
+ DUR-SAT SEC -.5 s -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '-.5']
```

### ch-0634

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT DAY -.5 s -> ok 0
+ DUR-SAT DAY -.5 s -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '-.5']
```

### ch-0635

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT NS 5. s -> ok 5000000000
+ DUR-SAT NS 5. s -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '5.']
```

### ch-0636

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT MS 5. s -> ok 5000
+ DUR-SAT MS 5. s -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '5.']
```

### ch-0637

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT SEC 5. s -> ok 5
+ DUR-SAT SEC 5. s -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '5.']
```

### ch-0638

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT DAY 5. s -> ok 0
+ DUR-SAT DAY 5. s -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '5.']
```

### ch-0659

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT NS 1E3 ms -> ok 1000000000
+ DUR-SAT NS 1E3 ms -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '1E3']
```

### ch-0660

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT MS 1E3 ms -> ok 1000
+ DUR-SAT MS 1E3 ms -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '1E3']
```

### ch-0661

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT SEC 1E3 ms -> ok 1
+ DUR-SAT SEC 1E3 ms -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '1E3']
```

### ch-0662

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT DAY 1E3 ms -> ok 0
+ DUR-SAT DAY 1E3 ms -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '1E3']
```

### ch-0663

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT NS 5e-1 s -> ok 500000000
+ DUR-SAT NS 5e-1 s -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '5e-1']
```

### ch-0664

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT MS 5e-1 s -> ok 500
+ DUR-SAT MS 5e-1 s -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '5e-1']
```

### ch-0665

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT SEC 5e-1 s -> ok 0
+ DUR-SAT SEC 5e-1 s -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '5e-1']
```

### ch-0666

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT DAY 5e-1 s -> ok 0
+ DUR-SAT DAY 5e-1 s -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '5e-1']
```

### ch-0707

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT NS 3e-10 d -> ok 25920
+ DUR-SAT NS 3e-10 d -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '3e-10']
```

### ch-0708

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT MS 3e-10 d -> ok 0
+ DUR-SAT MS 3e-10 d -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '3e-10']
```

### ch-0709

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT SEC 3e-10 d -> ok 0
+ DUR-SAT SEC 3e-10 d -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '3e-10']
```

### ch-0710

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT DAY 3e-10 d -> ok 0
+ DUR-SAT DAY 3e-10 d -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '3e-10']
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
