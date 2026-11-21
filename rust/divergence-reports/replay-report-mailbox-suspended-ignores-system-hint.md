# App replay

Build: `$TMPDIR/pekko-mutants-qfq24hxe/target-mailbox-suspended-ignores-system-hint/release/examples/classic-parity`

Passed: 2816/2819

Fallback rate: not reported by this build

## Failing cases

### mb-0004

Arguments: `mailbox-case`

stdout:

```diff
- MBX-ST-04 suspended and scheduled -> schedule=true open scheduled=true suspendCount=1 raw=6 canBe(f,f)=false canBe(f,sys)=true | resume=true open scheduled=true suspendCount=0 raw=2
+ MBX-ST-04 suspended and scheduled -> schedule=true open scheduled=true suspendCount=1 raw=6 canBe(f,f)=false canBe(f,sys)=false | resume=true open scheduled=true suspendCount=0 raw=2
```

### mb-0008

Arguments: `mailbox-case`

stdout:

```diff
- MBX-ST-08 canBeScheduledForExecution matrix -> open: FTTT withSystemMessage=true; scheduled: FTTT withSystemMessage=true; suspended: FTFT withSystemMessage=true; suspended+scheduled: FTFT withSystemMessage=true; closed: FFFF withSystemMessage=false;
+ MBX-ST-08 canBeScheduledForExecution matrix -> open: FTTT withSystemMessage=true; scheduled: FTTT withSystemMessage=true; suspended: FFFF withSystemMessage=true; suspended+scheduled: FFFF withSystemMessage=true; closed: FFFF withSystemMessage=false;
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
