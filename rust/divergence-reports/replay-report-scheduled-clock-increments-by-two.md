# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-j5z5j82_/target-scheduled-clock-increments-by-two/release/examples/classic-parity`

Passed: 1081/1085

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

### uc-0108

Arguments: `util-core-case`

stdout:

```diff
- CK-02 ScheduledClock increments by one until maxIncrement -> ok scheduled=true delayNanos=100003 reached=false c1=+1,reached=false c2=+2,reached=false c3=+3,reached=true c4=+3,reached=true c5=+3,reached=true c6=+3,reached=true earlier(0)=+3 earlier(5)=+-2
+ CK-02 ScheduledClock increments by one until maxIncrement -> ok scheduled=true delayNanos=100003 reached=false c1=+2,reached=false c2=+4,reached=true c3=+4,reached=true c4=+4,reached=true c5=+4,reached=true c6=+4,reached=true earlier(0)=+4 earlier(5)=+-1
```

### uc-0109

Arguments: `util-core-case`

stdout:

```diff
- CK-03 ScheduledClock maxIncrement boundaries -> ok interval=1: +1 +1 +1 +1 +1 +1 reached=true; interval=100000: +1 +1 +1 +1 +1 +1 reached=true; interval=100001: +1 +1 +1 +1 +1 +1 reached=true; interval=100002: +1 +2 +2 +2 +2 +2 reached=true; interval=100004: +1 +2 +3 +4 +4 +4 reached=true; interval=1000000: n=900000 +900000 reached=true n+1 +900000 n+2 +900000
+ CK-03 ScheduledClock maxIncrement boundaries -> ok interval=1: +2 +2 +2 +2 +2 +2 reached=true; interval=100000: +2 +2 +2 +2 +2 +2 reached=true; interval=100001: +2 +2 +2 +2 +2 +2 reached=true; interval=100002: +2 +2 +2 +2 +2 +2 reached=true; interval=100004: +2 +4 +4 +4 +4 +4 reached=true; interval=1000000: n=900000 +900000 reached=true n+1 +900000 n+2 +900000
```

### uc-0110

Arguments: `util-core-case`

stdout:

```diff
- CK-04 ScheduledClock update resumes the increments -> ok reachedBefore=true stuck=+3 movedForward=true reachedAfter=false nextIsPlusOne=true
+ CK-04 ScheduledClock update resumes the increments -> ok reachedBefore=true stuck=+4 movedForward=true reachedAfter=false nextIsPlusOne=false
```
