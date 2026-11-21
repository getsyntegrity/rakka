# App replay

Build: `$TMPDIR/pekko-mutants-qfq24hxe/target-mailbox-suspended-cannot-be-scheduled/release/examples/classic-parity`

Passed: 2817/2819

Fallback rate: not reported by this build

## Failing cases

### mb-0004

Arguments: `mailbox-case`

stdout:

```diff
- MBX-ST-04 suspended and scheduled -> schedule=true open scheduled=true suspendCount=1 raw=6 canBe(f,f)=false canBe(f,sys)=true | resume=true open scheduled=true suspendCount=0 raw=2
+ MBX-ST-04 suspended and scheduled -> schedule=false open scheduled=false suspendCount=1 raw=4 canBe(f,f)=false canBe(f,sys)=true | resume=true open scheduled=false suspendCount=0 raw=0
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
