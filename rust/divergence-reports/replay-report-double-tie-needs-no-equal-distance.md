# App replay

Build: `$TMPDIR/pekko-mutants-5fhmzk1v/target-double-tie-needs-no-equal-distance/release/examples/classic-parity`

Passed: 2567/2576

Fallback rate: not reported by this build

## Failing cases

### ch-0786

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-SHORT 2e23 -> ok 1.9999999999999998E23
+ NUM-SHORT 2e23 -> ok 2.0E23
```

### ch-0814

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-SHORT -4.872628193264896E19 -> ok -4.8726281932648956E19
+ NUM-SHORT -4.872628193264896E19 -> ok -4.872628193264896E19
```

### ch-0815

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-SHORT 9.8411002603808E18 -> ok 9.841100260380799E18
+ NUM-SHORT 9.8411002603808E18 -> ok 9.8411002603808E18
```

### ch-0816

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-SHORT 2.3851354565708684E25 -> ok 2.3851354565708683E25
+ NUM-SHORT 2.3851354565708684E25 -> ok 2.3851354565708684E25
```

### ch-0817

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-SHORT 9.23117748358592E18 -> ok 9.231177483585921E18
+ NUM-SHORT 9.23117748358592E18 -> ok 9.23117748358592E18
```

### ch-0818

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-SHORT -3.5624613975366163E25 -> ok -3.5624613975366162E25
+ NUM-SHORT -3.5624613975366163E25 -> ok -3.5624613975366163E25
```

### ch-0819

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-SHORT 5.960464477539063E-8 -> ok 5.9604644775390625E-8
+ NUM-SHORT 5.960464477539063E-8 -> ok 5.960464477539063E-8
```

### ch-0822

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-SHORT 1.3149029585571635E20 -> ok 1.3149029585571635E20
+ NUM-SHORT 1.3149029585571635E20 -> ok 1.3149029585571636E20
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
