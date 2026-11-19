# App replay

Build: `$TMPDIR/pekko-mutants-tigrcsc1/target-hocon-duration-converts-before-saturating/release/examples/classic-parity`

Passed: 2456/2484

Fallback rate: not reported by this build

## Failing cases

### ch-0576

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT MS 106752 d -> ok 9223372036854
+ DUR-SAT MS 106752 d -> ok 9223372800000
```

### ch-0577

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT SEC 106752 d -> ok 9223372036
+ DUR-SAT SEC 106752 d -> ok 9223372800
```

### ch-0578

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT DAY 106752 d -> ok 106751
+ DUR-SAT DAY 106752 d -> ok 106752
```

### ch-0584

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT MS -106752 d -> ok -9223372036854
+ DUR-SAT MS -106752 d -> ok -9223372800000
```

### ch-0585

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT SEC -106752 d -> ok -9223372036
+ DUR-SAT SEC -106752 d -> ok -9223372800
```

### ch-0586

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT DAY -106752 d -> ok -106751
+ DUR-SAT DAY -106752 d -> ok -106752
```

### ch-0588

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT MS -106753 d -> ok -9223372036854
+ DUR-SAT MS -106753 d -> ok -9223459200000
```

### ch-0589

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT SEC -106753 d -> ok -9223372036
+ DUR-SAT SEC -106753 d -> ok -9223459200
```

### ch-0590

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT DAY -106753 d -> ok -106751
+ DUR-SAT DAY -106753 d -> ok -106753
```

### ch-0592

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT MS 9223372036854775807 ms -> ok 9223372036854
+ DUR-SAT MS 9223372036854775807 ms -> ok 9223372036854775807
```

### ch-0593

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT SEC 9223372036854775807 ms -> ok 9223372036
+ DUR-SAT SEC 9223372036854775807 ms -> ok 9223372036854775
```

### ch-0594

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT DAY 9223372036854775807 ms -> ok 106751
+ DUR-SAT DAY 9223372036854775807 ms -> ok 106751991167
```

### ch-0596

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT MS 9223372036854775807 s -> ok 9223372036854
+ DUR-SAT MS 9223372036854775807 s -> ok 9223372036854775807
```

### ch-0597

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT SEC 9223372036854775807 s -> ok 9223372036
+ DUR-SAT SEC 9223372036854775807 s -> ok 9223372036854775807
```

### ch-0598

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT DAY 9223372036854775807 s -> ok 106751
+ DUR-SAT DAY 9223372036854775807 s -> ok 106751991167300
```

### ch-0688

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT MS 92233720368 s -> ok 9223372036854
+ DUR-SAT MS 92233720368 s -> ok 92233720368000
```

### ch-0689

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT SEC 92233720368 s -> ok 9223372036
+ DUR-SAT SEC 92233720368 s -> ok 92233720368
```

### ch-0690

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT DAY 92233720368 s -> ok 106751
+ DUR-SAT DAY 92233720368 s -> ok 1067519
```

### ch-0692

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT MS 92233720369 s -> ok 9223372036854
+ DUR-SAT MS 92233720369 s -> ok 92233720369000
```

### ch-0693

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT SEC 92233720369 s -> ok 9223372036
+ DUR-SAT SEC 92233720369 s -> ok 92233720369
```

### ch-0694

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT DAY 92233720369 s -> ok 106751
+ DUR-SAT DAY 92233720369 s -> ok 1067519
```

### ch-0696

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT MS 9223372036854775807 -> ok 9223372036854
+ DUR-SAT MS 9223372036854775807 -> ok 9223372036854775807
```

### ch-0697

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT SEC 9223372036854775807 -> ok 9223372036
+ DUR-SAT SEC 9223372036854775807 -> ok 9223372036854775
```

### ch-0698

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SAT DAY 9223372036854775807 -> ok 106751
+ DUR-SAT DAY 9223372036854775807 -> ok 106751991167
```

### cs-0044

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDSD log-dead-letters-suspend-duration = 106752 d -> ok LogDeadLettersSuspendDuration=9223372036854ms
+ SET-LDSD log-dead-letters-suspend-duration = 106752 d -> ERR BadValue [Invalid value at 'pekko.log-dead-letters-suspend-duration': requirement failed: Duration is limited to +-(2^63-1)ns (ca. 292 years)]
```

### cs-0046

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDSD log-dead-letters-suspend-duration = 9223372036854775807 d -> ok LogDeadLettersSuspendDuration=9223372036854ms
+ SET-LDSD log-dead-letters-suspend-duration = 9223372036854775807 d -> ERR BadValue [Invalid value at 'pekko.log-dead-letters-suspend-duration': requirement failed: Duration is limited to +-(2^63-1)ns (ca. 292 years)]
```

### cs-0048

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDSD log-dead-letters-suspend-duration = -9223372036854775808 d -> ok LogDeadLettersSuspendDuration=-9223372036854ms
+ SET-LDSD log-dead-letters-suspend-duration = -9223372036854775808 d -> ERR BadValue [Invalid value at 'pekko.log-dead-letters-suspend-duration': requirement failed: Duration is limited to +-(2^63-1)ns (ca. 292 years)]
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
