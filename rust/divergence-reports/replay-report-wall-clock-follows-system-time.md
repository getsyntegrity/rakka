# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-oxtumr43/target-wall-clock-follows-system-time/release/examples/classic-parity`

Passed: 1176/1187

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

### uc-0212

Arguments: `util-core-case`

stdout:

```diff
- WC-03 1000 readings strictly increase -> ok strict=true aboveBefore=true stateIsLast=true
+ WC-03 1000 readings strictly increase -> ok strict=false aboveBefore=true stateIsLast=true
```

### uc-0213

Arguments: `util-core-case`

stdout:

```diff
- WC-04 a clock ahead of the wall time adds one -> ok +1 +2 +3 +4 +5 +6 state=+6
+ WC-04 a clock ahead of the wall time adds one -> ok +-999998204970999842 +-999998204970999842 +-999998204970999842 +-999998204970999842 +-999998204970999842 +-999998204970999842 state=+-999998204970999842
```

### uc-0216

Arguments: `util-core-case`

stdout:

```diff
- WC-07 the increment wraps around at the top of the Long range -> ok a=9223372036854775806 b=9223372036854775807 c=-9223372036854775808 wrapped=true nextIsWall=true
+ WC-07 the increment wraps around at the top of the Long range -> ok a=1795029000161 b=1795029000161 c=1795029000161 wrapped=false nextIsWall=true
```

### uc-0217

Arguments: `util-core-case`

stdout:

```diff
- WC-08 two clocks are independent -> ok a=+1 bIsWall=true aState=+1
+ WC-08 two clocks are independent -> ok a=+-999998204970999834 bIsWall=true aState=+-999998204970999834
```

### uc-0218

Arguments: `util-core-case`

stdout:

```diff
- WC-09 interleaved readings of two clocks -> ok a=+1 b=+101 a=+2 b=+102 a=+3 b=+103 a=+4 b=+104
+ WC-09 interleaved readings of two clocks -> ok a=+-999998204970999829 b=+-999998204970999829 a=+-999998204970999829 b=+-999998204970999829 a=+-999998204970999829 b=+-999998204970999829 a=+-999998204970999829 b=+-999998204970999829
```

### uc-0219

Arguments: `util-core-case`

stdout:

```diff
- WC-10 set between readings -> ok r1=+1 r2=+51 r3=+-49
+ WC-10 set between readings -> ok r1=+-999998204970999829 r2=+-999998204970999829 r3=+-999998204970999829
```

### uc-0220

Arguments: `util-core-case`

stdout:

```diff
- WC-11 the shared instance -> ok same=true strict=true
+ WC-11 the shared instance -> ok same=true strict=false
```

### uc-0221

Arguments: `util-core-case`

stdout:

```diff
- WC-12 8 threads of 500 readings on a clock ahead of the wall time -> ok count=4000 distinct=true eachThreadIncreasing=true contiguous min=+1 max=+4000 state=+4000
+ WC-12 8 threads of 500 readings on a clock ahead of the wall time -> ok count=4000 distinct=false eachThreadIncreasing=false gaps min=+-999998204970999826 max=+-999998204970999826 state=+-999998204970999826
```

### uc-0222

Arguments: `util-core-case`

stdout:

```diff
- WC-13 8 threads of 500 readings on a new clock -> ok count=4000 distinct=true eachThreadIncreasing=true allAtLeastBefore=true
+ WC-13 8 threads of 500 readings on a new clock -> ok count=4000 distinct=false eachThreadIncreasing=false allAtLeastBefore=true
```

### uc-0223

Arguments: `util-core-case`

stdout:

```diff
- WC-14 4 threads of 2000 readings on a clock ahead of the wall time -> ok count=8000 distinct=true eachThreadIncreasing=true contiguous min=+1 max=+8000 state=+8000
+ WC-14 4 threads of 2000 readings on a clock ahead of the wall time -> ok count=8000 distinct=false eachThreadIncreasing=false gaps min=+-999998204970999823 max=+-999998204970999823 state=+-999998204970999823
```
