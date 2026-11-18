# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-j5z5j82_/target-utc-string-milliseconds-unpadded/release/examples/classic-parity`

Passed: 1082/1085

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

### uc-0101

Arguments: `util-core-case`

stdout:

```diff
- TS-UTC-01 currentTimeMillisToUTCString -> ok [0]=00:00:00.000UTC [1]=00:00:00.001UTC [9]=00:00:00.009UTC [10]=00:00:00.010UTC [99]=00:00:00.099UTC [100]=00:00:00.100UTC [999]=00:00:00.999UTC [1000]=00:00:01.000UTC [59999]=00:00:59.999UTC [60000]=00:01:00.000UTC [3599999]=00:59:59.999UTC [3600000]=01:00:00.000UTC [36000000]=10:00:00.000UTC [86399999]=23:59:59.999UTC [86400000]=00:00:00.000UTC [86400001]=00:00:00.001UTC [90061001]=01:01:01.001UTC [1557014400000]=00:00:00.000UTC [1557100799999]=23:59:59.999UTC [9223372036854775807]=07:12:55.807UTC
+ TS-UTC-01 currentTimeMillisToUTCString -> ok [0]=00:00:00.00UTC [1]=00:00:00.01UTC [9]=00:00:00.09UTC [10]=00:00:00.10UTC [99]=00:00:00.99UTC [100]=00:00:00.100UTC [999]=00:00:00.999UTC [1000]=00:00:01.00UTC [59999]=00:00:59.999UTC [60000]=00:01:00.00UTC [3599999]=00:59:59.999UTC [3600000]=01:00:00.00UTC [36000000]=10:00:00.00UTC [86399999]=23:59:59.999UTC [86400000]=00:00:00.00UTC [86400001]=00:00:00.01UTC [90061001]=01:01:01.01UTC [1557014400000]=00:00:00.00UTC [1557100799999]=23:59:59.999UTC [9223372036854775807]=07:12:55.807UTC
```

### uc-0102

Arguments: `util-core-case`

stdout:

```diff
- TS-UTC-02 negative timestamps -> ok [-1]=00:00:00.00-1UTC [-9]=00:00:00.00-9UTC [-10]=00:00:00.00-10UTC [-999]=00:00:00.00-999UTC [-1000]=00:00:0-1.000UTC [-60000]=00:0-1:00.000UTC [-3600000]=0-1:00:00.000UTC [-36000000]=0-10:00:00.000UTC [-86399999]=0-23:0-59:0-59.00-999UTC [-86400000]=00:00:00.000UTC [-86400001]=00:00:00.00-1UTC [-1234567]=00:0-20:0-34.00-567UTC [-9223372036854775808]=0-7:0-12:0-55.00-808UTC
+ TS-UTC-02 negative timestamps -> ok [-1]=00:00:00.0-1UTC [-9]=00:00:00.0-9UTC [-10]=00:00:00.0-10UTC [-999]=00:00:00.0-999UTC [-1000]=00:00:0-1.00UTC [-60000]=00:0-1:00.00UTC [-3600000]=0-1:00:00.00UTC [-36000000]=0-10:00:00.00UTC [-86399999]=0-23:0-59:0-59.0-999UTC [-86400000]=00:00:00.00UTC [-86400001]=00:00:00.0-1UTC [-1234567]=00:0-20:0-34.0-567UTC [-9223372036854775808]=0-7:0-12:0-55.0-808UTC
```
