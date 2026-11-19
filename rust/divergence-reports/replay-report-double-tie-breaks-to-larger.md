# App replay

Build: `$TMPDIR/pekko-mutants-il_n1gge/target-double-tie-breaks-to-larger/release/examples/classic-parity`

Passed: 2541/2572

Fallback rate: not reported by this build

## Failing cases

### ch-0786

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-SHORT 2e23 -> ok 1.9999999999999998E23
+ NUM-SHORT 2e23 -> ok 2.0E23
```

### ch-0790

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-SHORT -2.2373985447557512E15 -> ok -2.2373985447557512E15
+ NUM-SHORT -2.2373985447557512E15 -> ok -2.2373985447557513E15
```

### ch-0791

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-SHORT 7.243486803726562E10 -> ok 7.243486803726562E10
+ NUM-SHORT 7.243486803726562E10 -> ok 7.243486803726563E10
```

### ch-0792

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-SHORT 3.0643084718565062E13 -> ok 3.0643084718565062E13
+ NUM-SHORT 3.0643084718565062E13 -> ok 3.0643084718565063E13
```

### ch-0793

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-SHORT 7.711434686566412E13 -> ok 7.711434686566412E13
+ NUM-SHORT 7.711434686566412E13 -> ok 7.711434686566413E13
```

### ch-0794

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-SHORT 9.568658294914012E13 -> ok 9.568658294914012E13
+ NUM-SHORT 9.568658294914012E13 -> ok 9.568658294914013E13
```

### ch-0795

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-SHORT 1.6535794220103962E14 -> ok 1.6535794220103962E14
+ NUM-SHORT 1.6535794220103962E14 -> ok 1.6535794220103963E14
```

### ch-0796

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-SHORT 2.6122236014311862E14 -> ok 2.6122236014311862E14
+ NUM-SHORT 2.6122236014311862E14 -> ok 2.6122236014311863E14
```

### ch-0797

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-SHORT 5.804742410468122E14 -> ok 5.804742410468122E14
+ NUM-SHORT 5.804742410468122E14 -> ok 5.804742410468123E14
```

### ch-0798

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-SHORT 6.158966069360882E14 -> ok 6.158966069360882E14
+ NUM-SHORT 6.158966069360882E14 -> ok 6.158966069360883E14
```

### ch-0799

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-SHORT 6.509857562455022E14 -> ok 6.509857562455022E14
+ NUM-SHORT 6.509857562455022E14 -> ok 6.509857562455023E14
```

### ch-0800

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-SHORT 7.011602052238672E14 -> ok 7.011602052238672E14
+ NUM-SHORT 7.011602052238672E14 -> ok 7.011602052238673E14
```

### ch-0801

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-SHORT 7.418392440915202E14 -> ok 7.418392440915202E14
+ NUM-SHORT 7.418392440915202E14 -> ok 7.418392440915203E14
```

### ch-0802

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-SHORT 7.610212372772582E14 -> ok 7.610212372772582E14
+ NUM-SHORT 7.610212372772582E14 -> ok 7.610212372772583E14
```

### ch-0803

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-SHORT 7.893968423321002E14 -> ok 7.893968423321002E14
+ NUM-SHORT 7.893968423321002E14 -> ok 7.893968423321003E14
```

### ch-0804

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-SHORT 8.236185905254162E14 -> ok 8.236185905254162E14
+ NUM-SHORT 8.236185905254162E14 -> ok 8.236185905254163E14
```

### ch-0805

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-SHORT 8.769470039500742E14 -> ok 8.769470039500742E14
+ NUM-SHORT 8.769470039500742E14 -> ok 8.769470039500743E14
```

### ch-0806

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-SHORT 9.227015757318002E14 -> ok 9.227015757318002E14
+ NUM-SHORT 9.227015757318002E14 -> ok 9.227015757318003E14
```

### ch-0807

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-SHORT 9.733920635457882E14 -> ok 9.733920635457882E14
+ NUM-SHORT 9.733920635457882E14 -> ok 9.733920635457883E14
```

### ch-0808

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-SHORT 1.0126863366865002E15 -> ok 1.0126863366865002E15
+ NUM-SHORT 1.0126863366865002E15 -> ok 1.0126863366865003E15
```

### ch-0809

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-SHORT 1.2748364103640552E15 -> ok 1.2748364103640552E15
+ NUM-SHORT 1.2748364103640552E15 -> ok 1.2748364103640553E15
```

### ch-0810

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-SHORT 1.4245519162321872E15 -> ok 1.4245519162321872E15
+ NUM-SHORT 1.4245519162321872E15 -> ok 1.4245519162321873E15
```

### ch-0811

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-SHORT 1.6117025083796642E15 -> ok 1.6117025083796642E15
+ NUM-SHORT 1.6117025083796642E15 -> ok 1.6117025083796643E15
```

### ch-0812

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-SHORT 1.8542559445927642E15 -> ok 1.8542559445927642E15
+ NUM-SHORT 1.8542559445927642E15 -> ok 1.8542559445927643E15
```

### ch-0813

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-SHORT 2.1234256910487892E15 -> ok 2.1234256910487892E15
+ NUM-SHORT 2.1234256910487892E15 -> ok 2.1234256910487893E15
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
